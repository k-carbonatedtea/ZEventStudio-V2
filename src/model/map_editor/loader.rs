use super::{geometry, resources::Resources, ActorVisual, MapAssets, MapDocumentState};
use crate::model::{
    model_preview::{ModelDocumentState, PreviewMeshItem, RgbaTexture},
    settings::GamePathSettings,
};
use rbfres_library::{BfresFile, ExtractedMesh};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Instant,
};
use uking_library::{
    map::actor_name,
    terrain::{section_center, HeightField, TerrainTable},
};

pub(super) fn push_mesh(scene: &mut ModelDocumentState, mesh: ExtractedMesh) -> usize {
    let index = scene.mesh_items.len();
    Arc::make_mut(&mut scene.mesh_items).push(PreviewMeshItem {
        name: mesh.name.clone(),
        material_name: mesh.material_name.clone().unwrap_or_default(),
        texture_name: mesh.texture_name.clone(),
        vertex_count: mesh.positions.len(),
        triangle_count: mesh.indices.len() / 3,
        visible: true,
        min: [
            mesh.bounding_box.min().x,
            mesh.bounding_box.min().y,
            mesh.bounding_box.min().z,
        ],
        max: [
            mesh.bounding_box.max().x,
            mesh.bounding_box.max().y,
            mesh.bounding_box.max().z,
        ],
    });
    Arc::make_mut(&mut scene.meshes).push(mesh);
    index
}

pub fn load_scene(
    mut doc: MapDocumentState,
    settings: GamePathSettings,
    project: Option<String>,
) -> Result<MapDocumentState, String> {
    let started = Instant::now();
    let profile = std::env::var_os("ZEVENT_PROFILE_LOAD").is_some();
    let mut resources = Resources::new(
        &doc.scene.file_path,
        project.as_deref(),
        &settings,
        doc.file.big_endian,
    );
    let mut assets = MapAssets::default();
    load_context(&doc, &mut resources, &mut assets);
    let names: std::collections::BTreeSet<String> = doc
        .file
        .actors()
        .iter()
        .chain(&assets.context_actors)
        .map(|a| actor_name(a).to_string())
        .collect();
    assets.catalog = Arc::new(super::catalog::AssetCatalog::from_actor_info(
        &resources.actor_info,
    ));
    load_actor_models(
        &mut doc,
        &mut resources,
        &mut assets,
        names,
        project.as_deref(),
    );
    if profile {
        eprintln!(
            "[MAP LOAD] actors/models/textures: {:.3}s",
            started.elapsed().as_secs_f64()
        );
    }
    let mut materials = super::terrain_material::TerrainMaterials::new(project.as_deref());
    load_actor_terrain_materials(&mut doc, &mut resources, &mut assets, &mut materials);
    if profile {
        eprintln!(
            "[MAP LOAD] shared terrain materials: {:.3}s",
            started.elapsed().as_secs_f64()
        );
    }
    if let Err(error) = load_dungeon_background(&mut doc, &mut resources, &mut assets) {
        assets.warnings.push(error);
    }
    if let Err(error) = load_terrain(&mut doc, &mut resources, &mut assets, &mut materials) {
        assets.warnings.push(error);
    }
    doc.assets = Arc::new(assets);
    doc.loading = false;
    doc.rebuild_instances();
    if doc.selected_ref().is_some() {
        doc.focus_selected();
    } else {
        doc.scene.fit_camera();
    }
    doc.scene.camera.distance *= 0.65;
    doc.scene.camera.pitch = 0.7;
    if profile {
        eprintln!(
            "[MAP LOAD] complete: {:.3}s, decoded textures: {}",
            started.elapsed().as_secs_f64(),
            doc.scene.textures.len()
        );
    }
    Ok(doc)
}

