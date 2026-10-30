//! 3D 软光栅化渲染引擎 (分块多线程并行光栅化、视锥剔除与静态帧缓存)

pub mod batch;
pub mod pipeline;
pub mod types;

use batch::prepare_mesh_batch;
use crate::model::model_preview::ModelDocumentState;
use gpui::RenderImage;
use image::{Frame, ImageBuffer, Rgba};
use pipeline::rasterize_triangle_in_band;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use types::{CachedImage, PreparedMeshBatch, RenderCacheKey};

const BAND_HEIGHT: u32 = 32;

static RENDER_CACHE: Mutex<Option<CachedImage>> = Mutex::new(None);

/// 执行 3D 场景软光栅化渲染，输出带 Z-Buffer 的高质量 RGBA 图像 (100% 全分辨率超清原生渲染)
pub fn rasterize_scene(
    state: &ModelDocumentState,
    view_w: f32,
    view_h: f32,
) -> Option<Arc<RenderImage>> {
    let target_w = (view_w.round() as u32).clamp(16, 2560);
    let target_h = (view_h.round() as u32).clamp(16, 1600);

    // 1. 检查静态渲染缓存 (画面未变动直接瞬时返回，零画质损失)
    let cache_key = compute_cache_key(state, target_w, target_h);
    if let Ok(guard) = RENDER_CACHE.lock() {
        if let Some(ref cached) = *guard {
            if cached.key == cache_key {
                return Some(cached.image.clone());
            }
        }
    }

    let projector = state.camera.projector(target_w as f32, target_h as f32);
    let sun_dir = [0.35f32, 0.80f32, 0.45f32];
    let sun_len = (sun_dir[0] * sun_dir[0] + sun_dir[1] * sun_dir[1] + sun_dir[2] * sun_dir[2]).sqrt();
    let sun_norm = [sun_dir[0] / sun_len, sun_dir[1] / sun_len, sun_dir[2] / sun_len];

    // 2. 预计算所有可见子网格的顶点投影与三角面包围盒
    let batches: Vec<PreparedMeshBatch> = state
        .meshes
        .iter()
        .enumerate()
        .filter_map(|(idx, mesh)| {
            prepare_mesh_batch(mesh, idx, state, &projector, sun_norm, target_w, target_h)
        })
        .collect();

    let total_pixels = (target_w * target_h) as usize;
    let mut color_buffer = vec![0u8; total_pixels * 4];
    let mut depth_buffer = vec![f32::INFINITY; total_pixels];

    // 3. 水平扫描带分块并行光栅化 (Rayon 多核并发)
    let band_pixel_count = (BAND_HEIGHT * target_w) as usize;
    let band_color_count = (BAND_HEIGHT * target_w * 4) as usize;

    depth_buffer
        .par_chunks_mut(band_pixel_count)
        .zip(color_buffer.par_chunks_mut(band_color_count))
        .enumerate()
        .for_each(|(band_idx, (depth_band, color_band))| {
            let band_y_start = (band_idx as u32) * BAND_HEIGHT;
            let band_rows = (depth_band.len() as u32) / target_w;
            let band_y_end = band_y_start + band_rows.saturating_sub(1);

            for batch in &batches {
                // 网格粗剔除
                if batch.min_y > band_y_end || batch.max_y < band_y_start {
                    continue;
                }

                for tri in &batch.triangles {
                    // 三角形细剔除
                    if tri.min_y > band_y_end || tri.max_y < band_y_start {
                        continue;
                    }

                    rasterize_triangle_in_band(
                        tri,
                        batch.texture,
                        batch.base_color,
                        batch.is_alpha_cutout,
                        target_w,
                        band_y_start,
                        band_y_end,
                        depth_band,
                        color_band,
                    );
                }
            }
        });

    let img_buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(target_w, target_h, color_buffer)?;
    let frame = Frame::new(img_buffer);
    let render_image = Arc::new(RenderImage::new(vec![frame]));

    // 4. 写入静态渲染缓存
    if let Ok(mut guard) = RENDER_CACHE.lock() {
        *guard = Some(CachedImage {
            key: cache_key,
            image: render_image.clone(),
        });
    }

    Some(render_image)
}

/// 计算当前模型状态的快照哈希缓存键
fn compute_cache_key(
    state: &ModelDocumentState,
    target_w: u32,
    target_h: u32,
) -> RenderCacheKey {
    let mut vis_hash = 0u64;
    for (i, item) in state.mesh_items.iter().enumerate() {
        if item.visible {
            vis_hash = vis_hash.wrapping_add(1u64 << (i % 64));
        }
    }

    RenderCacheKey {
        model_path: state.file_path.clone(),
        yaw_bits: state.camera.yaw.to_bits(),
        pitch_bits: state.camera.pitch.to_bits(),
        distance_bits: state.camera.distance.to_bits(),
        target_bits: [
            state.camera.target[0].to_bits(),
            state.camera.target[1].to_bits(),
            state.camera.target[2].to_bits(),
        ],
        fov_bits: state.camera.fov.to_bits(),
        view_w: target_w,
        view_h: target_h,
        render_mode: state.render_mode,
        selected_mesh_idx: state.selected_mesh_idx,
        visibility_hash: vis_hash,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use rbfres_library::BfresFile;

    #[test]
    fn test_full_res_rasterize_dungeon085() {
        let pack_path = r"C:\Game\BOTW\01007EF00011E000\romfs\Pack\Dungeon085.pack";
        if !Path::new(pack_path).exists() {
            return;
        }
        let virtual_path = format!("SARC:{}//Model/DgnMrgPrt_Dungeon085.sbfres", pack_path);
        let Ok(bytes) = crate::model::file_tree::read_file_from_path(&virtual_path) else { return; };
        let decomp = crate::model::file_converter::decompress_if_yaz0(&bytes);
        let Ok(bfres) = BfresFile::from_bytes(&decomp) else { return; };
        let mut state = ModelDocumentState::from_bfres(&virtual_path, "DgnMrgPrt_Dungeon085.sbfres", &bfres);
        crate::model::model_preview::load_companion_textures(&virtual_path, &bfres, &mut state);

        let start = std::time::Instant::now();
        let image = rasterize_scene(&state, 1280.0, 720.0);
        let elapsed = start.elapsed();
        println!(">>> 200,000 Triangles Native 720p Render Time: {:?}", elapsed);

        assert!(image.is_some(), "Rasterized image should be produced");
    }
}

