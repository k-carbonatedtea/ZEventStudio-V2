//! Actor 3D 模型 BFRES 二进制解析与文档状态装配

use std::fs;
use std::path::Path;
use std::sync::Arc;
use rbfres_library::{BfresFile, ExtractedMesh};

use super::actor_resolver::ActorModelTarget;
use super::camera::OrbitCamera;
use super::texture_loader::load_companion_textures;
use super::types::{PreviewMeshItem, RenderMode};
use super::ModelDocumentState;

/// 解析 BFRES 二进制并构建经过 Actor 单元过滤的模型文档状态
pub fn load_actor_model_document(
    target: &ActorModelTarget,
    model_path: &Path,
) -> Result<ModelDocumentState, String> {
    let raw_bytes = fs::read(model_path).map_err(|e| format!("读取模型文件失败: {}", e))?;
    let decompressed = if raw_bytes.len() >= 4 && &raw_bytes[0..4] == b"Yaz0" {
        roead::yaz0::decompress(&raw_bytes).map_err(|e| format!("Yaz0 解压失败: {}", e))?
    } else {
        raw_bytes
    };

    let bfres = BfresFile::from_bytes(&decompressed)
        .map_err(|e| format!("BFRES 模型解析失败: {:?}", e))?;

    let is_switch = bfres.is_switch;
    let path_str = model_path.to_string_lossy().to_string();
    let model_name = if !target.unit_names.is_empty() {
        target.unit_names[0].clone()
    } else if !bfres.name.is_empty() {
        bfres.name.clone()
    } else {
        target.folder.clone()
    };

    let mut total_vertices = 0;
    let mut total_triangles = 0;
    let mut all_meshes = Vec::new();
    let mut mesh_items = Vec::new();
    let mut visible_min = [f32::MAX, f32::MAX, f32::MAX];
    let mut visible_max = [f32::MIN, f32::MIN, f32::MIN];
    let mut global_min = [f32::MAX, f32::MAX, f32::MAX];
    let mut global_max = [f32::MIN, f32::MIN, f32::MIN];

    // 判断当前 BFRES 模型集合中是否存在匹配该 Actor 指定 UnitName 的模型子字典
    let has_matching_model = !target.unit_names.is_empty()
        && bfres.models.iter().any(|(name, _)| {
            target.unit_names.iter().any(|u| u.eq_ignore_ascii_case(name))
        });

    for (dict_model_name, model) in bfres.models.iter() {
        let is_target_model = if has_matching_model {
            target.unit_names.iter().any(|u| u.eq_ignore_ascii_case(dict_model_name))
        } else {
            true
        };

        let sub_meshes = ExtractedMesh::extract_from_model(model, bfres.byte_order);
        for mesh in sub_meshes {
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

            if is_target_model {
                for axis in 0..3 {
                    visible_min[axis] = visible_min[axis].min(sub_min[axis]);
                    visible_max[axis] = visible_max[axis].max(sub_max[axis]);
                }
            }

            mesh_items.push(PreviewMeshItem {
                name: mesh.name.clone(),
                material_name: mesh.material_name.clone().unwrap_or_else(|| "Default".to_string()),
                texture_name: mesh.texture_name.clone(),
                vertex_count: v_count,
                triangle_count: t_count,
                visible: is_target_model,
                min: sub_min,
                max: sub_max,
            });
            all_meshes.push(mesh);
        }
    }

    if global_min[0] > global_max[0] {
        global_min = [-1.0, 0.0, -1.0];
        global_max = [1.0, 2.0, 1.0];
    }

    let (fit_min, fit_max) = if visible_min[0] <= visible_max[0] {
        (visible_min, visible_max)
    } else {
        (global_min, global_max)
    };

    let mut camera = OrbitCamera::new();
    camera.fit_bounding_box(fit_min, fit_max);

    let mut doc_state = ModelDocumentState {
        file_path: path_str.clone(),
        model_name,
        is_switch,
        meshes: Arc::new(all_meshes),
        textures: std::collections::HashMap::new(),
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
    };

    load_companion_textures(&path_str, &bfres, &mut doc_state);

    Ok(doc_state)
}