/// Load one catalog actor into the existing scene. Map terrain and companion
/// objects are retained, and repeated placements reuse the same visual.
pub fn load_actor_asset(
    mut doc: MapDocumentState,
    settings: GamePathSettings,
    project: Option<String>,
    name: &str,
) -> Result<MapDocumentState, String> {
    if name.is_empty() {
        return Err("资产名称为空".into());
    }
    if doc.assets.actors.contains_key(name) {
        return Ok(doc);
    }
    let mut resources = Resources::new(
        &doc.scene.file_path,
        project.as_deref(),
        &settings,
        doc.file.big_endian,
    );
    let mut assets = (*doc.assets).clone();
    load_actor_models(
        &mut doc,
        &mut resources,
        &mut assets,
        [name.to_string()],
        project.as_deref(),
    );
    let mut materials = super::terrain_material::TerrainMaterials::new(project.as_deref());
    load_actor_terrain_materials(&mut doc, &mut resources, &mut assets, &mut materials);
    doc.assets = Arc::new(assets);
    doc.rebuild_instances();
    Ok(doc)
}

/// Build an isolated asset preview through the same actor/model resolver as
/// map placement. `source` only chooses the project/game resource overlays.
pub fn load_asset_preview(
    source: &str,
    big_endian: bool,
    settings: GamePathSettings,
    project: Option<String>,
    name: &str,
) -> Result<ModelDocumentState, String> {
    use roead::byml::Byml;
    let mut actor = roead::byml::Map::default();
    actor.insert("UnitConfigName".into(), Byml::String(name.into()));
    actor.insert("HashId".into(), Byml::U32(1));
    let mut root = roead::byml::Map::default();
    root.insert("Objs".into(), Byml::Array(vec![Byml::Map(actor)]));
    let endian = if big_endian {
        roead::Endian::Big
    } else {
        roead::Endian::Little
    };
    let file = uking_library::map::MapFile::from_bytes(&Byml::Map(root).to_binary(endian))?;
    let mut doc = MapDocumentState::empty(file, source, name);
    doc.loading = false;
    doc.show_tags = true;
    doc.selected = Some(0);
    let mut doc = load_actor_asset(doc, settings, project, name)?;
    let visual = doc.assets.actors.get(name).ok_or("资产预览未生成")?;
    if let Some(error) = &visual.diagnostic {
        return Err(error.clone());
    }
    doc.scene.file_path = format!("asset-preview:{source}#{name}");
    doc.scene.model_name = name.into();
    // Preview instances should not carry the map editor's selection tint.
    if let Some(instances) = &mut doc.scene.instances {
        for instance in Arc::make_mut(instances) {
            instance.selected = false;
        }
    }
    doc.scene.fit_camera();
    doc.scene.camera.distance *= 1.1;
    doc.scene.camera.pitch = 0.3;
    doc.scene.show_stats = false;
    Ok(doc.scene)
}

fn ensure_marker(scene: &mut ModelDocumentState, mesh: ExtractedMesh, texture: &str) -> usize {
    if let Some(index) = scene.meshes.iter().position(|existing| {
        existing.name == mesh.name && existing.texture_name.as_deref() == Some(texture)
    }) {
        return index;
    }
    let index = push_mesh(scene, mesh);
    Arc::make_mut(&mut scene.meshes)[index].texture_name = Some(texture.into());
    index
}

