//! Native terrain splatting: the 256-square control map selects repeating
//! full-resolution materials instead of stretching a baked image over a tile.

use super::{context::WgpuContext, pipeline::WgpuPipeline};
use crate::model::model_preview::types::TerrainRenderData;
use std::{collections::HashMap, sync::Arc};
use wgpu::util::DeviceExt;

pub struct GpuTerrainMaterials {
    pub source: Arc<TerrainRenderData>,
    bindings: HashMap<String, Arc<wgpu::BindGroup>>,
}

pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Native terrain bindings"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Uint,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

impl GpuTerrainMaterials {
    pub fn new(
        ctx: &WgpuContext,
        pipeline: &WgpuPipeline,
        source: Arc<TerrainRenderData>,
    ) -> Option<Self> {
        let width = source.layers.iter().map(|t| t.width).max()?.max(1);
        let height = source.layers.iter().map(|t| t.height).max()?.max(1);
        let count = source.layers.len() as u32;
        let limits = ctx.device.limits();
        if width > limits.max_texture_dimension_2d
            || height > limits.max_texture_dimension_2d
            || count > limits.max_texture_array_layers
        {
            return None;
        }
        let mip_count = width.max(height).ilog2() + 1;
        let array = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Native terrain color array"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: count,
            },
            mip_level_count: mip_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        for (layer, image) in source.layers.iter().enumerate() {
            if image.rgba.len() != image.width as usize * image.height as usize * 4 {
                return None;
            }
            let resized;
            let pixels = if image.width == width && image.height == height {
                image.rgba.as_slice()
            } else {
                // Only fallback PNGs/placeholder layers may have differing sizes.
                let rgba = image::RgbaImage::from_raw(
                    image.width,
                    image.height,
                    image.rgba.as_ref().clone(),
                )?;
                resized = image::imageops::resize(
                    &rgba,
                    width,
                    height,
                    image::imageops::FilterType::Triangle,
                );
                resized.as_raw().as_slice()
            };
            ctx.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &array,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: layer as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                pixels,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(width * 4),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
        }
        generate_mips(ctx, &array, mip_count, count);
        let array_view = array.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Terrain detail anisotropic sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            anisotropy_clamp: 8,
            ..Default::default()
        });
        let mut params = [[10., 10., 0., 0.]; 256];
        for (dst, src) in params.iter_mut().zip(&source.material_params) {
            *dst = *src;
            dst[2] = dst[2].clamp(0., (count - 1) as f32);
        }
        let params = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Terrain UV and array layer table"),
                contents: bytemuck::cast_slice(&params),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let mut bindings = HashMap::with_capacity(source.tiles.len());
        for (name, map) in &source.tiles {
            if map.width == 0
                || map.height == 0
                || map.rgba.len() != map.width as usize * map.height as usize * 4
            {
                continue;
            }
            let control = ctx.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Terrain material IDs and blend weights"),
                size: wgpu::Extent3d {
                    width: map.width,
                    height: map.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Uint,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            ctx.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &control,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &map.rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(map.width * 4),
                    rows_per_image: Some(map.height),
                },
                wgpu::Extent3d {
                    width: map.width,
                    height: map.height,
                    depth_or_array_layers: 1,
                },
            );
            let control_view = control.create_view(&Default::default());
            let binding = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Native terrain tile materials"),
                layout: &pipeline.terrain_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&array_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&control_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: params.as_entire_binding(),
                    },
                ],
            });
            bindings.insert(name.clone(), Arc::new(binding));
        }
        Some(Self { source, bindings })
    }

    pub fn get_bind_group(&self, name: &str) -> Option<Arc<wgpu::BindGroup>> {
        self.bindings.get(name).cloned()
    }
}

