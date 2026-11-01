use super::*;
use crate::model::model_preview::OrbitCamera;

#[test]
#[ignore = "requires ZEVENT_TEST_MAP and configured local game resources"]
fn map_performance_benchmark() {
    use std::time::Instant;
    let path = std::env::var("ZEVENT_TEST_MAP").expect("set ZEVENT_TEST_MAP");
    let file =
        MapFile::from_bytes(&crate::model::file_tree::read_file_from_path(&path).unwrap()).unwrap();
    let start = Instant::now();
    let mut doc = loader::load_scene(
        MapDocumentState::empty(file, &path, "performance"),
        crate::model::settings::GamePathSettings::load_from_disk(),
        None,
    )
    .unwrap();
    println!(
        "BENCH load_ms={:.1} meshes={} textures={} triangles={}",
        start.elapsed().as_secs_f64() * 1000.,
        doc.scene.meshes.len(),
        doc.scene.textures.len(),
        doc.scene.total_triangles
    );
    let start = Instant::now();
    for _ in 0..100 {
        std::hint::black_box(doc.clone());
    }
    println!(
        "BENCH snapshot_ms={:.3}",
        start.elapsed().as_secs_f64() * 10.
    );
    let start = Instant::now();
    assert!(
        crate::ui::views::model_preview::wgpu_renderer::render_wgpu_scene(&doc.scene, 1766., 1172.)
            .is_some()
    );
    println!(
        "BENCH first_frame_ms={:.1}",
        start.elapsed().as_secs_f64() * 1000.
    );
    let mut times = Vec::new();
    for _ in 0..12 {
        doc.scene.camera.yaw += 0.03;
        let start = Instant::now();
        assert!(
            crate::ui::views::model_preview::wgpu_renderer::render_wgpu_scene(
                &doc.scene, 1766., 1172.
            )
            .is_some()
        );
        times.push(start.elapsed().as_secs_f64() * 1000.);
    }
    times.sort_by(f64::total_cmp);
    println!(
        "BENCH orbit_ms_p50={:.2} orbit_ms_max={:.2}",
        times[times.len() / 2],
        times[times.len() - 1]
    );
    if let Ok(output) = std::env::var("ZEVENT_TEST_RENDER") {
        doc.select(Some(0));
        doc.focus_selected();
        doc.select(None);
        let img = crate::ui::views::model_preview::wgpu_renderer::render_wgpu_scene(
            &doc.scene, 1280., 800.,
        )
        .unwrap();
        let mut rgba = img.as_bytes(0).unwrap().to_vec();
        for p in rgba.chunks_exact_mut(4) {
            p.swap(0, 2);
            if p[3] == 0 {
                p.copy_from_slice(&[23, 32, 39, 255]);
            }
        }
        image::save_buffer(output, &rgba, 1280, 800, image::ColorType::Rgba8).unwrap();
    }
}

#[test]
fn focus_frames_offset_child_meshes_after_rotation_and_negative_scale() {
    let root = Byml::from_text("Objs: [{UnitConfigName: Tree, HashId: !u 1}]").unwrap();
    let mut file = MapFile::from_bytes(&root.to_binary(roead::Endian::Little)).unwrap();
    let transform = Transform {
        translation: [100., 20., -30.],
        rotation: [0., std::f32::consts::FRAC_PI_2, 0.],
        scale: [-2., 3., 0.5],
    };
    file.set_transform(0, transform).unwrap();
    let mut doc = MapDocumentState::empty(file, "focus-test", "focus-test");
    for x in [0., 20.] {
        let mut mesh = geometry::cube("child");
        for p in &mut mesh.positions {
            p.x += x;
            p.y += 10.;
        }
        mesh.bounding_box = rbfres_library::ExtractedMesh::compute_bounding_box(&mesh.positions);
        loader::push_mesh(&mut doc.scene, mesh);
    }
    Arc::make_mut(&mut doc.assets).actors.insert(
        "Tree".into(),
        ActorVisual {
            meshes: vec![0, 1],
            resource: "test".into(),
            has_model: true,
            diagnostic: None,
        },
    );
    doc.select(Some(0));
    doc.focus_selected();
    for (actual, expected) in doc.scene.camera.target.iter().zip([100., 50., -10.]) {
        assert!((actual - expected).abs() < 1e-4);
    }
    for yaw in [0., 0.7, 2.] {
        doc.scene.camera.yaw = yaw;
        for mesh in doc.scene.meshes.iter() {
            for p in &mesh.positions {
                let (screen, _) = doc
                    .scene
                    .camera
                    .project_point(transform.point([p.x, p.y, p.z]), 800., 800.)
                    .expect("focused geometry must be in front of the camera");
                assert!(screen.into_iter().all(|v| (0. ..=800.).contains(&v)));
            }
        }
    }
}

