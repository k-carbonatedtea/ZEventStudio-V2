pub mod byml_msbt_ops;
pub mod file_read_write;
pub mod file_tree_ops;
pub mod sarc_ops;
pub mod tab_ops;

use crate::ui::state::AppState;

/// 插件本地 API 核心路由扩展
impl AppState {
    /// 执行针对工程文件、代码或编辑器标签页的操作指令
    pub fn execute_plugin_file_action(
        &mut self,
        action: &str,
        payload: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        match action {
            "list_files" => self.api_list_files(payload),
            "read_file" => self.api_read_file(payload),
            "write_file" => self.api_write_file(payload),
            "delete_file" => self.api_delete_file(payload),
            "rename_file" => self.api_rename_file(payload),
            "create_directory" => self.api_create_directory(payload),
            "search_code" => self.api_search_code(payload),
            "query_byml" => self.api_query_byml(payload),
            "patch_byml" => self.api_patch_byml(payload),
            "read_msbt" => self.api_read_msbt(payload),
            "write_msbt" => self.api_write_msbt(payload),
            "pack_sarc" => self.api_pack_sarc(payload),
            "unpack_sarc" => self.api_unpack_sarc(payload),
            "open_file" => self.api_open_file(payload),
            "save_file" => self.api_save_file(payload),
            "close_file" => self.api_close_file(payload),
            "get_tabs" => self.api_get_tabs(),
            "reload_project" => {
                self.reload_project();
                Ok(serde_json::json!({ "status": "ok" }))
            }
            "open_project" => self.api_open_project(payload),
            _ => Err(format!("未知的操作指令: {}", action)),
        }
    }
}