fn load_actor_models(
    doc: &mut MapDocumentState,
    resources: &mut Resources,
    assets: &mut MapAssets,
    names: impl IntoIterator<Item = String>,
    project: Option<&str>,
) {
    let mut cache: HashMap<(String, String), Vec<usize>> = HashMap::new();
    let mut existing_model_meshes: HashMap<String, Vec<usize>> = assets
        .actors
        .values()
        .filter(|visual| visual.has_model && !visual.meshes.is_empty())
        .map(|visual| (visual.resource.clone(), visual.meshes.clone()))
        .collect();
    let mut loaded_textures: HashSet<String> = doc
        .scene
        .textures
        .keys()
        .filter_map(|name| {
            name.rsplit_once("::")
                .map(|(namespace, _)| namespace.to_string())
        })
        .collect();
    let marker_index = ensure_marker(
        &mut doc.scene,
        geometry::rito_cube("Editor marker", 0.625),
        "__editor_marker_generic",
    );
    let area_marker_index = ensure_marker(
        &mut doc.scene,
        geometry::cube("Editor area"),
        "__editor_marker_area",
    );
    assets.round_markers = [
        ensure_marker(
            &mut doc.scene,
            geometry::round_marker(true),
            "__editor_marker_area",
        ),
        ensure_marker(
            &mut doc.scene,
            geometry::round_marker(false),
            "__editor_marker_area",
        ),
    ];
    let marker_texture = "__editor_marker_generic".to_string();
    Arc::make_mut(&mut doc.scene.textures).insert(
        marker_texture.clone(),
        editor_texture(include_bytes!(
            "../../../assets/editor-markers/TexturedCube.png"
        )),
    );
    Arc::make_mut(&mut doc.scene.meshes)[marker_index].texture_name = Some(marker_texture);
    Arc::make_mut(&mut doc.scene.textures).insert(
        "__editor_marker_area".into(),
        Arc::new(RgbaTexture {
            width: 1,
            height: 1,
            rgba: Arc::new(vec![92, 164, 220, 36]),
        }),
    );
    Arc::make_mut(&mut doc.scene.meshes)[area_marker_index].texture_name =
        Some("__editor_marker_area".into());
    let mut tag_meshes = HashMap::new();
    for (name, bytes) in [
        (
            "LinkTagAnd",
            include_bytes!("../../../assets/editor-markers/LinkTagAnd.png") as &[u8],
        ),
        (
            "LinkTagNAnd",
            include_bytes!("../../../assets/editor-markers/LinkTagNAnd.png") as &[u8],
        ),
        (
            "LinkTagOr",
            include_bytes!("../../../assets/editor-markers/LinkTagOr.png") as &[u8],
        ),
        (
            "LinkTagXOr",
            include_bytes!("../../../assets/editor-markers/LinkTagXOr.png") as &[u8],
        ),
        (
            "LinkTagNOr",
            include_bytes!("../../../assets/editor-markers/LinkTagNOr.png") as &[u8],
        ),
        (
            "LinkTagNone",
            include_bytes!("../../../assets/editor-markers/LinkTagNone.png") as &[u8],
        ),
        (
            "LinkTagCount",
            include_bytes!("../../../assets/editor-markers/LinkTagCount.png") as &[u8],
        ),
        (
            "LinkTagPulse",
            include_bytes!("../../../assets/editor-markers/LinkTagPulse.png") as &[u8],
        ),
        (
            "EventTag",
            include_bytes!("../../../assets/editor-markers/EventTag.png") as &[u8],
        ),
    ] {
        let texture_name = format!("__editor_marker_{name}");
        Arc::make_mut(&mut doc.scene.textures).insert(texture_name.clone(), editor_texture(bytes));
        let mesh_index = ensure_marker(
            &mut doc.scene,
            geometry::rito_cube(name, 0.625),
            &texture_name,
        );
        Arc::make_mut(&mut doc.scene.meshes)[mesh_index].texture_name = Some(texture_name);
        tag_meshes.insert(name.to_string(), mesh_index);
    }
    for i in assets.round_markers {
        Arc::make_mut(&mut doc.scene.meshes)[i].texture_name = Some("__editor_marker_area".into());
    }
    for name in names {
        if assets.actors.contains_key(&name) {
            continue;
        }
        let profile = resources
            .actor_info
            .get(&name)
            .and_then(|a| a.as_map().ok())
            .and_then(|m| m.get("profile"))
            .and_then(|v| v.as_string().ok())
            .map(ToString::to_string)
            .unwrap_or_default();
        let mut visual = ActorVisual {
            meshes: Vec::new(),
            resource: String::new(),
            profile: profile.clone(),
            has_model: false,
            diagnostic: None,
        };
        let logical = profile == "Area"
            || profile == "SpotBgmTag"
            || name.ends_with("Tag")
            || name.starts_with("Tag")
            || name.starts_with("LinkTag")
            || name.starts_with("EfLocater")
            || name.starts_with("EnvSeEmitPoint")
            || name.starts_with("AirWall")
            || name.starts_with("NavMesh")
            || name.contains("Anchor")
            || matches!(
                name.as_str(),
                "BoxWater"
                    | "AscendingCurrent"
                    | "FlyLocater"
                    | "TreasureSpot_00"
                    | "ObservationPoint"
                    | "SignalFlowchart"
                    | "SwitchTimeLag"
                    | "PosAfterDemoSkip"
            )
            || resources.actor_has_no_model(&name);
        if !logical && profile == "NPC" {
            if let Some(umii) = resources.actor_umii(&name) {
                let platform = if doc.file.big_endian {
                    crate::model::umii::ModelPlatform::WiiU
                } else {
                    crate::model::umii::ModelPlatform::Switch
                };
                match crate::model::umii::synthesize_npc_model_sync(
                    &umii,
                    platform,
                    Some(&resources.model_directories()),
                    project,
                ) {
                    Ok(npc) => {
                        let ns = format!("__umii_{name}");
                        for (name, texture) in npc.textures.iter() {
                            Arc::make_mut(&mut doc.scene.textures)
                                .insert(format!("{ns}::{name}"), texture.clone());
                        }
                        for mut mesh in npc.meshes.iter().cloned() {
                            mesh.texture_name = mesh.texture_name.map(|n| format!("{ns}::{n}"));
                            mesh.texture_refs = mesh
                                .texture_refs
                                .into_iter()
                                .map(|n| format!("{ns}::{n}"))
                                .collect();
                            visual.meshes.push(push_mesh(&mut doc.scene, mesh));
                        }
                        visual.resource = format!("Actor/Pack/{name}.sbactorpack · UMii");
                    }
                    Err(e) => visual.diagnostic = Some(format!("UMii: {e}")),
                }
            }
        }
        if !logical && visual.meshes.is_empty() {
            for (folder, unit) in resources.model_targets(&name) {
                let key = (folder.clone(), unit.clone());
                if let Some(indices) = cache.get(&key) {
                    visual.meshes.extend(indices);
                    visual.resource = format!("{folder} / {unit}");
                    continue;
                }
                let Some(resource) = resources.model_file(&folder, &unit) else {
                    visual.diagnostic = Some(format!("模型文件缺失: {folder} / {unit}"));
                    continue;
                };
                let model_resource = format!("{} / {unit}", resource.path);
                if let Some(indices) = existing_model_meshes.get(&model_resource) {
                    cache.insert(key, indices.clone());
                    visual.meshes.extend(indices);
                    visual.resource = model_resource;
                    continue;
                }
                let bfres = match resources.parse_model(&resource) {
                    Ok(file) => file,
                    Err(e) => {
                        visual.diagnostic = Some(format!("{}: {e}", resource.path));
                        continue;
                    }
                };
                let mut indices = Vec::new();
                let has_unit = bfres.models.iter().any(|(n, _)| n == &unit);
                if !unit.is_empty() && !has_unit {
                    visual.diagnostic = Some(format!("{folder} 中没有子模型 {unit}"));
                    continue;
                }
                let namespace = resource.path.clone();
                if loaded_textures.insert(namespace.clone()) {
                    let names = color_texture_names(&bfres);
                    let mut textures = ModelDocumentState::from_bfres("", "", &Default::default());
                    crate::model::model_preview::texture_loader::extract_bfres_textures_filtered(
                        &bfres,
                        &mut textures,
                        &names,
                    );
                    for suffix in [
                        "Tex", "Tex1", "Tex2", "Tex1.1", "Tex1.2", "Tex1.3", "Tex1.4", "Tex1.5",
                        "Tex1.6", "Tex1.7", "Tex1.8", "Tex1.9",
                    ] {
                        if let Some(tex) =
                            resources.read(&format!("Model/{folder}.{suffix}.sbfres"))
                        {
                            crate::model::model_preview::texture_loader::load_tex_bytes_filtered(
                                &tex.bytes,
                                &mut textures,
                                &names,
                            );
                        }
                    }
                    for (name, texture) in textures.textures.iter() {
                        Arc::make_mut(&mut doc.scene.textures)
                            .insert(format!("{namespace}::{name}"), texture.clone());
                    }
                }
                for (model_name, model) in bfres.models.iter() {
                    if has_unit && model_name != &unit {
                        continue;
                    }
                    for mut mesh in ExtractedMesh::extract_from_model(model, bfres.byte_order) {
                        mesh.texture_name = mesh.texture_name.map(|n| format!("{namespace}::{n}"));
                        mesh.texture_refs = mesh
                            .texture_refs
                            .into_iter()
                            .map(|n| format!("{namespace}::{n}"))
                            .collect();
                        indices.push(push_mesh(&mut doc.scene, mesh));
                    }
                }
                cache.insert(key, indices.clone());
                existing_model_meshes.insert(model_resource, indices.clone());
                visual.meshes.extend(indices);
                visual.resource = format!("{} / {unit}", resource.path);
            }
        }
        visual.has_model = !visual.meshes.is_empty();
        if visual.has_model {
            visual.diagnostic = None;
        }
        if visual.meshes.is_empty() {
            if logical {
                if is_area_profile(&profile, &name) {
                    assets.area_actors.insert(name.clone());
                    visual.meshes.push(area_marker_index);
                    visual.resource = format!("区域 / {name}");
                } else if let Some(tag) = rito_tag_name(&name) {
                    visual.meshes.push(tag_meshes[tag]);
                    visual.resource = format!("Rito 标记 / {tag}");
                } else {
                    visual.meshes.push(marker_index);
                    visual.resource = format!("逻辑对象 / {name}");
                }
            } else {
                visual.meshes.push(marker_index);
                visual
                    .diagnostic
                    .get_or_insert_with(|| "没有可绘制的模型网格".into());
            }
        }
        assets.actors.insert(name, visual);
    }
}

