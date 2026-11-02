use super::MapDocumentState;
use roead::byml::Byml;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};
use uking_library::map::hash_id;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ActorAsset {
    pub name: String,
    pub display_name: String,
    pub profile: String,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AssetCategory {
    pub profile: String,
    pub label: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AssetCatalog {
    pub entries: Vec<ActorAsset>,
    pub categories: Vec<AssetCategory>,
}

fn actor_names() -> &'static HashMap<String, String> {
    static NAMES: OnceLock<HashMap<String, String>> = OnceLock::new();
    NAMES.get_or_init(|| {
        serde_json::from_str(include_str!("../../../assets/actor-names-zh.json"))
            .unwrap_or_default()
    })
}

pub fn profile_label(profile: &str) -> &str {
    static LABELS: OnceLock<HashMap<String, String>> = OnceLock::new();
    LABELS
        .get_or_init(|| {
            include_str!("../../../assets/asset-profiles-zh.txt")
                .lines()
                .filter(|line| !line.trim_start().starts_with('#'))
                .filter_map(|line| line.split_once(" - "))
                .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
                .collect()
        })
        .get(profile)
        .map(String::as_str)
        .unwrap_or(profile)
}

impl AssetCatalog {
    /// ActorInfo contains the complete resource catalog, including actors which
    /// have not been placed in either half of the current map.
    pub fn from_actor_info(info: &HashMap<String, Byml>) -> Self {
        let mut entries = Vec::with_capacity(info.len());
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for (key, value) in info {
            let Ok(map) = value.as_map() else { continue };
            let text = |key: &str| {
                map.get(key)
                    .and_then(|value| value.as_string().ok())
                    .map(|value| value.as_str())
                    .unwrap_or("")
            };
            let name = if text("name").is_empty() {
                key.as_str()
            } else {
                text("name")
            };
            if name.trim().is_empty() {
                continue;
            }
            let profile = if text("profile").is_empty() {
                "其他"
            } else {
                text("profile")
            };
            let model = match (text("bfres"), text("mainModel")) {
                ("", "") => String::new(),
                (folder, "") => folder.to_string(),
                ("", model) => model.to_string(),
                (folder, model) => format!("{folder} / {model}"),
            };
            entries.push(ActorAsset {
                name: name.to_string(),
                display_name: actor_names()
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| name.to_string()),
                profile: profile.to_string(),
                model,
            });
        }
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        entries.dedup_by(|a, b| a.name == b.name);
        for actor in &entries {
            *counts.entry(actor.profile.clone()).or_default() += 1;
        }
        let categories = counts
            .into_iter()
            .map(|(profile, count)| AssetCategory {
                label: profile_label(&profile).to_string(),
                profile,
                count,
            })
            .collect();
        Self {
            entries,
            categories,
        }
    }

    pub fn find(&self, name: &str) -> Option<&ActorAsset> {
        self.entries
            .binary_search_by(|actor| actor.name.as_str().cmp(name))
            .ok()
            .map(|index| &self.entries[index])
    }

    pub fn filtered_indices(
        &self,
        query: &str,
        category: Option<&str>,
        favorites_only: bool,
        favorites: &HashSet<String>,
    ) -> Vec<usize> {
        let query = query.trim().to_lowercase();
        let words: Vec<_> = query.split_whitespace().collect();
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, actor)| {
                category.is_none_or(|profile| actor.profile == profile)
                    && (!favorites_only || favorites.contains(&actor.name))
                    && (words.is_empty() || {
                        let search = format!(
                            "{} {} {} {} {}",
                            actor.name,
                            actor.display_name,
                            actor.profile,
                            profile_label(&actor.profile),
                            actor.model
                        )
                        .to_lowercase();
                        words.iter().all(|word| search.contains(word))
                    })
            })
            .map(|(index, _)| index)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssetBrowserState {
    pub open: bool,
    pub height: f32,
    pub query: String,
    pub category: Option<String>,
    pub favorites_only: bool,
    pub favorites: Arc<HashSet<String>>,
    pub selected: Option<String>,
    pub grid: bool,
    pub pending_placement: Option<String>,
    pub resizing: Option<(f32, f32)>,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct AssetPreferences {
    favorites: Vec<String>,
}

fn preferences_path() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("Zevent")
        .join("asset-browser.json")
}

fn read_favorites(path: &Path) -> HashSet<String> {
    std::fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<AssetPreferences>(&bytes).ok())
        .unwrap_or_default()
        .favorites
        .into_iter()
        .collect()
}

fn write_favorites(path: &Path, favorites: &HashSet<String>) -> Result<(), String> {
    let mut names: Vec<_> = favorites.iter().cloned().collect();
    names.sort();
    let bytes = serde_json::to_vec_pretty(&AssetPreferences { favorites: names })
        .map_err(|error| error.to_string())?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("无法创建资产收藏目录：{error}"))?;
    }
    let temporary = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    std::fs::write(&temporary, bytes).map_err(|error| format!("无法保存资产收藏：{error}"))?;
    if let Err(error) = std::fs::rename(&temporary, path) {
        let _ = std::fs::remove_file(&temporary);
        return Err(format!("无法保存资产收藏：{error}"));
    }
    Ok(())
}

