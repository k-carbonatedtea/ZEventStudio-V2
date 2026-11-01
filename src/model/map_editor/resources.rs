use crate::model::settings::GamePathSettings;
use roead::{byml::Byml, sarc::Sarc};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Debug, Clone)]
pub struct Resource {
    pub path: String,
    pub bytes: Vec<u8>,
}

pub struct Resources {
    roots: Vec<PathBuf>,
    packs: Vec<PathBuf>,
    cache: HashMap<PathBuf, Arc<Vec<u8>>>,
    pub actor_info: HashMap<String, Byml>,
}

impl Resources {
    pub fn new(
        source: &str,
        project: Option<&str>,
        settings: &GamePathSettings,
        big_endian: bool,
    ) -> Self {
        let mut roots = Vec::new();
        let mut push_root = |path: &str| {
            if path.is_empty() {
                return;
            }
            let root = PathBuf::from(path);
            for p in [
                root.clone(),
                root.join("romfs"),
                root.join("content"),
                root.join("aoc/0010"),
            ] {
                if p.is_dir() && !roots.contains(&p) {
                    roots.push(p);
                }
            }
        };
        if let Some(project) = project {
            push_root(project);
        }
        let outer = source
            .trim_start_matches("SARC:")
            .split("//")
            .next()
            .unwrap_or(source);
        for parent in Path::new(outer).ancestors().skip(1) {
            if parent.join("Actor").is_dir() || parent.join("Map").is_dir() {
                push_root(&parent.to_string_lossy());
                break;
            }
        }
        if big_endian {
            for p in [
                &settings.wiiu.dlc_dir,
                &settings.wiiu.update_dir,
                &settings.wiiu.game_dir,
            ] {
                push_root(p);
            }
        } else {
            for p in [&settings.switch.dlc_dir, &settings.switch.game_dir] {
                push_root(p);
            }
        }
        let mut packs = Vec::new();
        if source.starts_with("SARC:") {
            packs.push(PathBuf::from(outer));
        }
        let normalized = source.replace('\\', "/");
        let name = normalized
            .rsplit('/')
            .next()
            .unwrap_or("")
            .split('_')
            .next()
            .unwrap_or("");
        for root in &roots {
            for pack in [
                format!("Pack/{name}.pack"),
                "Pack/TitleBG.pack".into(),
                "Pack/Bootup.pack".into(),
                "Pack/AocMainField.pack".into(),
            ] {
                let path = root.join(pack);
                if path.is_file() && !packs.contains(&path) {
                    packs.push(path);
                }
            }
        }
        let mut this = Self {
            roots,
            packs,
            cache: HashMap::new(),
            actor_info: HashMap::new(),
        };
        if let Some(info) = this.read("Actor/ActorInfo.product.sbyml") {
            if let Ok(bytes) = uking_library::decompress(&info.bytes) {
                if let Ok(root) = Byml::from_binary(bytes) {
                    if let Some(Byml::Array(actors)) =
                        root.as_map().ok().and_then(|m| m.get("Actors"))
                    {
                        for actor in actors {
                            if let Some(name) = actor
                                .as_map()
                                .ok()
                                .and_then(|m| m.get("name"))
                                .and_then(|v| v.as_string().ok())
                            {
                                this.actor_info.insert(name.to_string(), actor.clone());
                            }
                        }
                    }
                }
            }
        }
        this
    }

    pub fn read(&mut self, relative: &str) -> Option<Resource> {
        // Overlay order is per content root, so a mod pack also overrides base loose files.
        let roots = self.roots.clone();
        let packs = self.packs.clone();
        for root in &roots {
            let path = root.join(relative);
            if let Ok(bytes) = std::fs::read(&path) {
                return Some(Resource {
                    path: path.to_string_lossy().into(),
                    bytes,
                });
            }
            for pack in packs.iter().filter(|p| p.starts_with(root)) {
                if let Some(found) = self.read_pack(pack, relative) {
                    return Some(found);
                }
            }
        }
        for pack in &packs {
            if let Some(found) = self.read_pack(pack, relative) {
                return Some(found);
            }
        }
        None
    }

    fn read_pack(&mut self, path: &Path, relative: &str) -> Option<Resource> {
        if !self.cache.contains_key(path) {
            let raw = std::fs::read(path).ok()?;
            let bytes = uking_library::decompress(&raw).ok()?;
            self.cache.insert(path.to_path_buf(), Arc::new(bytes));
        }
        let sarc = Sarc::new(self.cache.get(path)?.as_slice()).ok()?;
        let file = sarc.files().find(|f| f.name() == Some(relative))?;
        Some(Resource {
            path: format!("SARC:{}//{relative}", path.display()),
            bytes: file.data().to_vec(),
        })
    }

