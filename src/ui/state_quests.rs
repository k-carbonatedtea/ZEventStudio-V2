use std::path::Path;
use crate::model::quest::{
    execute_full_quest_creation, scaffold_quest_graph_nodes, scan_project_quests,
    QuestDefinition, QuestScaffoldConfig,
};
use crate::ui::components::Toast;
use crate::ui::state::{AppState, QuestCreatorModalState};

/// 任务管理器在 AppState 中的操作与调度扩展
impl AppState {
    /// 扫描并刷新当前项目中的所有 BotW 任务
    pub fn refresh_quests_list(&mut self) {
        if let Some(ref path_str) = self.project_path {
            let path = Path::new(path_str);
            let quests = scan_project_quests(path);
            self.quests_list = quests;
        } else {
            self.quests_list.clear();
        }
    }

    /// 打开新建任务向导模态框
    pub fn open_quest_creator(&mut self) {
        self.clear_active_text_inputs();
        self.quest_creator_modal = Some(QuestCreatorModalState::default());
    }

    /// 关闭新建任务向导模态框
    pub fn close_quest_creator(&mut self) {
        self.quest_creator_modal = None;
    }

    /// 从当前 2D 地图标点一键拾取坐标至任务创建向导
    pub fn fill_quest_creator_from_map_pin(&mut self) -> bool {
        if let Some((wx, wz)) = self.quest_map_user_pin {
            if let Some(ref mut m) = self.quest_creator_modal {
                m.coord_x = format!("{:.1}", wx);
                m.coord_z = format!("{:.1}", wz);
                m.enable_indicator = true;
                return true;
            }
        }
        false
    }

    /// 执行任务创建并应用到项目和当前打开的流程图画布
    pub fn apply_quest_creation(&mut self) -> Result<Vec<String>, String> {
        let modal = self.quest_creator_modal.clone()
            .ok_or_else(|| "向导状态为空".to_string())?;

        let clean_name = modal.name.trim();
        if clean_name.is_empty() {
            return Err("任务名称不能为空".to_string());
        }

        let project_str = self.project_path.clone()
            .ok_or_else(|| "未打开任何项目，请先打开项目文件夹".to_string())?;
        let project_path = Path::new(&project_str);

        let ready_flag = if modal.ready_flag.trim().is_empty() {
            format!("{}_Ready", clean_name)
        } else {
            modal.ready_flag.trim().to_string()
        };
        let activated_flag = if modal.activated_flag.trim().is_empty() {
            format!("{}_Activated", clean_name)
        } else {
            modal.activated_flag.trim().to_string()
        };
        let finish_flag = if modal.finish_flag.trim().is_empty() {
            format!("{}_Finish", clean_name)
        } else {
            modal.finish_flag.trim().to_string()
        };

        let coord_x = modal.coord_x.trim().parse::<f32>().unwrap_or(0.0);
        let coord_y = modal.coord_y.trim().parse::<f32>().unwrap_or(0.0);
        let coord_z = modal.coord_z.trim().parse::<f32>().unwrap_or(0.0);

        let config = QuestScaffoldConfig {
            name: clean_name.to_string(),
            quest_type: modal.quest_type,
            orderer: modal.orderer.trim().to_string(),
            location: modal.location.trim().to_string(),
            display_title: modal.display_title.trim().to_string(),
            desc_text: modal.desc_text.trim().to_string(),
            finish_text: modal.finish_text.trim().to_string(),
            ready_flag,
            activated_flag,
            finish_flag,
            dependency_flag: modal.dependency_flag.trim().to_string(),
            extra_flags: modal.extra_flags.iter().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
            enable_indicator: modal.enable_indicator,
            indicator_actor: if modal.indicator_actor.trim().is_empty() {
                modal.orderer.trim().to_string()
            } else {
                modal.indicator_actor.trim().to_string()
            },
            indicator_pos: (coord_x, coord_y, coord_z),
            indicator_off_flag: modal.indicator_off_flag.trim().to_string(),
            indicator_has_far: modal.indicator_has_far,
            not_post_note: modal.not_post_note,
            inject_titlebg: modal.inject_titlebg,
            inject_bootup_flags: modal.inject_bootup_flags,
            inject_ql_msbt: modal.inject_ql_msbt,
            scaffold_graph_nodes: modal.scaffold_graph_nodes,
            inject_npc_choice: false,
        };

        // 1. 自动化注入文件 (TitleBG, Bootup flags, MSBT)
        let mut logs = execute_full_quest_creation(project_path, &config)?;

        // 2. 如果勾选了自动构建流程图拓扑，且当前处于流程图画布中
        if config.scaffold_graph_nodes {
            let created_nodes = scaffold_quest_graph_nodes(&mut self.graph, &config);
            self.mark_current_tab_modified();
            self.fit_view();
            logs.push(format!("已在当前画布注入 {} 个任务事件节点拓扑", created_nodes.len()));
        }

        // 3. 刷新任务列表
        self.refresh_quests_list();
        self.quests_selected_id = Some(config.name.clone());

        // 4. 关闭模态框并给出 Toast 提示
        self.quest_creator_modal = None;
        self.toast = Some(Toast::success(format!("任务 [{}] 创建成功！", config.name)));

        Ok(logs)
    }

