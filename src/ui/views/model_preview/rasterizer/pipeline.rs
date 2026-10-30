//! 扫描带三角形多核像素光栅化管线

use super::types::PreparedTriangle;
use crate::model::model_preview::RgbaTexture;

/// 在指定水平扫描带内光栅化单个三角形 (直接无锁写入带内局部缓冲区)
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn rasterize_triangle_in_band(
    tri: &PreparedTriangle,
    texture: Option<&RgbaTexture>,
    base_color: [u8; 4],
    is_alpha_cutout: bool,
    buf_w: u32,
    band_y_start: u32,
    band_y_end: u32,
    depth_band: &mut [f32],
    color_band: &mut [u8],
) {
    let start_y = tri.min_y.max(band_y_start);
    let end_y = tri.max_y.min(band_y_end);
    if start_y > end_y {
        return;
    }

    let start_x = tri.min_x;
    let end_x = tri.max_x;
    let inv_denom = 1.0 / tri.denom;

    for y in start_y..=end_y {
        let py = y as f32 + 0.5;
        let local_y = y - band_y_start;
        let row_offset = (local_y * buf_w) as usize;

        for x in start_x..=end_x {
            let px = x as f32 + 0.5;

            let w0 = ((tri.s1[0] - px) * (tri.s2[1] - py)
                - (tri.s1[1] - py) * (tri.s2[0] - px))
                * inv_denom;
            let w1 = ((tri.s2[0] - px) * (tri.s0[1] - py)
                - (tri.s2[1] - py) * (tri.s0[0] - px))
                * inv_denom;
            let w2 = 1.0 - w0 - w1;

            if w0 < -0.001 || w1 < -0.001 || w2 < -0.001 {
                continue;
            }

            let inv_z = w0 * tri.inv_z0 + w1 * tri.inv_z1 + w2 * tri.inv_z2;
            let z = 1.0 / inv_z;

            let pixel_idx = row_offset + x as usize;
            if pixel_idx >= depth_band.len() || z >= depth_band[pixel_idx] {
                continue;
            }

            let intensity = w0 * tri.int0 + w1 * tri.int1 + w2 * tri.int2;

            let (r, g, b, a) = if let Some(tex) = texture {
                let u = (w0 * tri.uv0[0] * tri.inv_z0
                    + w1 * tri.uv1[0] * tri.inv_z1
                    + w2 * tri.uv2[0] * tri.inv_z2)
                    * z;
                let v = (w0 * tri.uv0[1] * tri.inv_z0
                    + w1 * tri.uv1[1] * tri.inv_z1
                    + w2 * tri.uv2[1] * tri.inv_z2)
                    * z;
                let c = tex.sample_rgba_raw(u, v);
                (c[0], c[1], c[2], c[3])
            } else {
                (base_color[0], base_color[1], base_color[2], base_color[3])
            };

            if is_alpha_cutout && a < 25 {
                continue;
            }

            depth_band[pixel_idx] = z;

            let out_idx = pixel_idx * 4;
            if out_idx + 3 < color_band.len() {
                color_band[out_idx] = ((b as f32 * intensity) as u32).min(255) as u8;
                color_band[out_idx + 1] = ((g as f32 * intensity) as u32).min(255) as u8;
                color_band[out_idx + 2] = ((r as f32 * intensity) as u32).min(255) as u8;
                color_band[out_idx + 3] = if is_alpha_cutout { a } else { 255 };
            }
        }
    }
}

