//! Vulkan 渲染管线、RenderPass 与通用采样器创建配置

use super::shaders::create_shader_module;
use super::types::GpuVertex;
use std::collections::BTreeMap;
use std::sync::Arc;
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::image::sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo, SamplerMipmapMode};
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::depth_stencil::{DepthState, DepthStencilState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::{CullMode, FrontFace, RasterizationState};
use vulkano::pipeline::graphics::subpass::PipelineSubpassType;
use vulkano::pipeline::graphics::vertex_input::{
    VertexInputAttributeDescription, VertexInputBindingDescription, VertexInputRate, VertexInputState,
};
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::pipeline::graphics::{GraphicsPipeline, GraphicsPipelineCreateInfo};
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{DynamicState, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::render_pass::{RenderPass, Subpass};

/// 封装 Vulkan 图形管线核心渲染基础设施
pub struct VulkanPipeline {
    pub render_pass: Arc<RenderPass>,
    pub pipeline: Arc<GraphicsPipeline>,
    pub sampler: Arc<Sampler>,
}

impl VulkanPipeline {
    /// 构造包含深度测试、双面渲染与 Alpha 混合的通用 3D 渲染管线
    pub fn new(device: Arc<Device>) -> Option<Self> {
        let render_pass = vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    format: Format::B8G8R8A8_UNORM, // 对齐 Windows GPUI Direct3D 11 BGRA 图集格式
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
                depth: {
                    format: Format::D32_SFLOAT,
                    samples: 1,
                    load_op: Clear,
                    store_op: DontCare,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {depth},
            },
        ).ok()?;

        let shader = create_shader_module(device.clone())?;
        let vs = shader.entry_point("vs_main")?;
        let fs = shader.entry_point("fs_main")?;

        let binding_desc = VertexInputBindingDescription {
            stride: std::mem::size_of::<GpuVertex>() as u32,
            input_rate: VertexInputRate::Vertex,
            ..Default::default()
        };
        let mut bindings = BTreeMap::new();
        bindings.insert(0, binding_desc);

        let mut attributes = BTreeMap::new();
        attributes.insert(0, VertexInputAttributeDescription {
            binding: 0,
            format: Format::R32G32B32_SFLOAT,
            offset: 0,
            ..Default::default()
        });
        attributes.insert(1, VertexInputAttributeDescription {
            binding: 0,
            format: Format::R32G32B32_SFLOAT,
            offset: 12,
            ..Default::default()
        });
        attributes.insert(2, VertexInputAttributeDescription {
            binding: 0,
            format: Format::R32G32_SFLOAT,
            offset: 24,
            ..Default::default()
        });

        let vertex_input_state = VertexInputState::new()
            .bindings(bindings)
            .attributes(attributes);

        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let layout_info = PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())
            .ok()?;
        let layout = PipelineLayout::new(device.clone(), layout_info).ok()?;

        let subpass = Subpass::from(render_pass.clone(), 0)?;

        let pipeline = GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState::default()),
                rasterization_state: Some(RasterizationState {
                    cull_mode: CullMode::None, // 双面渲染支持
                    front_face: FrontFace::CounterClockwise,
                    ..Default::default()
                }),
                depth_stencil_state: Some(DepthStencilState {
                    depth: Some(DepthState::simple()),
                    ..Default::default()
                }),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default(),
                )),
                dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                subpass: Some(PipelineSubpassType::BeginRenderPass(subpass)),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        ).ok()?;

        let sampler = Sampler::new(
            device,
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                mipmap_mode: SamplerMipmapMode::Linear,
                address_mode: [
                    SamplerAddressMode::Repeat,
                    SamplerAddressMode::Repeat,
                    SamplerAddressMode::Repeat,
                ],
                ..Default::default()
            },
        ).ok()?;

        Some(Self {
            render_pass,
            pipeline,
            sampler,
        })
    }
}

