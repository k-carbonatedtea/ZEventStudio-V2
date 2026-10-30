//! Vulkan 渲染器核心数据类型与数学结构体

use vulkano::buffer::BufferContents;

/// 4x4 单精度浮点列主序矩阵 (用于顶点着色器变换)
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct Mat4(pub [f32; 16]);

impl Mat4 {
    /// 构造单位矩阵
    #[allow(dead_code)]
    pub fn identity() -> Self {
        Self([
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ])
    }

    /// 右手坐标系透视投影矩阵 (对齐 Vulkan 深度范围 [0.0, 1.0])
    pub fn perspective_rh(fov_y: f32, aspect: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / (fov_y * 0.5).tan().max(0.001);
        let range = near - far;

        Self([
            f / aspect.max(0.001), 0.0, 0.0, 0.0,
            0.0, -f, 0.0, 0.0, // Vulkan Y-down flip: 将 Y 轴反转，匹配屏幕向下坐标系
            0.0, 0.0, far / range, -1.0,
            0.0, 0.0, (near * far) / range, 0.0,
        ])
    }

    /// 右手坐标系观察视角变换矩阵 (Look-At View Matrix)
    pub fn look_at_rh(eye: [f32; 3], target: [f32; 3], up: [f32; 3]) -> Self {
        let forward = normalize([
            target[0] - eye[0],
            target[1] - eye[1],
            target[2] - eye[2],
        ]);
        let right = normalize(cross(forward, up));
        let true_up = cross(right, forward);

        Self([
            right[0], true_up[0], -forward[0], 0.0,
            right[1], true_up[1], -forward[1], 0.0,
            right[2], true_up[2], -forward[2], 0.0,
            -dot(right, eye), -dot(true_up, eye), dot(forward, eye), 1.0,
        ])
    }

    /// 矩阵乘法 (self * rhs)
    pub fn mul(&self, rhs: &Mat4) -> Mat4 {
        let mut out = [0.0f32; 16];
        for col in 0..4 {
            for row in 0..4 {
                let mut sum = 0.0;
                for k in 0..4 {
                    sum += self.0[k * 4 + row] * rhs.0[col * 4 + k];
                }
                out[col * 4 + row] = sum;
            }
        }
        Mat4(out)
    }

    /// 输出为二维数组格式用于 Uniform 传递
    pub fn to_cols_array_2d(self) -> [[f32; 4]; 4] {
        [
            [self.0[0], self.0[1], self.0[2], self.0[3]],
            [self.0[4], self.0[5], self.0[6], self.0[7]],
            [self.0[8], self.0[9], self.0[10], self.0[11]],
            [self.0[12], self.0[13], self.0[14], self.0[15]],
        ]
    }
}

/// GPU 顶点输入数据格式
#[derive(Clone, Copy, Debug, BufferContents, PartialEq)]
#[repr(C)]
pub struct GpuVertex {
    /// 顶点模型局部 3D 坐标
    pub position: [f32; 3],
    /// 顶点法线向量
    pub normal: [f32; 3],
    /// 漫反射贴图 UV 纹理坐标
    pub uv: [f32; 2],
}

/// GPU 常量 Uniform 缓冲结构体 (对齐 16 字节边界)
#[derive(Clone, Copy, Debug, BufferContents, PartialEq)]
#[repr(C)]
pub struct GpuUniforms {
    /// 模型-视图-投影综合变换矩阵 (MVP)
    pub mvp: [[f32; 4]; 4],
    /// 主光源归一化方向向量 [X, Y, Z, 0.0]
    pub sun_dir: [f32; 4],
    /// 子网格材质基础着色 [R, G, B, A]
    pub base_color: [f32; 4],
    /// 特性开关标志位 [has_texture, is_alpha_cutout, is_selected, 0]
    pub flags: [u32; 4],
}

/// 静态渲染缓存键 (用于视口未变动时瞬时返回画面)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VulkanCacheKey {
    pub model_path: String,
    pub yaw_bits: u32,
    pub pitch_bits: u32,
    pub distance_bits: u32,
    pub target_bits: [u32; 3],
    pub fov_bits: u32,
    pub view_w: u32,
    pub view_h: u32,
    pub render_mode: crate::model::model_preview::RenderMode,
    pub selected_mesh_idx: Option<usize>,
    pub visibility_hash: u64,
}

#[inline(always)]
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline(always)]
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline(always)]
fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 0.00001 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}