#[test]
#[ignore = "requires ZEVENT_UI_ORIGINAL and ZEVENT_UI_SAVED after a +1 X edit in the app"]
fn ui_saved_map_changes_only_selected_translation() {
    let original = std::env::var("ZEVENT_UI_ORIGINAL").expect("set ZEVENT_UI_ORIGINAL");
    let saved = std::env::var("ZEVENT_UI_SAVED").expect("set ZEVENT_UI_SAVED");
    let mut before =
        MapFile::from_bytes(&crate::model::file_tree::read_file_from_path(&original).unwrap())
            .unwrap();
    let after = MapFile::from_bytes(&crate::model::file_tree::read_file_from_path(&saved).unwrap())
        .unwrap();
    assert_eq!(before.actors().len(), after.actors().len());
    let changed: Vec<_> = before
        .actors()
        .iter()
        .zip(after.actors())
        .enumerate()
        .filter_map(|(i, (a, b))| (a != b).then_some(i))
        .collect();
    assert_eq!(changed.len(), 1, "the UI must change exactly one actor");
    let i = changed[0];
    let mut expected = Transform::from_actor(&before.actors()[i]);
    expected.translation[0] += 1.;
    before.set_transform(i, expected).unwrap();
    assert_eq!(
        after.root, before.root,
        "unrelated BYML data changed during UI save"
    );
    assert_eq!(
        (after.big_endian, after.compressed, after.version),
        (before.big_endian, before.compressed, before.version)
    );
    println!(
        "UI save verified: actor {i}, {}, HashId {:?}, X {}",
        uking_library::map::actor_name(&after.actors()[i]),
        uking_library::map::hash_id(&after.actors()[i]),
        expected.translation[0]
    );
}

#[test]
#[ignore = "requires configured game dump"]
fn rock_material_uses_shader_sampler_and_array_layer() {
    let settings = crate::model::settings::GamePathSettings::load_from_disk();
    let mut resources = resources::Resources::new("D-7_Dynamic.smubin", None, &settings, false);
    for (folder, expected) in [
        ("FldObj_BankingFlatCliffWhite_A", 17),
        ("Obj_SnowBall_A", 33),
    ] {
        let resource = resources.model_file(folder, "").unwrap();
        let file = rbfres_library::BfresFile::from_bytes(
            &uking_library::decompress(&resource.bytes).unwrap(),
        )
        .unwrap();
        let meshes: Vec<_> = file
            .models
            .iter()
            .flat_map(|(_, m)| {
                rbfres_library::ExtractedMesh::extract_from_model(m, file.byte_order)
            })
            .collect();
        assert!(meshes
            .iter()
            .any(|m| m.texture_name.as_deref() == Some(&format!("MaterialAlb#{expected}"))));
        assert!(meshes.iter().all(|m| m
            .texture_name
            .as_ref()
            .is_none_or(|t| !t.contains("Nrm") && !t.contains("Cmb"))));
    }
}

#[test]
#[ignore = "requires local game dump"]
fn actor_configuration_resolves_umii_and_logic() {
    let settings = crate::model::settings::GamePathSettings::load_from_disk();
    let mut res = resources::Resources::new("F-5_Dynamic.smubin", None, &settings, false);
    assert!(res.actor_umii("Npc_NorthHatelHatago001").is_some());
    assert!(res.actor_umii("Npc_HiddenKorokGround").is_some());
    assert!(res.actor_has_no_model("DungeonBird_test"));
    assert!(res.actor_has_no_model("ConstraintDummy"));
}

#[test]
fn water_uses_64_samples_and_keeps_sea_level() {
    let tile = uking_library::terrain::TerrainTile {
        name: "5000000000".into(),
        center: [0., 0.],
        size: 250.,
        min_height: 0.,
        max_height: 800.,
        has_water: true,
    };
    let field = uking_library::terrain::HeightField {
        tile,
        heights: vec![0.; 256 * 256],
    };
    let mut water = vec![0; 64 * 64 * 8];
    let mesh = geometry::terrain_mesh(&field, Some(&water)).unwrap();
    assert_eq!(mesh.positions.len(), 64 * 64);
    assert_eq!(mesh.indices.len(), 63 * 63 * 6);
    let mut buried = mesh.clone();
    let mut ground = field.clone();
    ground.heights.fill(10.);
    geometry::clip_buried_water(&mut buried, &[ground]);
    assert!(buried.indices.is_empty());
    let mut sea = mesh.clone();
    geometry::clip_buried_water(&mut sea, &[field.clone()]);
    assert_eq!(sea.indices, mesh.indices);
    for p in water.chunks_exact_mut(8) {
        p[7] = 255;
    }
    assert!(geometry::terrain_mesh(&field, Some(&water))
        .unwrap()
        .indices
        .is_empty());
}

