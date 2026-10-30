//! 统一多格式文件加载器 (支持 BFEVFL 流程图, BFEVTM 时间线, BFRES 3D 模型与通用代码文件)

use crate::model::code_document::{CodeDocumentState, CodeFormat};
use crate::model::file_converter::parse_file_to_code;
use crate::model::file_tree::read_file_from_path;
use crate::model::{DocumentTabKind, OpenDocumentTab};
use crate::ui::components::Toast;
use crate::ui::state::{AppState, AppWorkspaceTab};

impl AppState {
    /// 根据虚拟路径或物理路径加载事件流、时间线、3D 模型或数据文件至编辑器
    pub fn load_file_by_path(&mut self, path: &str) {
        self.clear_active_text_inputs();
        self.selected_file_path = Some(path.to_string());

        let raw_name = path
            .rsplit("//")
            .next()
            .unwrap_or(path)
            .rsplit('/')
            .next()
            .unwrap_or(path)
            .rsplit('\\')
            .next()
            .unwrap_or(path)
            .to_string();

        let lower_name = raw_name.to_lowercase();

        // 缓存当前旧活跃标签图
        if let Some(active_id) = &self.active_tab_id {
            if let Some(tab) = self.open_tabs.iter_mut().find(|t| &t.id == active_id) {
                if tab.tab_kind == DocumentTabKind::Flowchart {
                    tab.graph = Some(self.graph.clone());
                }
            }
        }

        // 如果该文件已存在打开标签页，直接切换
        if let Some(existing) = self.open_tabs.iter().find(|t| t.file_path == path) {
            let id = existing.id.clone();
            self.switch_document_tab(&id);
            return;
        }

        let bytes = match read_file_from_path(path) {
            Ok(b) => b,
            Err(e) => {
                self.toast = Some(Toast::error(format!("{} {}", self.t("toast.file_load_error"), e)));
                return;
            }
        };

        let decompressed = crate::model::file_converter::decompress_if_yaz0(&bytes);

        // 1. 优先将 BFEVTM 打开为时间线模式，并同步生成原生 EventFlow 代码状态
        if (lower_name.ends_with(".bfevtm") || lower_name.ends_with(".evtm"))
            && self.load_timeline_tab(path, &raw_name, &decompressed)
        {
            return;
        }

        // 2. 检查 BFEVFL 流程图或普通 BFEVFL 二进制文件
        if (lower_name.ends_with(".bfevfl") || bytes.starts_with(b"BFEVFL"))
            && self.load_flowchart_tab(path, &raw_name, &decompressed)
        {
            return;
        }

        // 3. 检查 Nintendo BFRES 3D 模型归档 (如 .sbfres, .bfres 或以魔数 FRES 开头)
        let is_bfres_file = lower_name.ends_with(".bfres")
            || lower_name.ends_with(".sbfres")
            || decompressed.starts_with(b"FRES");
        if is_bfres_file && self.load_bfres_model_tab(path, &raw_name, &decompressed) {
            return;
        }

        // 4. 通用代码编辑器解析：支持 MSBT, BYML, AAMP, BAIPROG, JSON, YAML, TXT 等任意可打开文件
        match parse_file_to_code(&bytes, path) {
            Ok((code_text, format, endian)) => {
                let code_state = CodeDocumentState::new(
                    code_text,
                    format,
                    endian,
                    path.to_string(),
                );
                let new_id = uuid::Uuid::new_v4().to_string();
                let file_type = format.language_id().to_string();
                let new_tab = OpenDocumentTab::new_code(
                    new_id.clone(),
                    raw_name.clone(),
                    path.to_string(),
                    file_type,
                    code_state,
                );

                self.open_tabs.push(new_tab);
                self.active_tab_id = Some(new_id);

                if format == CodeFormat::Msbt {
                    self.sync_msbt_selected_entry_to_draft();
                }

                self.current_tab = AppWorkspaceTab::CodeEditor;
                self.toast = Some(Toast::success(format!("{} {}", self.t("toast.file_loaded"), raw_name)));
            }
            Err(err) => {
                self.toast = Some(Toast::error(format!("{} {}", self.t("toast.file_load_error"), err)));
            }
        }
    }

