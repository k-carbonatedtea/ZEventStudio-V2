//! WGPU 离屏渲染目标与显存读回缓冲管理

/// 离屏渲染目标绑定包 (颜色缓冲、深度缓冲与 CPU 读取回读暂存缓冲)
pub struct RenderTargetBundle {
    pub width: u32,
    pub height: u32,
    pub padded_bytes_per_row: u32,
    pub unpadded_bytes_per_row: u32,
    pub color_texture: wgpu::Texture,
    pub color_view: wgpu::TextureView,
    #[allow(dead_code)]
    pub depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,
    pub staging_buffer: wgpu::Buffer,
}

impl RenderTargetBundle {
    /// 确保或重新分配匹配视口尺寸的 GPU 离屏渲染目标
    pub fn ensure(
        current: &mut Option<Self>,
        device: &wgpu::Device,
        target_w: u32,
        target_h: u32,
    ) -> Option<()> {
        if let Some(ref target) = *current {
            if target.width == target_w && target.height == target_h {
                return Some(());
            }
        }

        let unpadded_bytes_per_row = target_w * 4;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;
        let total_bytes = (padded_bytes_per_row as u64) * (target_h as u64);

        let color_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Offscreen Color Texture"),
            size: wgpu::Extent3d {
                width: target_w,
                height: target_h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm, // 对齐 Windows GPUI Direct3D 11 BGRA 图集格式
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let color_view = color_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Offscreen Depth Texture"),
            size: wgpu::Extent3d {
                width: target_w,
                height: target_h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Offscreen Readback Staging Buffer"),
            size: total_bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        *current = Some(RenderTargetBundle {
            width: target_w,
            height: target_h,
            padded_bytes_per_row,
            unpadded_bytes_per_row,
            color_texture,
            color_view,
            depth_texture,
            depth_view,
            staging_buffer,
        });

        Some(())
    }
}
