//! WGPU 网格顶点、索引与静态 Uniform/BindGroup 显存复用管理器

use super::pipeline::WgpuPipeline;
use super::types::{GpuUniforms, GpuVertex};
use rbfres_library::ExtractedMesh;
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::util::DeviceExt;

/// GPU 显存网格几何体对象 (持久持有显存顶点缓冲、索引缓冲、UBO 与 BindGroup)
pub struct GpuMeshGeometry {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: Option<Arc<wgpu::BindGroup>>,
    pub bound_texture_id: Option<usize>,
}

impl GpuMeshGeometry {
    /// 原位覆写常量 Uniform 缓冲内容 (0 内存分配，极速写入 host 映射区)
    pub fn update_uniforms(&self, queue: &wgpu::Queue, uniforms: &GpuUniforms) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(uniforms));
    }

    /// 获取或按需绑定构建当前网格的 BindGroup (贴图未变时 0 分配复用)
    pub fn get_or_create_bind_group(
        &mut self,
        device: &wgpu::Device,
        pipeline: &WgpuPipeline,
        tex_view: &Arc<wgpu::TextureView>,
    ) -> Option<Arc<wgpu::BindGroup>> {
        let tex_id = Arc::as_ptr(tex_view) as usize;
        if let Some(ref bg) = self.bind_group {
            if self.bound_texture_id == Some(tex_id) {
                return Some(bg.clone());
            }
        }

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Mesh Material BindGroup"),
            layout: &pipeline.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(tex_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&pipeline.sampler),
                },
            ],
        });

        let bg = Arc::new(bind_group);
        self.bound_texture_id = Some(tex_id);
        self.bind_group = Some(bg.clone());
        Some(bg)
    }
}

/// 网格显存缓冲缓存池 (以模型路径和网格索引联合为键)
#[derive(Default)]
pub struct GpuGeometryCache {
    cache: HashMap<(String, usize), GpuMeshGeometry>,
}

impl GpuGeometryCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// 获取或按需上传几何子网格至显存
    pub fn get_or_upload(
        &mut self,
        device: &wgpu::Device,
        model_path: &str,
        mesh_idx: usize,
        mesh: &ExtractedMesh,
    ) -> Option<&mut GpuMeshGeometry> {
        let key = (model_path.to_string(), mesh_idx);
        if !self.cache.contains_key(&key) {
            let geom = upload_mesh_geometry(device, mesh)?;
            self.cache.insert(key.clone(), geom);
        }
        self.cache.get_mut(&key)
    }

    /// 清空指定路径的显存网格缓存
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

/// 打包网格顶点与索引并分配 GPU 设备显存缓冲
fn upload_mesh_geometry(device: &wgpu::Device, mesh: &ExtractedMesh) -> Option<GpuMeshGeometry> {
    let v_len = mesh.positions.len();
    if v_len == 0 || mesh.indices.is_empty() {
        return None;
    }

    let norm_len = mesh.normals.len();
    let uv_len = mesh.uvs.len();

    let mut vertices = Vec::with_capacity(v_len);
    for i in 0..v_len {
        let pos = mesh.positions[i];
        let norm = if i < norm_len {
            [mesh.normals[i].x, mesh.normals[i].y, mesh.normals[i].z]
        } else {
            [0.0, 1.0, 0.0]
        };
        let uv = if i < uv_len {
            [mesh.uvs[i].x, mesh.uvs[i].y]
        } else {
            [0.0, 0.0]
        };

        vertices.push(GpuVertex {
            position: [pos.x, pos.y, pos.z],
            normal: norm,
            uv,
        });
    }

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Mesh Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_count = mesh.indices.len() as u32;
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Mesh Index Buffer"),
        contents: bytemuck::cast_slice(&mesh.indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Mesh Uniform Buffer"),
        size: std::mem::size_of::<GpuUniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    Some(GpuMeshGeometry {
        vertex_buffer,
        index_buffer,
        index_count,
        uniform_buffer,
        bind_group: None,
        bound_texture_id: None,
    })
}