#[test]
fn software_renderer_places_instances_and_invalidates_after_moving() {
    let mesh = geometry::cube("test");
    let mut scene = crate::model::model_preview::ModelDocumentState::from_bfres(
        "instance-test",
        "instance-test",
        &Default::default(),
    );
    loader::push_mesh(&mut scene, mesh);
    scene.camera.target = [100., 0., 0.];
    scene.camera.distance = 10.;
    scene.instances = Some(Arc::new(vec![crate::model::model_preview::ModelInstance {
        mesh_index: 0,
        transform: Transform {
            translation: [100., 0., 0.],
            ..Default::default()
        },
        selected: false,
    }]));
    let first =
        crate::ui::views::model_preview::rasterizer::rasterize_scene(&scene, 128., 128.).unwrap();
    assert!(
        first
            .as_bytes(0)
            .unwrap()
            .chunks_exact(4)
            .filter(|p| p[3] > 0)
            .count()
            > 100
    );
    Arc::make_mut(scene.instances.as_mut().unwrap())[0]
        .transform
        .translation[0] = 1000.;
    let second =
        crate::ui::views::model_preview::rasterizer::rasterize_scene(&scene, 128., 128.).unwrap();
    assert_eq!(
        second
            .as_bytes(0)
            .unwrap()
            .chunks_exact(4)
            .filter(|p| p[3] > 0)
            .count(),
        0
    );
}

#[test]
#[ignore = "requires a graphics adapter"]
fn gpu_render_cache_tracks_camera_edit_undo_selection_and_textures() {
    use crate::model::model_preview::{ModelDocumentState, ModelInstance, RgbaTexture};
    use crate::ui::views::model_preview::wgpu_renderer::render_wgpu_scene;
    let mut scene = ModelDocumentState::from_bfres("cache-test", "cache-test", &Default::default());
    let mut mesh = geometry::cube("cache-test");
    mesh.texture_name = Some("color".into());
    loader::push_mesh(&mut scene, mesh);
    scene.camera.target = [0.; 3];
    scene.camera.distance = 8.;
    scene.instances = Some(Arc::new(vec![ModelInstance {
        mesh_index: 0,
        transform: Transform::default(),
        selected: false,
    }]));
    Arc::make_mut(&mut scene.textures).insert(
        "color".into(),
        Arc::new(RgbaTexture {
            width: 1,
            height: 1,
            rgba: Arc::new(vec![200, 40, 20, 255]),
        }),
    );
    let render = |s: &ModelDocumentState| render_wgpu_scene(s, 128., 128.).unwrap();
    let original = scene.clone();
    let first = render(&scene);
    assert!(first.as_bytes(0).unwrap().chunks_exact(4).any(|p| p[3] > 0));
    assert!(
        Arc::ptr_eq(&first, &render(&scene)),
        "unchanged frame must be reused"
    );
    scene.camera.yaw += 0.4;
    assert_ne!(first.as_bytes(0), render(&scene).as_bytes(0));
    scene = original.clone();
    scene.selected_mesh_idx = Some(0);
    assert_ne!(first.as_bytes(0), render(&scene).as_bytes(0));
    scene = original.clone();
    Arc::make_mut(scene.instances.as_mut().unwrap())[0]
        .transform
        .translation[0] = 100.;
    assert!(render(&scene)
        .as_bytes(0)
        .unwrap()
        .chunks_exact(4)
        .all(|p| p[3] == 0));
    scene = original.clone();
    assert_eq!(
        first.as_bytes(0),
        render(&scene).as_bytes(0),
        "undo must restore pixels"
    );
    Arc::make_mut(&mut scene.textures).insert(
        "color".into(),
        Arc::new(RgbaTexture {
            width: 1,
            height: 1,
            rgba: Arc::new(vec![20, 200, 40, 255]),
        }),
    );
    assert_ne!(first.as_bytes(0), render(&scene).as_bytes(0));
    scene = original.clone();
    Arc::make_mut(&mut scene.mesh_items)[0].visible = false;
    assert!(render(&scene)
        .as_bytes(0)
        .unwrap()
        .chunks_exact(4)
        .all(|p| p[3] == 0));
    assert_eq!(first.as_bytes(0), render(&original).as_bytes(0));
}