    /// 获取经过类型和搜索关键词过滤后的任务列表
    pub fn get_filtered_quests(&self) -> Vec<&QuestDefinition> {
        let query = self.quests_search_query.trim().to_lowercase();
        self.quests_list
            .iter()
            .filter(|q| {
                if let Some(target_type) = self.quests_filter_type {
                    if q.quest_type != target_type {
                        return false;
                    }
                }
                if query.is_empty() {
                    return true;
                }
                q.name.to_lowercase().contains(&query)
                    || q.display_title.as_deref().unwrap_or("").to_lowercase().contains(&query)
                    || q.orderer.to_lowercase().contains(&query)
                    || q.location.to_lowercase().contains(&query)
            })
            .collect()
    }

    /// 切换选中任务详情展开
    pub fn toggle_quest_selection(&mut self, quest_name: &str) {
        if self.quests_selected_id.as_deref() == Some(quest_name) {
            self.quests_selected_id = None;
        } else {
            self.quests_selected_id = Some(quest_name.to_string());
        }
        self.quests_list_state.remeasure();
    }

    /// 获取当前选中的任务定义
    pub fn get_selected_quest(&self) -> Option<&QuestDefinition> {
        self.quests_selected_id
            .as_ref()
            .and_then(|id| self.quests_list.iter().find(|q| &q.name == id))
    }

    /// 将 2D 地图聚焦居中到选中的任务标点群
    pub fn focus_selected_quest_on_map(&mut self) {
        if let Some(quest) = self.get_selected_quest() {
            let mut sum_x = 0.0f32;
            let mut sum_z = 0.0f32;
            let mut count = 0usize;

            for step in &quest.steps {
                for ind in &step.indicators {
                    sum_x += ind.position.0;
                    sum_z += ind.position.2;
                    count += 1;
                }
            }

            if count > 0 {
                let center_x = sum_x / (count as f32);
                let center_z = sum_z / (count as f32);
                self.quest_map_zoom = 0.18;
                self.quest_map_pan = (600.0 - center_x * self.quest_map_zoom, 400.0 - center_z * self.quest_map_zoom);
            }
        }
    }

    /// 将 2D 地图精确定位聚焦至指定标点世界坐标 (X, Y, Z)
    pub fn focus_pin_on_map(&mut self, pos: (f32, f32, f32)) {
        self.quest_map_zoom = 0.25;
        self.quest_map_pan = (600.0 - pos.0 * self.quest_map_zoom, 400.0 - pos.2 * self.quest_map_zoom);
    }

    /// 将 2D 地图聚焦至指定地理地标世界坐标 (X, Z)（保留用于外部搜索或指令主动导航）
    #[allow(dead_code)]
    pub fn focus_landmark_on_map(&mut self, pos: (f32, f32)) {
        self.quest_map_zoom = 0.22;
        self.quest_map_pan = (600.0 - pos.0 * self.quest_map_zoom, 400.0 - pos.1 * self.quest_map_zoom);
    }

    /// 重置 2D 地图至海拉鲁全景居中
    pub fn reset_quest_map_view(&mut self) {
        self.quest_map_pan = (600.0, 400.0);
        self.quest_map_zoom = 0.085;
        self.quest_map_selected_pin = None;
    }

    /// 缩放 2D 地图
    pub fn zoom_quest_map(&mut self, factor: f32) {
        self.quest_map_zoom = (self.quest_map_zoom * factor).clamp(0.02, 1.8);
    }