fn editor_texture(bytes: &[u8]) -> Arc<RgbaTexture> {
    let image = image::load_from_memory(bytes)
        .expect("embedded Rito editor marker texture must decode")
        .to_rgba8();
    Arc::new(RgbaTexture {
        width: image.width(),
        height: image.height(),
        rgba: Arc::new(image.into_raw()),
    })
}

fn is_area_profile(profile: &str, name: &str) -> bool {
    profile == "Area"
        || profile == "SpotBgmTag"
        || matches!(
            name,
            "BoxWater" | "AscendingCurrent" | "AirWall" | "NavMesh"
        )
}

fn rito_tag_name(name: &str) -> Option<&'static str> {
    [
        "LinkTagAnd",
        "LinkTagNAnd",
        "LinkTagOr",
        "LinkTagXOr",
        "LinkTagNOr",
        "LinkTagNone",
        "LinkTagCount",
        "LinkTagPulse",
        "EventTag",
    ]
    .into_iter()
    .find(|tag| name == *tag || name.starts_with(&format!("{tag}_")))
}

fn color_texture_names(file: &BfresFile) -> HashSet<String> {
    file.models
        .values()
        .flat_map(|model| model.materials.values())
        .filter_map(|material| {
            material
                .albedo_binding()
                .and_then(|i| material.texture_refs.get(i))
        })
        .filter(|name| name.as_str() != "MaterialAlb")
        .cloned()
        .collect()
}

