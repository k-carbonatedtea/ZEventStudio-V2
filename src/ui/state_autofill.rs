use std::path::PathBuf;
use crate::model::{NodeKind, PlatformType, PropertyItem};
use crate::ui::components::Toast;
use crate::ui::state::AppState;

impl AppState {
    /// 自动收集所有可用的游戏 AI 搜寻路径 (合并自定义目录、工程路径、Switch 与 Wii U 目录及 BCML/Zevent 配置文件)
    pub fn build_game_ai_search_dirs(&self) -> Vec<PathBuf> {
        let mut explicit = Vec::new();

        // 1. 自定义覆盖目录
        if !self.game_paths.custom_ai_dump_dir.trim().is_empty() {
            explicit.push(PathBuf::from(&self.game_paths.custom_ai_dump_dir));
        }

        // 2. 当前工程路径 (若包含解包资源或 mod 封包)
        if let Some(ref proj_path) = self.project_path {
            if !proj_path.trim().is_empty() {
                explicit.push(PathBuf::from(proj_path));
            }
        }

        // 3. 当前运行平台路径优先，另一平台作为回退 (不管是 Switch 还是 Wii U，均自动支持)
        if self.game_paths.current_platform == PlatformType::Switch {
            if !self.game_paths.switch.game_dir.trim().is_empty() {
                explicit.push(PathBuf::from(&self.game_paths.switch.game_dir));
            }
            if !self.game_paths.switch.dlc_dir.trim().is_empty() {
                explicit.push(PathBuf::from(&self.game_paths.switch.dlc_dir));
            }
            // 回退：Wii U 补丁与本体路径
            if !self.game_paths.wiiu.update_dir.trim().is_empty() {
                explicit.push(PathBuf::from(&self.game_paths.wiiu.update_dir));
            }
            if !self.game_paths.wiiu.game_dir.trim().is_empty() {
                explicit.push(PathBuf::from(&self.game_paths.wiiu.game_dir));
            }
            if !self.game_paths.wiiu.dlc_dir.trim().is_empty() {
                explicit.push(PathBuf::from(&self.game_paths.wiiu.dlc_dir));
            }
        } else {
            // Wii U 优先
            if !self.game_paths.wiiu.update_dir.trim().is_empty() {
                explicit.push(PathBuf::from(&self.game_paths.wiiu.update_dir));
            }
            if !self.game_paths.wiiu.game_dir.trim().is_empty() {
                explicit.push(PathBuf::from(&self.game_paths.wiiu.game_dir));
            }
            if !self.game_paths.wiiu.dlc_dir.trim().is_empty() {
                explicit.push(PathBuf::from(&self.game_paths.wiiu.dlc_dir));
            }
            // 回退：Switch 路径
            if !self.game_paths.switch.game_dir.trim().is_empty() {
                explicit.push(PathBuf::from(&self.game_paths.switch.game_dir));
            }
            if !self.game_paths.switch.dlc_dir.trim().is_empty() {
                explicit.push(PathBuf::from(&self.game_paths.switch.dlc_dir));
            }
        }

        crate::model::ai_def::build_search_dirs(&explicit, Some(&self.game_paths.custom_ai_dump_dir))
    }

    /// 基于 AI 定义文件自动填充指定节点 (Action/Query/Switch) 的参数属性
    pub fn autofill_node_parameters(&mut self, node_id: &str) {
        let (actor_name, action_query_name, is_query) = {
            if let Some(node) = self.graph.find_node(node_id) {
                let is_q = node.kind == NodeKind::Query || node.kind == NodeKind::Switch;
                if node.kind != NodeKind::Action && !is_q {
                    return;
                }

                let actor = node.properties.iter()
                    .find(|p| p.key == "ActorName")
                    .map(|p| p.val.clone())
                    .unwrap_or_else(|| node.subtitle.clone().unwrap_or_default());
                let name = node.properties.iter()
                    .find(|p| p.key == "Name")
                    .map(|p| p.val.clone())
                    .unwrap_or_else(|| node.title.clone());

                let clean_actor = actor.split('[').next().unwrap_or(&actor);
                let clean_actor = clean_actor.split('(').next().unwrap_or(clean_actor).trim().to_string();
                let clean_name = name.trim().to_string();

                if clean_actor.is_empty() || clean_name.is_empty() {
                    self.toast = Some(Toast::error(self.t("toast.node_missing_info")));
                    return;
                }
                (clean_actor, clean_name, is_q)
            } else {
                return;
            }
        };

        let search_dirs = self.build_game_ai_search_dirs();

        match crate::model::ai_def::get_autofill_parameters(
            &actor_name, &action_query_name, is_query, &search_dirs,
        ) {
            Ok(params) => {
                if let Some(node) = self.graph.find_node_mut(node_id) {
                    let mut added_count = 0;
                    for (key, value) in params {
                        if !node.properties.iter().any(|p| p.key == key) {
                            if let Some(obj) = value.as_object() {
                                if let Some((param_type, param_val)) = obj.iter().next() {
                                    node.properties.push(PropertyItem {
                                        val_type: param_type.clone(),
                                        key: key.clone(),
                                        val: if let Some(s) = param_val.as_str() {
                                            s.to_string()
                                        } else {
                                            param_val.to_string()
                                        },
                                    });
                                    added_count += 1;
                                }
                            }
                        }
                    }

                    // 填充后自动按官方 AI 规范顺序排布
                    node.properties = crate::model::ai_def::reorder_parameter_items(
                        &actor_name, &action_query_name, is_query, &search_dirs, &node.properties,
                    );

                    self.mark_current_tab_modified();

                    if added_count > 0 {
                        self.toast = Some(Toast::success(
                            format!("{} {}", self.t("toast.params_autofilled"), added_count),
                        ));
                    } else {
                        self.toast = Some(Toast::success(self.t("toast.params_complete")));
                    }
                }
            }
            Err(e) => {
                let msg = format!("{}: {}", self.t("toast.autofill_failed"), e);
                self.toast = Some(Toast::error(&msg));
            }
        }
    }

