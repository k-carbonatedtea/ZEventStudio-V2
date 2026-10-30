//! Vulkan GPU 纹理上传与缓存管理器

use super::context::VulkanContext;
use crate::model::model_preview::RgbaTexture;
use std::collections::HashMap;
use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferToImageInfo,
    PrimaryCommandBufferAbstract,
};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::sync::GpuFuture;

/// GPU 纹理生命周期与显存缓存池
pub struct GpuTextureManager {
    /// 默认纯白 1x1 占位贴图 (用于未绑定纹理的几何网格)
    pub default_white: Arc<ImageView>,
    /// 已上传至 GPU 显存的纹理缓存表
    pub cache: HashMap<String, Arc<ImageView>>,
}

impl GpuTextureManager {
    /// 初始化纹理管理器并预先烘焙默认 1x1 纯白贴图
    pub fn new(ctx: &VulkanContext) -> Option<Self> {
        let white_pixels = [255u8, 255u8, 255u8, 255u8];
        let default_white = upload_raw_texture(ctx, 1, 1, &white_pixels)?;

        Some(Self {
            default_white,
            cache: HashMap::new(),
        })
    }

    /// 获取或按需上传 BFRES 漫反射贴图至 GPU 显存
    pub fn get_or_upload(
        &mut self,
        ctx: &VulkanContext,
        name: &str,
        texture: &RgbaTexture,
    ) -> Option<Arc<ImageView>> {
        if let Some(view) = self.cache.get(name) {
            return Some(view.clone());
        }

        let view = upload_raw_texture(ctx, texture.width, texture.height, &texture.rgba)?;
        self.cache.insert(name.to_string(), view.clone());
        Some(view)
    }
}

/// 将原始 RGBA8 像素数组上传至 Vulkan 2D 纹理并返回 ImageView
fn upload_raw_texture(
    ctx: &VulkanContext,
    width: u32,
    height: u32,
    pixels: &[u8],
) -> Option<Arc<ImageView>> {
    let staging_buffer = Buffer::from_iter(
        ctx.memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_SRC,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        pixels.iter().copied(),
    )
    .ok()?;

    let image = Image::new(
        ctx.memory_allocator.clone(),
        ImageCreateInfo {
            image_type: ImageType::Dim2d,
            format: Format::R8G8B8A8_UNORM,
            extent: [width.max(1), height.max(1), 1],
            usage: ImageUsage::SAMPLED | ImageUsage::TRANSFER_DST,
            ..Default::default()
        },
        AllocationCreateInfo::default(),
    )
    .ok()?;

    let mut builder = AutoCommandBufferBuilder::primary(
        ctx.command_buffer_allocator.clone(),
        ctx.queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .ok()?;

    builder
        .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
            staging_buffer,
            image.clone(),
        ))
        .ok()?;

    let command_buffer = builder.build().ok()?;
    command_buffer
        .execute(ctx.queue.clone())
        .ok()?
        .then_signal_fence_and_flush()
        .ok()?
        .wait(None)
        .ok()?;

    ImageView::new_default(image).ok()
}