fn load_actor_terrain_materials(
    doc: &mut MapDocumentState,
    resources: &mut Resources,
    assets: &mut MapAssets,
    materials: &mut super::terrain_material::TerrainMaterials,
) {
    let bindings: Vec<_> = doc
        .scene
        .meshes
        .iter()
        .filter_map(|m| {
            let name = m.texture_name.as_ref()?;
            let (_, layer) = name.rsplit_once("::MaterialAlb#")?;
            if doc.scene.textures.contains_key(name) {
                return None;
            }
            Some((name.clone(), layer.parse::<u32>().ok()?))
        })
        .collect();
    if bindings.is_empty() {
        return;
    }
    if let Err(e) = materials.load_game_array(resources) {
        assets.warnings.push(e);
    }
    for (name, index) in bindings {
        if let Some(texture) = materials.layer(index) {
            Arc::make_mut(&mut doc.scene.textures).insert(name, texture);
        } else {
            assets
                .warnings
                .push(format!("岩壁材质缺少 MaterialAlb 第 {index} 层"));
        }
    }
}

fn load_dungeon_background(
    doc: &mut MapDocumentState,
    resources: &mut Resources,
    assets: &mut MapAssets,
) -> Result<(), String> {
    let path = doc.scene.file_path.replace('\\', "/");
    let dungeon = path
        .rsplit('/')
        .next()
        .unwrap_or("")
        .split('_')
        .next()
        .unwrap_or("");
    if !dungeon.starts_with("Dungeon") {
        return Ok(());
    }
    let folder = format!("DgnMrgPrt_{dungeon}");
    let Some(resource) = resources.model_file(&folder, "") else {
        return Err(format!("缺少神庙合并模型 {folder}"));
    };
    let bfres = resources.parse_model(&resource)?;
    let ns = resource.path;
    let mut textures = ModelDocumentState::from_bfres("", "", &Default::default());
    let names = color_texture_names(&bfres);
    crate::model::model_preview::texture_loader::extract_bfres_textures_filtered(
        &bfres,
        &mut textures,
        &names,
    );
    for suffix in ["Tex", "Tex1", "Tex2"] {
        if let Some(resource) = resources.read(&format!("Model/{folder}.{suffix}.sbfres")) {
            crate::model::model_preview::texture_loader::load_tex_bytes_filtered(
                &resource.bytes,
                &mut textures,
                &names,
            );
        }
    }
    for (name, texture) in textures.textures.iter() {
        Arc::make_mut(&mut doc.scene.textures).insert(format!("{ns}::{name}"), texture.clone());
    }
    for (_, model) in bfres.models.iter() {
        for mut mesh in ExtractedMesh::extract_from_model(model, bfres.byte_order) {
            mesh.texture_name = mesh.texture_name.map(|n| format!("{ns}::{n}"));
            mesh.texture_refs = mesh
                .texture_refs
                .into_iter()
                .map(|n| format!("{ns}::{n}"))
                .collect();
            assets
                .background_meshes
                .push(push_mesh(&mut doc.scene, mesh));
        }
    }
    Ok(())
}

