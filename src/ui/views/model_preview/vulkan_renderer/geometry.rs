//! Vulkan GPU 网格顶点、索引与静态 Uniform/Descriptor 显存复用管理器

use super::context::VulkanContext;
use super::pipeline::VulkanPipeline;
use super::types::{GpuUniforms, GpuVertex};
use rbfres_library::ExtractedMesh;
use std::collections::HashMap;
use std::sync::Arc;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
use vulkano::image::view::ImageView;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::pipeline::Pipeline;

/// GPU 显存网格几何体对象 (完全持久化持有显存顶点缓冲、索引缓冲、UBO 与 DescriptorSet)
pub struct GpuMeshGeometry {
    pub vertex_buffer: Subbuffer<[GpuVertex]>,
    pub index_buffer: Subbuffer<[u32]>,
    pub index_count: u32,
    pub uniform_buffer: Subbuffer<GpuUniforms>,
    pub descriptor_set: Option<Arc<DescriptorSet>>,
    pub bound_texture_id: Option<usize>,
}

impl GpuMeshGeometry {
    /// 原位覆写常量 Uniform 缓冲内容 (0 内存分配，极速写入 host 映射区)
    pub fn update_uniforms(&self, uniforms: GpuUniforms) -> Option<()> {
        let mut guard = self.uniform_buffer.write().ok()?;
        *guard = uniforms;
        Some(())
    }

    /// 获取或按需绑定构建当前网格的 DescriptorSet (贴图未变时 0 分配复用)
    pub fn get_or_create_descriptor_set(
        &mut self,
        ctx: &VulkanContext,
        pipeline: &VulkanPipeline,
        tex_view: Arc<ImageView>,
    ) -> Option<Arc<DescriptorSet>> {
        let tex_id = Arc::as_ptr(&tex_view) as usize;
        if let Some(ref set) = self.descriptor_set {
            if self.bound_texture_id == Some(tex_id) {
                return Some(set.clone());
            }
        }

        let set_layout = pipeline.pipeline.layout().set_layouts().first()?.clone();
        let descriptor_set = DescriptorSet::new(
            ctx.descriptor_set_allocator.clone(),
            set_layout,
            [
                WriteDescriptorSet::buffer(0, self.uniform_buffer.clone()),
                WriteDescriptorSet::image_view(1, tex_view),
                WriteDescriptorSet::sampler(2, pipeline.sampler.clone()),
            ],
            [],
        )
        .ok()?;

        self.bound_texture_id = Some(tex_id);
        self.descriptor_set = Some(descriptor_set.clone());
        Some(descriptor_set)
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
        ctx: &VulkanContext,
        model_path: &str,
        mesh_idx: usize,
        mesh: &ExtractedMesh,
    ) -> Option<&mut GpuMeshGeometry> {
        let key = (model_path.to_string(), mesh_idx);
        if !self.cache.contains_key(&key) {
            let geom = upload_mesh_geometry(ctx, mesh)?;
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
fn upload_mesh_geometry(ctx: &VulkanContext, mesh: &ExtractedMesh) -> Option<GpuMeshGeometry> {
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

    let vertex_buffer = Buffer::from_iter(
        ctx.memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        vertices,
    )
    .ok()?;

    let index_count = mesh.indices.len() as u32;
    let index_buffer = Buffer::from_iter(
        ctx.memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::INDEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        mesh.indices.clone(),
    )
    .ok()?;

    let uniform_buffer = Buffer::new_sized::<GpuUniforms>(
        ctx.memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::UNIFORM_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
    )
    .ok()?;

    Some(GpuMeshGeometry {
        vertex_buffer,
        index_buffer,
        index_count,
        uniform_buffer,
        descriptor_set: None,
        bound_texture_id: None,
    })
}
