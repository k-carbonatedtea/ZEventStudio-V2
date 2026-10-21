use super::manifest::{PluginInstance, PluginManifest, PluginStatus};
use std::fs;
use std::path::{Path, PathBuf};

/// 插件系统管理器，负责插件目录扫描、清单解析与生命周期追踪
#[derive(Debug, Clone)]
pub struct PluginManager {
    /// 插件存放的主目录路径 (默认为工作目录下的 plugins 文件夹)
    pub plugins_dir: PathBuf,
    /// 当前已扫描并加载的插件列表
    pub plugins: Vec<PluginInstance>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    /// 初始化插件管理器，探测并确定默认插件存储目录
    pub fn new() -> Self {
        let plugins_dir = Self::resolve_plugins_dir();
        let mut manager = Self {
            plugins_dir,
            plugins: Vec::new(),
        };
        manager.rescan();
        manager
    }

    /// 确定插件目录物理路径 (优先当前工作目录 ./plugins，次选可执行文件同级 plugins)
    fn resolve_plugins_dir() -> PathBuf {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let local_plugins = current_dir.join("plugins");
        if local_plugins.exists() {
            return local_plugins;
        }

        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(parent) = exe_path.parent() {
                let exe_plugins = parent.join("plugins");
                if exe_plugins.exists() {
                    return exe_plugins;
                }
            }
        }

        local_plugins
    }

    /// 确保插件物理目录存在，若不存在则自动创建
    pub fn ensure_plugins_dir(&self) -> Result<(), String> {
        if !self.plugins_dir.exists() {
            fs::create_dir_all(&self.plugins_dir)
                .map_err(|e| format!("无法创建插件目录 {:?}: {}", self.plugins_dir, e))?;
        }
        Ok(())
    }

    /// 确保默认内置插件 (如 zevent-mcp-bridge) 存在，若缺失则自动部署
    pub fn ensure_default_plugins(&self) {
        let mcp_dir = self.plugins_dir.join("zevent-mcp-bridge");
        let manifest_file = mcp_dir.join("plugin.json");
        let script_file = mcp_dir.join("mcp_server.py");

        if !manifest_file.exists() || !script_file.exists() {
            let _ = fs::create_dir_all(&mcp_dir);
            if !manifest_file.exists() {
                let default_manifest = include_str!("../../../plugins/zevent-mcp-bridge/plugin.json");
                let _ = fs::write(&manifest_file, default_manifest);
            }
            if !script_file.exists() {
                let default_script = include_str!("../../../plugins/zevent-mcp-bridge/mcp_server.py");
                let _ = fs::write(&script_file, default_script);
            }
        }
    }

    /// 重新扫描插件主目录并重新加载所有合规的插件
    pub fn rescan(&mut self) {
        let _ = self.ensure_plugins_dir();
        self.ensure_default_plugins();
        let mut loaded_plugins = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.plugins_dir) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    let manifest_file = entry_path.join("plugin.json");
                    if manifest_file.exists() && manifest_file.is_file() {
                        if let Some(instance) = Self::load_single_plugin(&entry_path, &manifest_file) {
                            loaded_plugins.push(instance);
                        }
                    }
                }
            }
        }

        // 按照插件名称自然排序
        loaded_plugins.sort_by(|a, b| a.manifest.name.cmp(&b.manifest.name));
        self.plugins = loaded_plugins;
    }

    /// 解析指定目录下的单个插件清单文件
    fn load_single_plugin(plugin_dir: &Path, manifest_file: &Path) -> Option<PluginInstance> {
        let content = fs::read_to_string(manifest_file).ok()?;
        match serde_json::from_str::<PluginManifest>(&content) {
            Ok(manifest) => {
                let instance = PluginInstance::new(manifest, plugin_dir.to_path_buf());
                Some(instance)
            }
            Err(e) => {
                let fallback_id = plugin_dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                let mut fallback_manifest = PluginManifest {
                    id: fallback_id.clone(),
                    name: fallback_id,
                    version: "0.0.0".to_string(),
                    description: format!("清单解析失败: {}", e),
                    author: "Unknown".to_string(),
                    icon: "⚠️".to_string(),
                    kind: super::manifest::PluginKind::General,
                    entry: None,
                    enabled: false,
                    homepage: None,
                    server: None,
                    contributes: Default::default(),
                };
                fallback_manifest.enabled = false;
                let mut inst = PluginInstance::new(fallback_manifest, plugin_dir.to_path_buf());
                inst.status = PluginStatus::Error(e.to_string());
                inst.error_message = Some(e.to_string());
                Some(inst)
            }
        }
    }

    /// 切换指定插件的启用/禁用状态
    pub fn toggle_plugin(&mut self, plugin_id: &str) -> bool {
        if let Some(plugin) = self.plugins.iter_mut().find(|p| p.id() == plugin_id) {
            let next_enabled = !plugin.manifest.enabled;
            plugin.manifest.enabled = next_enabled;
            plugin.status = if next_enabled {
                PluginStatus::Loaded
            } else {
                PluginStatus::Disabled
            };

            // 尝试同步回写至 plugin.json
            let manifest_path = plugin.dir_path.join("plugin.json");
            if let Ok(content) = serde_json::to_string_pretty(&plugin.manifest) {
                let _ = fs::write(manifest_path, content);
            }
            return true;
        }
        false
    }

    /// 根据 ID 查找指定插件
    pub fn find_plugin(&self, plugin_id: &str) -> Option<&PluginInstance> {
        self.plugins.iter().find(|p| p.id() == plugin_id)
    }

    /// 根据 ID 查找指定插件可变引用
    #[allow(dead_code)]
    pub fn find_plugin_mut(&mut self, plugin_id: &str) -> Option<&mut PluginInstance> {
        self.plugins.iter_mut().find(|p| p.id() == plugin_id)
    }

    /// 获取当前处于已启用状态的插件数量
    pub fn active_count(&self) -> usize {
        self.plugins.iter().filter(|p| p.is_enabled()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_loads_mcp_bridge() {
        let manager = PluginManager::new();
        assert!(!manager.plugins.is_empty(), "Plugins list should not be empty");
        let mcp = manager.find_plugin("zevent-mcp-bridge");
        assert!(mcp.is_some(), "zevent-mcp-bridge plugin must be found");
        let mcp_inst = mcp.unwrap();
        assert_eq!(mcp_inst.manifest.kind, crate::model::plugin::PluginKind::Mcp);
        assert!(mcp_inst.is_enabled());
        assert!(!mcp_inst.manifest.contributes.tools.is_empty());
    }

    #[test]
    fn test_plugin_toggle() {
        let mut manager = PluginManager::new();
        if manager.find_plugin("zevent-mcp-bridge").is_some() {
            let initial_status = manager.find_plugin("zevent-mcp-bridge").unwrap().manifest.enabled;
            manager.toggle_plugin("zevent-mcp-bridge");
            assert_ne!(
                manager.find_plugin("zevent-mcp-bridge").unwrap().manifest.enabled,
                initial_status
            );
            // 切换还原
            manager.toggle_plugin("zevent-mcp-bridge");
            assert_eq!(
                manager.find_plugin("zevent-mcp-bridge").unwrap().manifest.enabled,
                initial_status
            );
        }
    }
}
