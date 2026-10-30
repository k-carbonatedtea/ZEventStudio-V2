//! Vulkan GPU 网格顶点缓冲与索引缓冲管理器

use super::context::VulkanContext;
use super::types::GpuVertex;
use rbfres_library::ExtractedMesh;
use std::collections::HashMap;
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};

/// GPU 显存网格几何体对象
pub struct GpuMeshGeometry {
    pub vertex_buffer: Subbuffer<[GpuVertex]>,
    pub index_buffer: Subbuffer<[u32]>,
    pub index_count: u32,
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
    ) -> Option<&GpuMeshGeometry> {
        let key = (model_path.to_string(), mesh_idx);
        if !self.cache.contains_key(&key) {
            let geom = upload_mesh_geometry(ctx, mesh)?;
            self.cache.insert(key.clone(), geom);
        }
        self.cache.get(&key)
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

    Some(GpuMeshGeometry {
        vertex_buffer,
        index_buffer,
        index_count,
    })
}