fn load_context(doc: &MapDocumentState, resources: &mut Resources, assets: &mut MapAssets) {
    let path = &doc.scene.file_path;
    let pair = if path.ends_with("_Dynamic.smubin") {
        Some(("_Dynamic.smubin", "_Static.smubin"))
    } else if path.ends_with("_Static.smubin") {
        Some(("_Static.smubin", "_Dynamic.smubin"))
    } else if path.ends_with("_Dynamic.mubin") {
        Some(("_Dynamic.mubin", "_Static.mubin"))
    } else if path.ends_with("_Static.mubin") {
        Some(("_Static.mubin", "_Dynamic.mubin"))
    } else {
        None
    };
    let Some((suffix, replacement)) = pair else {
        return;
    };
    let candidate = format!("{}{replacement}", path.strip_suffix(suffix).unwrap());
    let mut found = crate::model::file_tree::read_file_from_path(&candidate)
        .ok()
        .map(|bytes| super::resources::Resource {
            path: candidate,
            bytes: Arc::new(bytes),
        });
    if found.is_none() {
        let normalized = path.replace('\\', "/");
        let stem = normalized
            .rsplit('/')
            .next()
            .unwrap_or("")
            .split('_')
            .next()
            .unwrap_or("");
        let field = if path.contains("AocField") {
            "AocField"
        } else if stem.starts_with("Dungeon") {
            "CDungeon"
        } else {
            "MainField"
        };
        found = resources.read(&format!("Map/{field}/{stem}/{stem}{replacement}"));
    }
    if let Some(resource) = found {
        match uking_library::map::MapFile::from_bytes(&resource.bytes) {
            Ok(file) => {
                assets.context_actors = file.actors().to_vec();
                assets.context_path = Some(resource.path);
            }
            Err(e) => assets.warnings.push(format!("配套地图读取失败: {e}")),
        }
    }
}

