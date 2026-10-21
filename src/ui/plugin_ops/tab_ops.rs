use crate::ui::state::AppState;
use std::path::Path;

/// 针对编辑器标签页与工程项目加载的 API 实现
impl AppState {
    /// 打开目标文件至编辑器标签页
    pub fn api_open_file(&mut self, payload: &serde_json::Value) -> Result<serde_json::Value, String> {
        let raw_path = payload
            .get("path")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "缺少 path 参数".to_string())?;

        let resolved_path = self.resolve_target_path(raw_path);
        self.load_file_by_path(&resolved_path);

        Ok(serde_json::json!({
            "status": "ok",
            "active_tab_id": self.active_tab_id,
            "tab_kind": format!("{:?}", self.current_tab),
        }))
    }

    /// 保存当前或指定文档标签页
    pub fn api_save_file(&mut self, payload: &serde_json::Value) -> Result<serde_json::Value, String> {
        let success = if let Some(tab_id) = payload.get("tab_id").and_then(|value| value.as_str()) {
            self.save_document_tab(tab_id)
        } else {
            self.save_active_document()
        };

        Ok(serde_json::json!({ "status": "ok", "saved": success }))
    }

    /// 关闭指定标签页
    pub fn api_close_file(&mut self, payload: &serde_json::Value) -> Result<serde_json::Value, String> {
        if let Some(tab_id) = payload.get("tab_id").and_then(|value| value.as_str()) {
            self.close_document_tab(tab_id);
            return Ok(serde_json::json!({ "status": "ok", "closed": tab_id }));
        }

        if let Some(raw_path) = payload.get("path").and_then(|value| value.as_str()) {
            let resolved_path = self.resolve_target_path(raw_path);
            if let Some(tab) = self.open_tabs.iter().find(|tab_item| tab_item.file_path == resolved_path).cloned() {
                self.close_document_tab(&tab.id);
                return Ok(serde_json::json!({ "status": "ok", "closed": tab.id }));
            }
        }

        Err("请提供 tab_id 或 path".to_string())
    }

    /// 获取全部打开标签页状态
    pub fn api_get_tabs(&self) -> Result<serde_json::Value, String> {
        let tabs_list: Vec<serde_json::Value> = self
            .open_tabs
            .iter()
            .map(|tab_item| {
                serde_json::json!({
                    "id": tab_item.id,
                    "title": tab_item.title,
                    "file_path": tab_item.file_path,
                    "file_type": tab_item.file_type,
                    "tab_kind": format!("{:?}", tab_item.tab_kind),
                    "is_modified": tab_item.is_modified,
                    "is_active": self.active_tab_id.as_deref() == Some(&tab_item.id),
                })
            })
            .collect();

        Ok(serde_json::json!({
            "status": "ok",
            "active_tab_id": self.active_tab_id,
            "tabs_count": tabs_list.len(),
            "tabs": tabs_list,
        }))
    }

    /// 打开指定项目目录或 SARC 独立封包文件
    pub fn api_open_project(&mut self, payload: &serde_json::Value) -> Result<serde_json::Value, String> {
        let path = payload
            .get("path")
            .and_then(|value| value.as_str())
            .ok_or_else(|| "缺少 path 参数".to_string())?;

        let p = Path::new(path);
        if !p.exists() {
            return Err(format!("路径不存在: {}", path));
        }

        if p.is_dir() {
            self.open_project_directory(path);
        } else {
            self.open_pack_file(path);
        }

        Ok(self.get_project_summary_json())
    }
}