// Generate the full pyramid on the GPU once. CPU downsampling 83 1024-square
// layers would put the loading bottleneck back into the terrain path.
fn generate_mips(ctx: &WgpuContext, texture: &wgpu::Texture, mip_count: u32, layers: u32) {
    if mip_count <= 1 {
        return;
    }
    let shader = ctx
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Terrain mip generator"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var filtering: sampler;
struct Vertex { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32>, };
@vertex fn vs(@builtin(vertex_index) i: u32) -> Vertex {
    let uv = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    var out: Vertex;
    out.position = vec4<f32>(uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
    out.uv = uv;
    return out;
}
@fragment fn fs(in: Vertex) -> @location(0) vec4<f32> {
    return textureSampleLevel(source, filtering, in.uv, 0.0);
}
"#
                .into(),
            ),
        });
    let layout = ctx
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Terrain mip bindings"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
    let pipeline_layout = ctx
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Terrain mip pipeline layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
    let pipeline = ctx
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Terrain mip pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
    let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Terrain mip filter"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Generate terrain mipmaps"),
        });
    for mip in 1..mip_count {
        for layer in 0..layers {
            let view = |level| {
                texture.create_view(&wgpu::TextureViewDescriptor {
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    base_mip_level: level,
                    mip_level_count: Some(1),
                    base_array_layer: layer,
                    array_layer_count: Some(1),
                    ..Default::default()
                })
            };
            let previous = view(mip - 1);
            let target = view(mip);
            let binding = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Terrain mip source"),
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&previous),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Terrain mip level"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &binding, &[]);
            pass.draw(0..3, 0..1);
        }
    }
    ctx.queue.submit(std::iter::once(encoder.finish()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::model_preview::{ModelDocumentState, ModelInstance, RgbaTexture};

    #[test]
    #[ignore = "requires a graphics adapter"]
    fn native_terrain_repeats_detail_instead_of_stretching_fallback() {
        let tile = uking_library::terrain::TerrainTile {
            name: "synthetic".into(),
            center: [0., 0.],
            size: 250.,
            min_height: 0.,
            max_height: 0.,
            has_water: false,
        };
        let field = uking_library::terrain::HeightField {
            tile,
            heights: vec![0.; 256 * 256],
        };
        let mut mesh = crate::model::map_editor::geometry::terrain_mesh(&field, None).unwrap();
        mesh.texture_name = Some("__terrain_test".into());
        let mut scene = ModelDocumentState::from_bfres(
            "native-terrain-test",
            "native-terrain-test",
            &Default::default(),
        );
        Arc::make_mut(&mut scene.mesh_items).push(crate::model::model_preview::PreviewMeshItem {
            name: mesh.name.clone(),
            material_name: String::new(),
            texture_name: mesh.texture_name.clone(),
            vertex_count: mesh.positions.len(),
            triangle_count: mesh.indices.len() / 3,
            visible: true,
            min: [-125., 0., -125.],
            max: [125., 0., 125.],
        });
        Arc::make_mut(&mut scene.meshes).push(mesh);
        scene.instances = Some(Arc::new(vec![ModelInstance {
            mesh_index: 0,
            transform: Default::default(),
            selected: false,
        }]));
        // The fallback intentionally has no detail. Both stripe colors must come
        // from the native array, including the UV repetition in the parameter table.
        Arc::make_mut(&mut scene.textures).insert(
            "__terrain_test".into(),
            Arc::new(RgbaTexture {
                width: 1,
                height: 1,
                rgba: Arc::new(vec![100, 100, 100, 255]),
            }),
        );
        let mut pixels = vec![0; 1024 * 1024 * 4];
        for y in 0..1024 {
            for x in 0..1024 {
                let color = if (x / 16) % 2 == 0 {
                    [240, 20, 20, 255]
                } else {
                    [20, 240, 20, 255]
                };
                pixels[(y * 1024 + x) * 4..(y * 1024 + x + 1) * 4].copy_from_slice(&color);
            }
        }
        scene.terrain = Some(Arc::new(TerrainRenderData {
            layers: vec![Arc::new(RgbaTexture {
                width: 1024,
                height: 1024,
                rgba: Arc::new(pixels),
            })],
            material_params: vec![[10., 10., 0., 0.]; 256],
            tiles: HashMap::from([(
                "__terrain_test".into(),
                Arc::new(RgbaTexture {
                    width: 256,
                    height: 256,
                    rgba: Arc::new(vec![0; 256 * 256 * 4]),
                }),
            )]),
        }));
        scene.camera.target = [0.; 3];
        scene.camera.distance = 20.;
        scene.camera.pitch = 88f32.to_radians();
        scene.camera.yaw = 0.;
        let image = super::super::render_wgpu_scene(&scene, 256., 256.).expect("native GPU render");
        let bytes = image.as_bytes(0).unwrap();
        let red = bytes
            .chunks_exact(4)
            .filter(|p| p[3] > 0 && p[2] > p[1].saturating_add(30))
            .count();
        let green = bytes
            .chunks_exact(4)
            .filter(|p| p[3] > 0 && p[1] > p[2].saturating_add(30))
            .count();
        assert!(
            red > 5000 && green > 5000,
            "native detail lost: {red} red / {green} green pixels"
        );
    }
}
