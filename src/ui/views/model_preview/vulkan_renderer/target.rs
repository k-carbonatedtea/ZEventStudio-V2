//! Vulkan 离屏渲染目标与显存读回缓冲管理

use super::context::VulkanContext;
use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};

/// 离屏渲染目标绑定包 (颜色缓冲、深度缓冲与 CPU 读取回读暂存缓冲)
pub struct RenderTargetBundle {
    pub width: u32,
    pub height: u32,
    pub color_image: Arc<Image>,
    pub framebuffer: Arc<Framebuffer>,
    pub staging_buffer: Subbuffer<[u8]>,
}

impl RenderTargetBundle {
    /// 确保或重新分配匹配视口尺寸的 GPU 离屏渲染目标
    pub fn ensure(
        current: &mut Option<Self>,
        ctx: &VulkanContext,
        render_pass: Arc<RenderPass>,
        target_w: u32,
        target_h: u32,
    ) -> Option<()> {
        if let Some(ref target) = *current {
            if target.width == target_w && target.height == target_h {
                return Some(());
            }
        }

        let color_image = Image::new(
            ctx.memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::B8G8R8A8_UNORM, // 对齐 Windows GPUI Direct3D 11 BGRA 图集格式
                extent: [target_w, target_h, 1],
                usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .ok()?;
        let color_view = ImageView::new_default(color_image.clone()).ok()?;

        let depth_image = Image::new(
            ctx.memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::D32_SFLOAT,
                extent: [target_w, target_h, 1],
                usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .ok()?;
        let depth_view = ImageView::new_default(depth_image).ok()?;

        let framebuffer = Framebuffer::new(
            render_pass,
            FramebufferCreateInfo {
                attachments: vec![color_view, depth_view],
                ..Default::default()
            },
        )
        .ok()?;

        let total_bytes = (target_w as u64) * (target_h as u64) * 4;
        let staging_buffer = Buffer::new_slice::<u8>(
            ctx.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::TRANSFER_DST,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
            total_bytes,
        )
        .ok()?;

        *current = Some(RenderTargetBundle {
            width: target_w,
            height: target_h,
            color_image,
            framebuffer,
            staging_buffer,
        });

        Some(())
    }
}

