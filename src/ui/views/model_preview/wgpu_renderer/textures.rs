//! WGPU 纹理上传与缓存管理器

use super::context::WgpuContext;
use crate::model::model_preview::RgbaTexture;
use std::collections::HashMap;
use std::sync::Arc;

/// GPU 纹理生命周期与显存缓存池
pub struct GpuTextureManager {
    /// 默认纯白 1x1 占位贴图 (用于未绑定纹理的几何网格)
    pub default_white: Arc<wgpu::TextureView>,
    /// 已上传至 GPU 显存的纹理缓存表
    pub cache: HashMap<String, Arc<wgpu::TextureView>>,
}

impl GpuTextureManager {
    /// 初始化纹理管理器并预先烘焙默认 1x1 纯白贴图
    pub fn new(ctx: &WgpuContext) -> Option<Self> {
        let white_pixels = [255u8, 255u8, 255u8, 255u8];
        let default_white = upload_raw_texture(&ctx.device, &ctx.queue, 1, 1, &white_pixels)?;

        Some(Self {
            default_white,
            cache: HashMap::new(),
        })
    }

    /// 获取或按需上传 BFRES 漫反射贴图至 GPU 显存
    pub fn get_or_upload(
        &mut self,
        ctx: &WgpuContext,
        name: &str,
        texture: &RgbaTexture,
    ) -> Option<Arc<wgpu::TextureView>> {
        if let Some(view) = self.cache.get(name) {
            return Some(view.clone());
        }

        let view = upload_raw_texture(
            &ctx.device,
            &ctx.queue,
            texture.width,
            texture.height,
            &texture.rgba,
        )?;
        self.cache.insert(name.to_string(), view.clone());
        Some(view)
    }
}

/// 将原始 RGBA8 像素数组上传至 WGPU 2D 纹理并返回 TextureView
fn upload_raw_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
    pixels: &[u8],
) -> Option<Arc<wgpu::TextureView>> {
    let w = width.max(1);
    let h = height.max(1);

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Uploaded Model Texture"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(w * 4),
            rows_per_image: Some(h),
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );

    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    Some(Arc::new(view))
}