    /// 触发从已打开项目或游戏本体 Dump 提取海拉鲁地图瓦片
    pub fn trigger_extract_map_tiles(&mut self, model: &gpui::Entity<AppState>, cx: &mut gpui::App) {
        if self.quest_map_is_extracting_tiles {
            return;
        }

        let proj_path = self.project_path.clone();
        let game_paths = self.game_paths.clone();

        // 1. 解析瓦片来源目录 (优先项目内部，若无则回退游戏本体 Dump)
        let (src_dir, is_project) = match crate::ui::views::quest_map::tiles_layer::resolve_maptex_source(
            proj_path.as_deref(),
            Some(&game_paths),
        ) {
            Some(pair) => pair,
            None => {
                self.toast = Some(Toast::error(self.t("quests.maptex_not_found")));
                return;
            }
        };

        // 2. 目标输出缓存目录 (项目优先输出至项目内部 .cache/map_tiles，实现隔离)
        let out_dir = if let Some(ref p) = proj_path {
            crate::ui::views::quest_map::tiles_layer::get_project_tiles_cache_dir(p)
        } else {
            crate::ui::views::quest_map::tiles_layer::get_global_map_tiles_cache_dir()
        };

        self.quest_map_is_extracting_tiles = true;
        if is_project {
            self.toast = Some(Toast::info(self.t("quests.extracting_project_tiles")));
        } else {
            self.toast = Some(Toast::info(self.t("quests.extracting_game_tiles")));
        }

        let model = model.clone();
        cx.spawn(async move |cx| {
            let src = src_dir.clone();
            let out = out_dir.clone();

            let res = cx
                .background_executor()
                .spawn(async move {
                    crate::ui::views::quest_map::tiles_layer::execute_tile_extraction(&src, &out)
                })
                .await;

            cx.update(|cx| {
                model.update(cx, |st, cx| {
                    st.quest_map_is_extracting_tiles = false;
                    match res {
                        Ok(count) => {
                            let msg = if is_project {
                                format!(
                                    "{} ({} {})",
                                    st.t("quests.extract_project_success"),
                                    count,
                                    st.t("quests.tiles_unit")
                                )
                            } else {
                                format!(
                                    "{} ({} {})",
                                    st.t("quests.extract_game_success"),
                                    count,
                                    st.t("quests.tiles_unit")
                                )
                            };
                            st.toast = Some(Toast::success(msg));
                        }
                        Err(e) => {
                            st.toast = Some(Toast::error(format!(
                                "{}: {}",
                                st.t("quests.extract_failed"),
                                e
                            )));
                        }
                    }
                    cx.notify();
                });
            });
        })
        .detach();
    }

    /// 异步触发海拉鲁标点提取（后台多线程执行，主 UI 保持 60+ FPS，并实时汇报提取进度）
    pub fn trigger_async_extract_hyrule_markers(
        &mut self,
        model: &gpui::Entity<AppState>,
        cx: &mut gpui::App,
    ) {
        if self.quest_map_is_extracting_markers || self.quest_map_markers_cache.is_some() {
            return;
        }

        self.quest_map_is_extracting_markers = true;
        self.quest_map_extracting_file_label = self.t("quests.loading_init_status").to_string();

        let proj_path = self.project_path.clone();
        let game_paths = self.game_paths.clone();
        let model = model.clone();

        cx.spawn(async move |cx| {
            let (tx, rx) = smol::channel::unbounded::<String>();

            // 1. 实时监听进度消息并通知主 UI 刷新
            let rx_model = model.clone();
            let progress_task = cx.spawn(async move |cx| {
                while let Ok(label) = rx.recv().await {
                    cx.update(|cx| {
                        rx_model.update(cx, |st, cx| {
                            st.quest_map_extracting_file_label = label;
                            cx.notify();
                        });
                    });
                }
            });

            // 2. 在后台工作线程池执行 CPU 密集型解压与 I/O 扫描
            let markers = cx
                .background_executor()
                .spawn(async move {
                    let tx_progress = tx.clone();
                    let res = crate::ui::views::quest_map::markers_extractor::extract_hyrule_markers_with_progress(
                        proj_path.as_deref(),
                        Some(&game_paths),
                        move |status| {
                            let _ = tx_progress.try_send(status.to_string());
                        },
                    );
                    drop(tx); // 显式释放 tx，使 rx 能够感知结束
                    res
                })
                .await;

            let _ = progress_task.await;

            // 3. 回到主线程提交标点缓存并关闭 loading
            cx.update(|cx| {
                model.update(cx, |st, cx| {
                    st.quest_map_markers_cache = Some(markers);
                    st.quest_map_is_extracting_markers = false;
                    st.quest_map_extracting_file_label.clear();
                    cx.notify();
                });
            });
        })
        .detach();
    }
}

