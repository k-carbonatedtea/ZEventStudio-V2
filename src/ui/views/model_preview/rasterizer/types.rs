//! 3D 软光栅化内部图元与缓存数据类型

use crate::model::model_preview::{RenderMode, RgbaTexture};
use gpui::RenderImage;
use std::sync::Arc;

/// 光栅化渲染缓存键 (用于静止画面零开销复用)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderCacheKey {
    pub model_path: String,
    pub yaw_bits: u32,
    pub pitch_bits: u32,
    pub distance_bits: u32,
    pub target_bits: [u32; 3],
    pub fov_bits: u32,
    pub view_w: u32,
    pub view_h: u32,
    pub render_mode: RenderMode,
    pub selected_mesh_idx: Option<usize>,
    pub visibility_hash: u64,
}

pub struct CachedImage {
    pub key: RenderCacheKey,
    pub image: Arc<RenderImage>,
}

/// 预计算的单个三角面渲染参数
#[derive(Clone, Copy)]
pub struct PreparedTriangle {
    pub s0: [f32; 2],
    pub s1: [f32; 2],
    pub s2: [f32; 2],
    pub inv_z0: f32,
    pub inv_z1: f32,
    pub inv_z2: f32,
    pub int0: f32,
    pub int1: f32,
    pub int2: f32,
    pub uv0: [f32; 2],
    pub uv1: [f32; 2],
    pub uv2: [f32; 2],
    pub denom: f32,
    pub min_x: u32,
    pub max_x: u32,
    pub min_y: u32,
    pub max_y: u32,
}

/// 预计算的子网格图元渲染批次
pub struct PreparedMeshBatch<'a> {
    pub min_y: u32,
    pub max_y: u32,
    pub texture: Option<&'a RgbaTexture>,
    pub base_color: [u8; 4],
    pub is_alpha_cutout: bool,
    pub triangles: Vec<PreparedTriangle>,
}