    /// 打开 BFEVTM 时间线文件
    fn load_timeline_tab(&mut self, path: &str, raw_name: &str, decompressed: &[u8]) -> bool {
        match crate::model::evfl::load_eventflow_from_bytes(decompressed) {
            Ok(evfl) => {
                let json_str = serde_json::to_string_pretty(&evfl).unwrap_or_default();
                if let Some(mut timeline) = evfl.timeline {
                    if timeline.name.is_empty() {
                        timeline.name = raw_name.trim_end_matches(".bfevtm").to_string();
                    }
                    let code_state = CodeDocumentState::new(
                        json_str,
                        CodeFormat::Bfevtm,
                        crate::model::code_document::CodeEndian::Little,
                        path.to_string(),
                    );
                    let new_id = uuid::Uuid::new_v4().to_string();
                    let mut new_tab = OpenDocumentTab::new_timeline(
                        new_id.clone(),
                        raw_name.to_string(),
                        path.to_string(),
                        "bfevtm".to_string(),
                        timeline,
                    );
                    new_tab.code_state = Some(code_state);
                    self.open_tabs.push(new_tab);
                    self.active_tab_id = Some(new_id);
                    self.current_tab = AppWorkspaceTab::Timeline;
                    self.toast = Some(Toast::success(format!("{} {}", self.t("toast.file_loaded"), raw_name)));
                    return true;
                }
                false
            }
            Err(err) => {
                self.toast = Some(Toast::error(format!("{} {}", self.t("toast.file_load_error"), err)));
                true
            }
        }
    }

    /// 打开 BFEVFL 事件流程图文件
    fn load_flowchart_tab(&mut self, path: &str, raw_name: &str, decompressed: &[u8]) -> bool {
        let evfl_res = crate::model::evfl::load_eventflow_from_bytes(decompressed);

        if let Ok(ref evfl) = evfl_res {
            if evfl.flowchart.is_none() && evfl.timeline.is_some() {
                let timeline = evfl.timeline.clone().unwrap_or_default();
                let code_state = CodeDocumentState::new(
                    serde_json::to_string_pretty(evfl).unwrap_or_default(),
                    CodeFormat::Bfevtm,
                    crate::model::code_document::CodeEndian::Little,
                    path.to_string(),
                );
                let new_id = uuid::Uuid::new_v4().to_string();
                let mut new_tab = OpenDocumentTab::new_timeline(
                    new_id.clone(),
                    raw_name.to_string(),
                    path.to_string(),
                    "bfevtm".to_string(),
                    timeline,
                );
                new_tab.code_state = Some(code_state);
                self.open_tabs.push(new_tab);
                self.active_tab_id = Some(new_id);
                self.current_tab = AppWorkspaceTab::Timeline;
                self.toast = Some(Toast::success(format!("{} {}", self.t("toast.file_loaded"), raw_name)));
                return true;
            }
        }

        let graph_res = crate::model::evfl::load_bfevfl_from_bytes(decompressed);
        match graph_res {
            Ok(mut loaded_graph) => {
                loaded_graph.name = raw_name.to_string();
                loaded_graph.ensure_actors_populated();
                crate::model::layout::auto_layout_graph(&mut loaded_graph);
                self.graph = loaded_graph.clone();

                let code_state_opt = evfl_res.ok().map(|evfl| {
                    let json_text = serde_json::to_string_pretty(&evfl).unwrap_or_default();
                    CodeDocumentState::new(
                        json_text,
                        CodeFormat::Bfevfl,
                        crate::model::code_document::CodeEndian::Little,
                        path.to_string(),
                    )
                });

                let new_id = uuid::Uuid::new_v4().to_string();
                let mut new_tab = OpenDocumentTab::new(
                    new_id.clone(),
                    raw_name.to_string(),
                    path.to_string(),
                    "bfevfl".to_string(),
                    Some(loaded_graph.clone()),
                    Some(loaded_graph),
                );
                new_tab.code_state = code_state_opt;
                self.open_tabs.push(new_tab);
                self.active_tab_id = Some(new_id);

                self.fit_view();
                self.current_tab = AppWorkspaceTab::Flowchart;
                self.toast = Some(Toast::success(format!("{} {}", self.t("toast.file_loaded"), raw_name)));
                true
            }
            Err(err) => {
                self.toast = Some(Toast::error(format!("{} {}", self.t("toast.file_load_error"), err)));
                true
            }
        }
    }

    /// 打开 BFRES / SBFRES 3D 模型归档并构建模型预览标签页
    fn load_bfres_model_tab(&mut self, path: &str, raw_name: &str, decompressed: &[u8]) -> bool {
        match rbfres_library::BfresFile::from_bytes(decompressed) {
            Ok(bfres) => {
                let mut model_state =
                    crate::model::ModelDocumentState::from_bfres(path, raw_name, &bfres);
                crate::model::model_preview::load_companion_textures(
                    path,
                    &bfres,
                    &mut model_state,
                );
                let new_id = uuid::Uuid::new_v4().to_string();
                let new_tab = OpenDocumentTab::new_model(
                    new_id.clone(),
                    raw_name.to_string(),
                    path.to_string(),
                    "bfres".to_string(),
                    model_state,
                );

                self.open_tabs.push(new_tab);
                self.active_tab_id = Some(new_id);
                self.current_tab = AppWorkspaceTab::ModelPreview;
                self.toast = Some(Toast::success(format!("{} {}", self.t("toast.file_loaded"), raw_name)));
                true
            }
            Err(err) => {
                self.toast = Some(Toast::error(format!("{} {}", self.t("toast.file_load_error"), err)));
                true
            }
        }
    }
}