fn load_terrain(
    doc: &mut MapDocumentState,
    resources: &mut Resources,
    assets: &mut MapAssets,
    materials: &mut super::terrain_material::TerrainMaterials,
) -> Result<(), String> {
    let path = doc.scene.file_path.replace('\\', "/");
    let file_name = path.rsplit('/').next().unwrap_or("");
    let section = file_name.split('_').next().unwrap_or("");
    let Some(center) = section_center(section) else {
        return Ok(());
    };
    let field = if path.contains("AocField") {
        "AocField"
    } else {
        "MainField"
    };
    let resource = resources
        .read(&format!("Terrain/A/{field}.tscb"))
        .ok_or("没有找到地形索引 TSCB，请配置游戏资源目录")?;
    let data = uking_library::decompress(&resource.bytes)?;
    let table = TerrainTable::from_bytes(&data)?;
    if let Err(error) = materials.load_game_array(resources) {
        assets.warnings.push(error);
    }
    let mut render_data = materials.render_data(&table);
    let terrain_tiles = table.section_tiles(center, 1000., 250.);
    for &tile in &terrain_tiles {
        let height = match resources.terrain_file(field, tile, "hght") {
            Ok(data) => HeightField::from_bytes(tile.clone(), &data)?,
            Err(e) => {
                assets.warnings.push(e);
                continue;
            }
        };
        let mut mesh = geometry::terrain_mesh(&height, None)?;
        geometry::clip_tile(&mut mesh, tile, &terrain_tiles, center);
        if mesh.indices.is_empty() {
            continue;
        }
        match resources.terrain_file(field, tile, "mate") {
            Ok(material) if material.len() == 256 * 256 * 4 => {
                let texture = materials.bake(&table, &material);
                let texture_name = format!("__terrain_{}", tile.name);
                mesh.texture_name = Some(texture_name.clone());
                render_data.tiles.insert(
                    texture_name.clone(),
                    Arc::new(RgbaTexture {
                        width: 256,
                        height: 256,
                        rgba: Arc::new(material),
                    }),
                );
                Arc::make_mut(&mut doc.scene.textures).insert(texture_name, Arc::new(texture));
            }
            Ok(_) => assets
                .warnings
                .push(format!("{} 材质图长度无效", tile.name)),
            Err(e) => assets.warnings.push(e),
        }
        assets.terrain_meshes.push(push_mesh(&mut doc.scene, mesh));
        assets.heights.push(height);
    }
    let water_tiles = table.water_tiles(center, 1000., 250.);
    for &tile in &water_tiles {
        match resources.terrain_file(field, tile, "water.extm") {
            Ok(water) => {
                let height = HeightField {
                    tile: tile.clone(),
                    heights: vec![0.; 256 * 256],
                };
                if let Ok(mut mesh) = geometry::terrain_mesh(&height, Some(&water)) {
                    geometry::clip_tile(&mut mesh, tile, &water_tiles, center);
                    geometry::clip_buried_water(&mut mesh, &assets.heights);
                    if !mesh.indices.is_empty() {
                        let texture_name = "__terrain_water".to_string();
                        mesh.texture_name = Some(texture_name.clone());
                        Arc::make_mut(&mut doc.scene.textures)
                            .entry(texture_name)
                            .or_insert_with(|| {
                                Arc::new(RgbaTexture {
                                    width: 1,
                                    height: 1,
                                    rgba: Arc::new(vec![42, 132, 165, 190]),
                                })
                            });
                        assets.water_meshes.push(push_mesh(&mut doc.scene, mesh));
                    }
                }
            }
            Err(e) => assets.warnings.push(e),
        }
    }
    if materials.missing {
        assets.warnings.push("未找到部分地形材质贴图；可将 MaterialAlb_*.png 放入 .cache/map_terrain，或设置 ZEVENT_TERRAIN_CACHE。地形高度仍使用原始数据。".into());
    }
    doc.scene.terrain = Some(Arc::new(render_data));
    Ok(())
}

#[cfg(test)]
mod asset_tests {
    use super::*;
    use roead::byml::Byml;
    use uking_library::map::{hash_id, MapFile};

    fn empty_document(path: &str) -> MapDocumentState {
        let root = Byml::from_text("Objs: []\nRails: [{Preserved: true}]").unwrap();
        let file = MapFile::from_bytes(&root.to_binary(roead::Endian::Little)).unwrap();
        let mut doc = MapDocumentState::empty(file, path, "asset-test");
        doc.loading = false;
        doc
    }