    /// 规范重排指定节点的参数顺序
    pub fn reorder_node_parameters(&mut self, node_id: &str) {
        let (clean_actor, clean_name, is_q) = {
            if let Some(node) = self.graph.find_node(node_id) {
                let is_q = node.kind == NodeKind::Query || node.kind == NodeKind::Switch;
                if node.kind != NodeKind::Action && !is_q {
                    return;
                }
                let actor = node.properties.iter()
                    .find(|p| p.key == "ActorName")
                    .map(|p| p.val.clone())
                    .unwrap_or_else(|| node.subtitle.clone().unwrap_or_default());
                let name = node.properties.iter()
                    .find(|p| p.key == "Name")
                    .map(|p| p.val.clone())
                    .unwrap_or_else(|| node.title.clone());

                let clean_actor = actor.split('[').next().unwrap_or(&actor);
                let clean_actor = clean_actor.split('(').next().unwrap_or(clean_actor).trim().to_string();
                let clean_name = name.trim().to_string();
                (clean_actor, clean_name, is_q)
            } else {
                return;
            }
        };

        let search_dirs = self.build_game_ai_search_dirs();
        let toast_msg = self.t("modal.sort_success").to_string();

        if let Some(node) = self.graph.find_node_mut(node_id) {
            node.properties = crate::model::ai_def::reorder_parameter_items(
                &clean_actor, &clean_name, is_q, &search_dirs, &node.properties,
            );
            self.mark_current_tab_modified();
            self.toast = Some(Toast::info(toast_msg));
        }
    }

    /// 在节点编辑模态弹窗中自动填充参数
    pub fn autofill_modal_parameters(&mut self) {
        let (actor_name, action_name, is_query) = {
            if let Some(ref m) = self.active_node_editor_modal {
                let is_q = m.node_kind == NodeKind::Query || m.node_kind == NodeKind::Switch;
                let clean_actor = m.actor_name.split('[').next().unwrap_or(&m.actor_name);
                let clean_actor = clean_actor.split('(').next().unwrap_or(clean_actor).trim().to_string();
                let clean_name = m.action_name.trim().to_string();
                if clean_actor.is_empty() || clean_name.is_empty() {
                    self.toast = Some(Toast::error(self.t("toast.node_missing_info")));
                    return;
                }
                (clean_actor, clean_name, is_q)
            } else {
                return;
            }
        };

        let search_dirs = self.build_game_ai_search_dirs();

        match crate::model::ai_def::get_autofill_parameters(
            &actor_name, &action_name, is_query, &search_dirs,
        ) {
            Ok(params) => {
                if let Some(ref mut m) = self.active_node_editor_modal {
                    let mut added_count = 0;
                    for (key, value) in params {
                        if !m.properties.iter().any(|p| p.key == key) {
                            if let Some(obj) = value.as_object() {
                                if let Some((param_type, param_val)) = obj.iter().next() {
                                    m.properties.push(PropertyItem {
                                        val_type: param_type.clone(),
                                        key: key.clone(),
                                        val: if let Some(s) = param_val.as_str() {
                                            s.to_string()
                                        } else {
                                            param_val.to_string()
                                        },
                                    });
                                    added_count += 1;
                                }
                            }
                        }
                    }

                    // 自动按标准排序
                    m.properties = crate::model::ai_def::reorder_parameter_items(
                        &actor_name, &action_name, is_query, &search_dirs, &m.properties,
                    );

                    if added_count > 0 {
                        self.toast = Some(Toast::success(
                            format!("{} {}", self.t("toast.params_autofilled"), added_count),
                        ));
                    } else {
                        self.toast = Some(Toast::success(self.t("toast.params_complete")));
                    }
                }
            }
            Err(e) => {
                let msg = format!("{}: {}", self.t("toast.autofill_failed"), e);
                self.toast = Some(Toast::error(&msg));
            }
        }
    }

    /// 在节点编辑模态弹窗中对参数执行标准排序
    pub fn reorder_modal_parameters(&mut self) {
        let (clean_actor, clean_name, is_q) = {
            if let Some(ref m) = self.active_node_editor_modal {
                let is_q = m.node_kind == NodeKind::Query || m.node_kind == NodeKind::Switch;
                let clean_actor = m.actor_name.split('[').next().unwrap_or(&m.actor_name);
                let clean_actor = clean_actor.split('(').next().unwrap_or(clean_actor).trim().to_string();
                let clean_name = m.action_name.trim().to_string();
                (clean_actor, clean_name, is_q)
            } else {
                return;
            }
        };

        let search_dirs = self.build_game_ai_search_dirs();
        let toast_msg = self.t("modal.sort_success").to_string();

        if let Some(ref mut m) = self.active_node_editor_modal {
            m.properties = crate::model::ai_def::reorder_parameter_items(
                &clean_actor, &clean_name, is_q, &search_dirs, &m.properties,
            );
        }
        self.toast = Some(Toast::info(toast_msg));
    }

    /// 获取当前环境游戏 AI 的检测与加载状态
    pub fn get_game_ai_status(&self) -> crate::model::ai_def::AiDataStatus {
        let search_dirs = self.build_game_ai_search_dirs();
        crate::model::ai_def::get_ai_data_status(&search_dirs)
    }
}