#[test]
fn picking_respects_rotation_scale_and_nearest_hit() {
    let cube = geometry::cube("test");
    let t = Transform {
        translation: [0., 0., 10.],
        rotation: [0.3, 0.7, 0.2],
        scale: [2., 3., 4.],
    };
    let hit = geometry::ray_mesh([0., 0., 0.], [0., 0., 1.], t, &cube, 100.).unwrap();
    assert!(hit > 3. && hit < 10.);
    assert!(geometry::ray_mesh([100., 0., 0.], [0., 0., 1.], t, &cube, 100.).is_none());
    assert!(geometry::ray_mesh([0., 0., 0.], [0., 0., 1.], t, &cube, 1.).is_none());
}

#[test]
#[ignore = "requires ZEVENT_TEST_MAP and a configured local game dump"]
fn real_map_models_terrain_render_and_save() {
    let path = std::env::var("ZEVENT_TEST_MAP")
        .expect("set ZEVENT_TEST_MAP to a local .smubin or SARC virtual path");
    let bytes = crate::model::file_tree::read_file_from_path(&path).unwrap();
    let file = MapFile::from_bytes(&bytes).unwrap();
    let doc = MapDocumentState::empty(file, &path, "map validation");
    let mut doc = loader::load_scene(
        doc,
        crate::model::settings::GamePathSettings::load_from_disk(),
        None,
    )
    .unwrap();
    let missing: Vec<_> = doc
        .assets
        .actors
        .iter()
        .filter_map(|(n, v)| v.diagnostic.as_ref().map(|e| format!("{n}: {e}")))
        .collect();
    println!("Objects: {}, context: {}, model types: {}, missing: {}, terrain: {}, water: {}, meshes: {}, triangles: {}, textures: {}",doc.file.actors().len(),doc.assets.context_actors.len(),doc.assets.actors.values().filter(|v|v.has_model).count(),missing.len(),doc.assets.heights.len(),doc.assets.water_meshes.len(),doc.scene.meshes.len(),doc.scene.total_triangles,doc.scene.textures.len());
    for error in &missing {
        println!("MISSING {error}");
    }
    for warning in &doc.assets.warnings {
        println!("WARNING {warning}");
    }
    if path.contains("Dungeon") {
        assert!(
            !doc.assets.background_meshes.is_empty(),
            "no merged shrine model loaded"
        );
        for &index in &doc.assets.background_meshes {
            let mesh = &doc.scene.meshes[index];
            assert!(
                mesh.texture_name
                    .as_ref()
                    .is_some_and(|n| doc.scene.textures.contains_key(n)),
                "unresolved shrine material {:?}",
                mesh.material_name
            );
        }
    }
    assert!(
        doc.assets.actors.values().any(|v| v.has_model),
        "no real actor models loaded"
    );
    if path.contains("MainField") {
        assert!(!doc.assets.heights.is_empty(), "no terrain loaded");
    }
    if path.contains("F-5_Dynamic") {
        assert!(!doc.assets.water_meshes.is_empty());
        assert!(missing.is_empty());
        assert!(doc.assets.warnings.is_empty());
    }
    assert_eq!(
        MapFile::from_bytes(&doc.file.to_bytes()).unwrap().root,
        doc.file.root
    );
    let original = doc.file.root.clone();
    doc.select(Some(0));
    let mut t = Transform::from_actor(&doc.file.actors()[0]);
    t.translation[0] += 1.;
    doc.change_transform(t).unwrap();
    assert!(doc.file.is_modified());
    doc.undo();
    assert_eq!(doc.file.root, original);
    doc.redo();
    assert_ne!(doc.file.root, original);
    doc.undo();
    if let Ok(output) = std::env::var("ZEVENT_TEST_RENDER") {
        let image = crate::ui::views::model_preview::wgpu_renderer::render_wgpu_scene(
            &doc.scene, 1280., 800.,
        )
        .expect("GPU renderer failed");
        let mut rgba = image.as_bytes(0).unwrap().to_vec();
        let nonempty = rgba.chunks_exact(4).filter(|p| p[3] > 0).count();
        assert!(nonempty > 1000, "map render is empty");
        for p in rgba.chunks_exact_mut(4) {
            p.swap(0, 2);
            if p[3] == 0 {
                p.copy_from_slice(&[23, 32, 39, 255]);
            }
        }
        image::save_buffer(output, &rgba, 1280, 800, image::ColorType::Rgba8).unwrap();
    }
}

#[test]
fn screen_ray_points_at_camera_target() {
    let camera = OrbitCamera::new();
    let (_, _, f) = camera.compute_basis();
    let eye = camera.compute_eye();
    let target = std::array::from_fn::<_, 3, _>(|i| eye[i] + f[i] * camera.distance);
    for (a, b) in target.iter().zip(camera.target) {
        assert!((a - b).abs() < 1e-4);
    }
}