impl Default for AssetBrowserState {
    fn default() -> Self {
        Self {
            open: true,
            height: 250.,
            query: String::new(),
            category: None,
            favorites_only: false,
            favorites: Arc::new(read_favorites(&preferences_path())),
            selected: None,
            grid: true,
            pending_placement: None,
            resizing: None,
        }
    }
}

impl AssetBrowserState {
    pub fn toggle_favorite(&mut self, name: &str) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("资产名称为空".into());
        }
        let path = preferences_path();
        // Merge persisted names so another map tab's favorites are retained.
        let mut favorites = read_favorites(&path);
        if self.favorites.contains(name) {
            favorites.remove(name);
        } else {
            favorites.insert(name.to_string());
        }
        write_favorites(&path, &favorites)?;
        self.favorites = Arc::new(favorites);
        Ok(())
    }
}

impl MapDocumentState {
    /// Create a fresh placement, never a copy of another instance's links or
    /// runtime parameters. ActorInfo describes assets, not map instances.
    pub fn add_asset(&mut self, name: &str, position: [f32; 3]) -> Result<usize, String> {
        if self.loading {
            return Err("请等待地图资源加载完成".into());
        }
        if self.assets.catalog.find(name).is_none() {
            return Err("资产目录中没有此对象".into());
        }
        if !position.iter().all(|value| value.is_finite()) {
            return Err("放置坐标必须是有限数值".into());
        }
        let ids: HashSet<_> = self
            .file
            .actors()
            .iter()
            .chain(&self.assets.context_actors)
            .filter_map(hash_id)
            .collect();
        let mut id = ids.iter().max().copied().unwrap_or(0).wrapping_add(1);
        while id == 0 || ids.contains(&id) {
            id = id.wrapping_add(1);
        }
        let actor = Byml::Map(
            [
                ("UnitConfigName".into(), Byml::String(name.into())),
                ("HashId".into(), Byml::U32(id)),
                (
                    "Translate".into(),
                    Byml::Array(position.into_iter().map(Byml::Float).collect()),
                ),
                ("SRTHash".into(), Byml::I32(0)),
                ("!Parameters".into(), Byml::Map(Default::default())),
            ]
            .into_iter()
            .collect(),
        );
        let mut root = (*self.file.root).clone();
        let actors = root
            .as_mut_map()
            .map_err(|error| error.to_string())?
            .get_mut("Objs")
            .ok_or("地图缺少 Objs")?
            .as_mut_array()
            .map_err(|error| error.to_string())?;
        let index = actors.len();
        actors.push(actor);
        self.checkpoint();
        self.file.root = Arc::new(root);
        if !self.assets.actors.contains_key(name)
            && self.asset_loading.as_deref() != Some(name)
            && !self.asset_queue.iter().any(|queued| queued == name)
        {
            self.asset_queue.push_back(name.to_string());
        }
        self.asset_browser.pending_placement = None;
        // Placement is an editing action: make the newly selected instance
        // inspectable even if the user was looking at the clean game view.
        if self.view_mode == super::MapViewMode::Game {
            self.view_mode = super::MapViewMode::Context;
            self.show_links = true;
            self.selected_links_only = true;
        }
        self.show_models = true;
        self.select(Some(index));
        // Missing visuals are appended by the background asset loader. Keep
        // the current terrain and scene available while that work runs.
        Ok(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::map_editor::ActorVisual;
    use uking_library::map::MapFile;

    fn info() -> HashMap<String, Byml> {
        [
            (
                "Item_Apple",
                "{name: Item_Apple, profile: Item, bfres: Fruit, mainModel: Apple}",
            ),
            ("LinkTagAnd", "{name: LinkTagAnd, profile: LinkTag}"),
            ("Animal_Bear_A", "{name: Animal_Bear_A, profile: Prey}"),
        ]
        .into_iter()
        .map(|(name, text)| (name.into(), Byml::from_text(text).unwrap()))
        .collect()
    }

    #[test]
    fn catalog_uses_real_profiles_and_searches_chinese_identifiers_and_models() {
        let catalog = AssetCatalog::from_actor_info(&info());
        assert_eq!(
            catalog
                .entries
                .iter()
                .map(|actor| actor.name.as_str())
                .collect::<Vec<_>>(),
            ["Animal_Bear_A", "Item_Apple", "LinkTagAnd"]
        );
        assert_eq!(
            catalog.find("Animal_Bear_A").unwrap().display_name,
            "哈特诺熊"
        );
        assert_eq!(
            catalog
                .categories
                .iter()
                .find(|c| c.profile == "Item")
                .unwrap()
                .label,
            "物品"
        );
        let favorites = HashSet::from(["Item_Apple".to_string()]);
        assert_eq!(
            catalog.filtered_indices("APPLE fruit", None, false, &favorites),
            [1]
        );
        assert_eq!(
            catalog.filtered_indices("哈特诺", Some("Prey"), false, &favorites),
            [0]
        );
        assert!(catalog
            .filtered_indices("哈特诺", Some("Item"), false, &favorites)
            .is_empty());
        assert_eq!(catalog.filtered_indices("", None, true, &favorites), [1]);
        assert!(catalog.find("DoesNotExist").is_none());
    }

    #[test]
    fn favorites_roundtrip_and_replace_existing_file() {
        let directory =
            std::env::temp_dir().join(format!("zevent-assets-{}", uuid::Uuid::new_v4()));
        let path = directory.join("asset-browser.json");
        let names = HashSet::from(["Item_Apple".to_string(), "LinkTagAnd".to_string()]);
        write_favorites(&path, &names).unwrap();
        assert_eq!(read_favorites(&path), names);
        write_favorites(&path, &HashSet::new()).unwrap();
        assert!(read_favorites(&path).is_empty());
        std::fs::write(&path, b"invalid json").unwrap();
        assert!(read_favorites(&path).is_empty());
        std::fs::remove_dir_all(directory).unwrap();
    }

    fn document() -> MapDocumentState {
        let root = Byml::from_text("Objs: [{UnitConfigName: LinkTagAnd, HashId: !u 4294967295, LinksToObj: [{DestUnitHashId: !u 1, DefinitionName: BasicSig}]}]\nRails: [{Unknown: retained}]\nExtension: 17").unwrap();
        let file = MapFile::from_bytes(&root.to_binary(roead::Endian::Little)).unwrap();
        let mut doc = MapDocumentState::empty(file, "asset-test.mubin", "asset-test");
        doc.loading = false;
        Arc::make_mut(&mut doc.assets).catalog = Arc::new(AssetCatalog::from_actor_info(&info()));
        Arc::make_mut(&mut doc.assets)
            .context_actors
            .push(Byml::from_text("{UnitConfigName: Item_Apple, HashId: !u 1}").unwrap());
        Arc::make_mut(&mut doc.assets).actors.insert(
            "Item_Apple".into(),
            ActorVisual {
                meshes: vec![],
                resource: String::new(),
                profile: "Item".into(),
                has_model: false,
                diagnostic: None,
            },
        );
        doc
    }

    #[test]
    fn fresh_placement_preserves_other_data_reserves_paired_ids_and_supports_history() {
        let mut doc = document();
        let original = doc.file.root.clone();
        let paired = doc.assets.context_actors.clone();
        let index = doc.add_asset("Item_Apple", [10., 20., 30.]).unwrap();
        assert_eq!(index, 1);
        assert_eq!(doc.selected, Some(index));
        let actor = doc.file.actors()[index].as_map().unwrap();
        assert_eq!(actor["HashId"], Byml::U32(2));
        assert_eq!(actor["SRTHash"], Byml::I32(0));
        assert!(!actor.contains_key("LinksToObj"));
        assert!(actor["!Parameters"].as_map().unwrap().is_empty());
        assert_eq!(
            doc.file.root.as_map().unwrap()["Rails"],
            original.as_map().unwrap()["Rails"]
        );
        assert_eq!(
            doc.file.root.as_map().unwrap()["Extension"],
            original.as_map().unwrap()["Extension"]
        );
        assert_eq!(doc.assets.context_actors, paired);
        assert_eq!(
            MapFile::from_bytes(&doc.file.to_bytes()).unwrap().root,
            doc.file.root
        );
        let placed = doc.file.root.clone();
        assert_eq!(doc.undo.len(), 1);
        doc.undo();
        assert_eq!(doc.file.root, original);
        doc.redo();
        assert_eq!(doc.file.root, placed);
    }

    #[test]
    fn invalid_placement_does_not_change_map_or_history() {
        let mut doc = document();
        let original = doc.file.root.clone();
        assert!(doc.add_asset("NotAnActor", [0.; 3]).is_err());
        assert!(doc.add_asset("Item_Apple", [f32::NAN, 0., 0.]).is_err());
        doc.loading = true;
        assert!(doc.add_asset("Item_Apple", [0.; 3]).is_err());
        assert!(Arc::ptr_eq(&doc.file.root, &original));
        assert!(doc.undo.is_empty());
    }

    #[test]
    fn placement_returns_to_editing_context_without_resetting_loaded_scene() {
        let mut doc = document();
        doc.view_mode = super::super::MapViewMode::Game;
        doc.show_models = false;
        let meshes = doc.scene.meshes.clone();
        doc.add_asset("LinkTagAnd", [0.; 3]).unwrap();
        assert_eq!(doc.view_mode, super::super::MapViewMode::Context);
        assert!(doc.show_models);
        assert!(!doc.loading);
        assert!(Arc::ptr_eq(&meshes, &doc.scene.meshes));
        assert_eq!(doc.selected, Some(1));
        assert_eq!(doc.asset_queue, ["LinkTagAnd"]);
    }
}
