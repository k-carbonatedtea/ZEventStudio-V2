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

        let config = QuestScaffoldConfig {
            name: clean_name.to_string(),
            quest_type: modal.quest_type,
            orderer: modal.orderer.trim().to_string(),
            location: modal.location.trim().to_string(),
            display_title: modal.display_title.trim().to_string(),
            desc_text: modal.desc_text.trim().to_string(),
            finish_text: modal.finish_text.trim().to_string(),
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
    }
}
