//! UMii 3D 模型部件合成算法核心 (支持 Switch NX 与 Wii U Cafe 双平台 BFRES 模型装配)

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use rbfres_library::{BfresFile, ExtractedMesh};
use serde::{Deserialize, Serialize};

use super::data::UmiiData;
use super::parts::{collect_model_search_dirs, generate_part_targets, locate_part_file};
use crate::model::model_preview::camera::OrbitCamera;
use crate::model::model_preview::texture_loader::load_companion_textures;
use crate::model::model_preview::types::{PreviewMeshItem, RenderMode};
use crate::model::model_preview::ModelDocumentState;

/// 目标游戏资产运行平台
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ModelPlatform {
    #[default]
    Switch,
    WiiU,
}

#[allow(dead_code)]
impl ModelPlatform {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Switch => "Nintendo Switch",
            Self::WiiU => "Wii U",
        }
    }

    pub fn endian(&self) -> roead::Endian {
        match self {
            Self::Switch => roead::Endian::Little,
            Self::WiiU => roead::Endian::Big,
        }
    }
}

/// 执行 UMii 多部件模型的同步合成装配
pub fn synthesize_npc_model_sync(
    data: &UmiiData,
    platform: ModelPlatform,
    custom_search_dirs: Option<&[PathBuf]>,
    project_root: Option<&str>,
) -> Result<ModelDocumentState, String> {
    let search_dirs = if let Some(custom) = custom_search_dirs {
        custom.to_vec()
    } else {
        collect_model_search_dirs(platform, project_root)
    };

    let part_targets = generate_part_targets(data);
    let mut all_meshes: Vec<ExtractedMesh> = Vec::new();
    let mut mesh_items: Vec<PreviewMeshItem> = Vec::new();
    let mut textures = std::collections::HashMap::new();

    let mut global_min = [f32::MAX; 3];
    let mut global_max = [f32::MIN; 3];
    let mut total_vertices = 0;
    let mut total_triangles = 0;
    let mut is_switch_detected = platform == ModelPlatform::Switch;
    let mut loaded_part_count = 0;

    for target in &part_targets {
        let part_path = match locate_part_file(target, &search_dirs) {
            Some(p) => p,
            None => continue,
        };

        let raw_bytes = match fs::read(&part_path) {
            Ok(b) => b,
            Err(_) => continue,
        };

        let decompressed = roead::yaz0::decompress(&raw_bytes).unwrap_or(raw_bytes);
        let bfres = match BfresFile::from_bytes(&decompressed) {
            Ok(f) => f,
            Err(_) => continue,
        };

        loaded_part_count += 1;
        is_switch_detected = bfres.is_switch;

        let part_meshes = bfres.extract_meshes();
        for mut mesh in part_meshes {
            let v_count = mesh.positions.len();
            let t_count = mesh.indices.len() / 3;
            total_vertices += v_count;
            total_triangles += t_count;

            let b_min = mesh.bounding_box.min();
            let b_max = mesh.bounding_box.max();
            let sub_min = [b_min.x, b_min.y, b_min.z];
            let sub_max = [b_max.x, b_max.y, b_max.z];

            for axis in 0..3 {
                global_min[axis] = global_min[axis].min(sub_min[axis]);
                global_max[axis] = global_max[axis].max(sub_max[axis]);
            }

            let item_name = format!("{}:{}", target.category, mesh.name);
            mesh.name = item_name.clone();

            mesh_items.push(PreviewMeshItem {
                name: item_name,
                material_name: mesh
                    .material_name
                    .clone()
                    .unwrap_or_else(|| "Default".to_string()),
                texture_name: mesh.texture_name.clone(),
                vertex_count: v_count,
                triangle_count: t_count,
                visible: true,
                min: sub_min,
                max: sub_max,
            });

            all_meshes.push(mesh);
        }

        // 加载伴生纹理并汇集
        let mut temp_state = ModelDocumentState {
            file_path: part_path.to_string_lossy().to_string(),
            model_name: target.category.to_string(),
            is_switch: is_switch_detected,
            meshes: Arc::new(Vec::new()),
            textures: std::collections::HashMap::new(),
            mesh_items: Vec::new(),
            total_vertices: 0,
            total_triangles: 0,
            bounding_box_min: [0.0; 3],
            bounding_box_max: [0.0; 3],
            camera: OrbitCamera::new(),
            render_mode: RenderMode::Shaded,
            show_grid: true,
            show_axes: true,
            show_bounds: false,
            show_stats: true,
            selected_mesh_idx: None,
            filter_query: String::new(),
            search_focused: false,
            is_orbiting: false,
            is_panning: false,
            last_mouse_pos: (0.0, 0.0),
        };
        load_companion_textures(&part_path.to_string_lossy(), &bfres, &mut temp_state);
        for (k, v) in temp_state.textures {
            textures.insert(k, v);
        }
    }

    if loaded_part_count == 0 {
        return Err("未能在指定搜索路径中找到任何 UMii 3D 模型部件 (.sbfres)".to_string());
    }

    if global_min[0] > global_max[0] {
        global_min = [-0.5, 0.0, -0.5];
        global_max = [0.5, 1.8, 0.5];
    }

    let mut camera = OrbitCamera::new();
    camera.fit_bounding_box(global_min, global_max);

    Ok(ModelDocumentState {
        file_path: format!("UMii://{}", data.name),
        model_name: format!("UMii_{}", data.name),
        is_switch: is_switch_detected,
        meshes: Arc::new(all_meshes),
        textures,
        mesh_items,
        total_vertices,
        total_triangles,
        bounding_box_min: global_min,
        bounding_box_max: global_max,
        camera,
        render_mode: RenderMode::Shaded,
        show_grid: true,
        show_axes: true,
        show_bounds: false,
        show_stats: true,
        selected_mesh_idx: None,
        filter_query: String::new(),
        search_focused: false,
        is_orbiting: false,
        is_panning: false,
        last_mouse_pos: (0.0, 0.0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_synthesize_npc_model() {
        let bumii_path = r"C:\Users\Administrator\.gemini\antigravity\brain\ba961cff-94a8-4946-992d-f2b7af722911\scratch\AncientAssistant001.bumii";
        if !std::path::Path::new(bumii_path).exists() {
            return;
        }

        let bytes = fs::read(bumii_path).unwrap();
        let umii_data = UmiiData::from_binary(&bytes).expect("Failed to parse AncientAssistant001.bumii");
        assert_eq!(umii_data.body_race, 0); // Hylia

        let model_dir = PathBuf::from(r"C:\Users\Administrator\Desktop\Tes\Romfs\Model");
        if model_dir.exists() {
            let res = synthesize_npc_model_sync(
                &umii_data,
                ModelPlatform::Switch,
                Some(&[model_dir]),
                None,
            );
            assert!(res.is_ok(), "UMii synthesis failed: {:?}", res.err());
            let doc = res.unwrap();
            assert!(!doc.meshes.is_empty(), "Mesh count should be > 0");
            assert!(doc.total_vertices > 0);
            assert!(doc.total_triangles > 0);
            assert_eq!(doc.model_name, "UMii_DefaultNPC");
        }
    }
}