    pub fn terrain_file(
        &mut self,
        field: &str,
        tile: &uking_library::terrain::TerrainTile,
        kind: &str,
    ) -> Result<Vec<u8>, String> {
        let name = format!("{}.{}", tile.name, kind);
        let relative = format!("Terrain/A/{field}/{}.{}.sstera", tile.pack_name()?, kind);
        let pack = self
            .read(&relative)
            .ok_or_else(|| format!("找不到 {relative}"))?;
        let bytes = uking_library::decompress(&pack.bytes)?;
        let sarc = Sarc::new(&bytes).map_err(|e| e.to_string())?;
        let file = sarc
            .files()
            .find(|f| f.name() == Some(name.as_str()))
            .ok_or_else(|| format!("封包缺少 {name}"))?;
        uking_library::decompress(file.data())
    }

    pub fn model_targets(&mut self, actor: &str) -> Vec<(String, String)> {
        if let Some(info) = self.actor_info.get(actor).and_then(|v| v.as_map().ok()) {
            let bfres = info.get("bfres").and_then(|v| v.as_string().ok());
            let unit = info.get("mainModel").and_then(|v| v.as_string().ok());
            if let Some(folder) = bfres.filter(|f| !f.is_empty()) {
                return vec![(
                    folder.to_string(),
                    unit.map(ToString::to_string).unwrap_or_default(),
                )];
            }
        }
        let mut targets = Vec::new();
        if let Some(pack) = self.read(&format!("Actor/Pack/{actor}.sbactorpack")) {
            if let Ok(bytes) = uking_library::decompress(&pack.bytes) {
                if let Ok(sarc) = Sarc::new(&bytes) {
                    for file in sarc.files().filter(|f| {
                        f.name().is_some_and(|n| {
                            n.ends_with(".bmodellist") || n.ends_with(".sbmodellist")
                        })
                    }) {
                        if let Ok(data) = uking_library::decompress(file.data()) {
                            if let Ok(pio) = roead::aamp::ParameterIO::from_binary(data) {
                                if let Some(models) = pio.param_root.lists.get("ModelData") {
                                    for model in models.lists.0.values() {
                                        let folder = model
                                            .objects
                                            .get("Base")
                                            .and_then(|b| b.get("Folder"))
                                            .and_then(|p| p.as_str().ok())
                                            .unwrap_or("");
                                        if let Some(units) = model.lists.get("Unit") {
                                            for unit in units.objects.0.values() {
                                                if let Some(name) = unit
                                                    .get("UnitName")
                                                    .and_then(|p| p.as_str().ok())
                                                {
                                                    targets.push((
                                                        folder.to_string(),
                                                        name.to_string(),
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if targets.is_empty() {
            targets.push((actor.to_string(), actor.to_string()));
        }
        targets
    }

    pub fn actor_has_no_model(&mut self, actor: &str) -> bool {
        let Some(pack) = self.read(&format!("Actor/Pack/{actor}.sbactorpack")) else {
            return false;
        };
        let Ok(bytes) = uking_library::decompress(&pack.bytes) else {
            return false;
        };
        let Ok(sarc) = Sarc::new(&bytes) else {
            return false;
        };
        sarc.files()
            .any(|f| f.name().is_some_and(|n| n.ends_with("/None.bmodellist")))
    }

    pub fn actor_umii(&mut self, actor: &str) -> Option<crate::model::umii::UmiiData> {
        let pack = self.read(&format!("Actor/Pack/{actor}.sbactorpack"))?;
        let bytes = uking_library::decompress(&pack.bytes).ok()?;
        let sarc = Sarc::new(&bytes).ok()?;
        for file in sarc.files() {
            if file.name().is_some_and(|n| n.ends_with(".bumii")) {
                return crate::model::umii::UmiiData::from_binary(
                    &uking_library::decompress(file.data()).ok()?,
                )
                .ok();
            }
        }
        None
    }

    pub fn model_directories(&self) -> Vec<PathBuf> {
        self.roots.iter().map(|p| p.join("Model")).collect()
    }

    pub fn model_file(&mut self, folder: &str, unit: &str) -> Option<Resource> {
        for part in -1..256 {
            let suffix = if part < 0 {
                String::new()
            } else {
                format!("-{part:02}")
            };
            if let Some(resource) = self.read(&format!("Model/{folder}{suffix}.sbfres")) {
                // Split archives may put the requested mainModel in a later part.
                if !unit.is_empty() {
                    if let Ok(data) = uking_library::decompress(&resource.bytes) {
                        if let Ok(bfres) = rbfres_library::BfresFile::from_bytes(&data) {
                            if !bfres.models.iter().any(|(name, _)| name == unit) {
                                continue;
                            }
                        }
                    }
                }
                return Some(resource);
            } else if part > 0 {
                break;
            }
        }
        self.read(&format!("Model/{folder}.bfres"))
    }
}