    #[test]
    fn incremental_markers_preserve_document_camera_and_scene_indices() {
        let settings = GamePathSettings::default();
        let mut doc = empty_document("incremental-test");
        let existing = geometry::cube("existing terrain");
        push_mesh(&mut doc.scene, existing.clone());
        doc.scene.terrain = Some(Arc::new(crate::model::model_preview::TerrainRenderData {
            layers: Vec::new(),
            material_params: Vec::new(),
            tiles: HashMap::new(),
        }));
        let original_root = doc.file.root.clone();
        let original_camera = doc.scene.camera.clone();
        let original_terrain = doc.scene.terrain.clone().unwrap();
        let paired = Byml::from_text("{UnitConfigName: LinkTagOr, HashId: !u 42}").unwrap();
        Arc::make_mut(&mut doc.assets).context_actors.push(paired);
        let doc = load_actor_asset(doc, settings.clone(), None, "LinkTagAnd").unwrap();
        assert!(Arc::ptr_eq(&doc.file.root, &original_root));
        assert_eq!(doc.scene.camera, original_camera);
        assert!(Arc::ptr_eq(
            doc.scene.terrain.as_ref().unwrap(),
            &original_terrain
        ));
        assert_eq!(doc.scene.meshes[0], existing);
        assert_eq!(
            doc.assets.context_actors.first().and_then(hash_id),
            Some(42)
        );
        let count = doc.scene.meshes.len();
        let doc = load_actor_asset(doc, settings.clone(), None, "EventTag").unwrap();
        assert_eq!(doc.scene.meshes.len(), count, "editor markers are reused");
        let meshes = doc.scene.meshes.clone();
        let textures = doc.scene.textures.clone();
        let assets = doc.assets.clone();
        let doc = load_actor_asset(doc, settings, None, "EventTag").unwrap();
        assert!(Arc::ptr_eq(&meshes, &doc.scene.meshes));
        assert!(Arc::ptr_eq(&textures, &doc.scene.textures));
        assert!(Arc::ptr_eq(&assets, &doc.assets));
    }

    #[test]
    fn marker_preview_contains_only_one_origin_instance() {
        let scene = load_asset_preview(
            "asset-test",
            false,
            GamePathSettings::default(),
            None,
            "LinkTagAnd",
        )
        .unwrap();
        let instances = scene.instances.as_ref().unwrap();
        assert_eq!(instances.len(), 1);
        assert_eq!(instances[0].transform.translation, [0.; 3]);
        assert!(!instances[0].selected);
        assert!(scene.terrain.is_none());
        assert!(scene.camera.distance.is_finite());
        assert!(scene.total_triangles > 0);
    }

    #[test]
    #[ignore = "requires ZEVENT_TEST_MAP and local game resources"]
    fn real_asset_incremental_load_and_isolated_preview() {
        let path = std::env::var("ZEVENT_TEST_MAP").expect("set ZEVENT_TEST_MAP");
        let file =
            MapFile::from_bytes(&crate::model::file_tree::read_file_from_path(&path).unwrap())
                .unwrap();
        let settings = GamePathSettings::load_from_disk();
        let doc = load_scene(
            MapDocumentState::empty(file, &path, "asset-test"),
            settings.clone(),
            None,
        )
        .unwrap();
        let root = doc.file.root.clone();
        let camera = doc.scene.camera.clone();
        let meshes = doc.scene.meshes.clone();
        let count = meshes.len();
        let terrain = doc.scene.terrain.clone();
        let candidate = doc
            .assets
            .catalog
            .entries
            .iter()
            .find(|asset| !asset.model.is_empty() && !doc.assets.actors.contains_key(&asset.name))
            .map(|asset| asset.name.clone())
            .expect("unloaded real actor with a model");
        let doc = load_actor_asset(doc, settings.clone(), None, &candidate).unwrap();
        assert!(Arc::ptr_eq(&root, &doc.file.root));
        assert_eq!(camera, doc.scene.camera);
        assert!(
            doc.assets.actors[&candidate].has_model,
            "{:?}",
            doc.assets.actors[&candidate].diagnostic
        );
        assert!(doc.scene.meshes.len() > count);
        assert_eq!(&doc.scene.meshes[..count], meshes.as_slice());
        if let Some(terrain) = terrain {
            assert!(Arc::ptr_eq(&terrain, doc.scene.terrain.as_ref().unwrap()));
        }
        let preview = load_asset_preview(&path, false, settings, None, &candidate).unwrap();
        assert!(preview.terrain.is_none());
        assert!(preview.instances.as_ref().unwrap().len() < preview.meshes.len());
        assert!(preview.total_triangles > 12);
        assert_eq!(
            preview.bounding_box_min,
            doc.assets.actors[&candidate].meshes.iter().fold(
                [f32::INFINITY; 3],
                |mut bound, &index| {
                    let min = doc.scene.meshes[index].bounding_box.min();
                    for (axis, value) in [min.x, min.y, min.z].into_iter().enumerate() {
                        bound[axis] = bound[axis].min(value);
                    }
                    bound
                }
            )
        );
    }
}
