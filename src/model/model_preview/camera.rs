//! 3D 轨道摄像机与交互式视口投影核心算法

use std::f32::consts::PI;

/// 摄像机快捷预设视角方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewPresetView {
    /// 前视视角 (正对着 Z 轴)
    Front,
    /// 后视视角
    #[allow(dead_code)]
    Back,
    /// 左侧视角 (正对着 X 轴)
    #[allow(dead_code)]
    Left,
    /// 右侧视角
    Right,
    /// 俯视顶视角 (正对着 Y 轴)
    Top,
    /// 等轴测透视视角 (Isometric)
    Isometric,
}

/// 3D 轨道环绕摄像机 (Orbit Camera)
/// 支持围绕目标中心点自由俯仰、环绕旋转、平移对齐与透视投影
#[derive(Debug, Clone, PartialEq)]
pub struct OrbitCamera {
    /// 水平环绕偏航角 (Yaw，弧度制)
    pub yaw: f32,
    /// 垂直俯仰角 (Pitch，弧度制，受限在 -89° 至 +89°)
    pub pitch: f32,
    /// 摄像机与聚焦点中心的目标距离
    pub distance: f32,
    /// 摄像机聚焦点目标坐标 [X, Y, Z]
    pub target: [f32; 3],
    /// 垂直视场角 (Field of View，弧度制，默认 45°)
    pub fov: f32,
    /// 允许的最小缩放距离
    pub min_distance: f32,
    /// 允许的最大缩放距离
    pub max_distance: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self::new()
    }
}

impl OrbitCamera {
    /// 构造默认参数的轨道摄像机 (斜俯视 25 度等轴透视)
    pub fn new() -> Self {
        Self {
            yaw: std::f32::consts::FRAC_PI_4, // 45 度
            pitch: 0.436332,                 // 25 度
            distance: 10.0,
            target: [0.0, 0.0, 0.0],
            fov: 45.0 * PI / 180.0,
            min_distance: 0.1,
            max_distance: 50000.0,
        }
    }

    /// 环绕旋转摄像机 (依据鼠标相对移动像素偏移量)
    pub fn orbit(&mut self, delta_x: f32, delta_y: f32) {
        let sensitivity = 0.008;
        self.yaw += delta_x * sensitivity;
        self.pitch -= delta_y * sensitivity;

        // 限制垂直俯仰角度，防止万向节死锁翻转 (Clamp within [-88°, +88°])
        let max_pitch = 88.0 * PI / 180.0;
        self.pitch = self.pitch.clamp(-max_pitch, max_pitch);

        // 规范化偏航角至 [0, 2π)
        let two_pi = 2.0 * PI;
        self.yaw = (self.yaw % two_pi + two_pi) % two_pi;
    }

    /// 平移视口中心聚焦点 (根据视口大小与相机距离自适应调整平移速率)
    pub fn pan(&mut self, delta_x: f32, delta_y: f32, viewport_height: f32) {
        let factor = (self.distance * (self.fov * 0.5).tan() * 2.0) / viewport_height.max(1.0);
        let (right, up, _) = self.compute_basis();

        for axis in 0..3 {
            self.target[axis] -= right[axis] * delta_x * factor;
            self.target[axis] += up[axis] * delta_y * factor;
        }
    }

    /// 缩放调整摄像机距离 (滚轮缩放，factor > 1 放大拉近，factor < 1 缩小远离)
    pub fn zoom(&mut self, factor: f32) {
        self.distance = (self.distance * factor).clamp(self.min_distance, self.max_distance);
    }

    /// 将摄像机居中对齐并自动适配包裹指定的 3D 轴对齐包围盒 (AABB)
    pub fn fit_bounding_box(&mut self, min: [f32; 3], max: [f32; 3]) {
        let center = [
            (min[0] + max[0]) * 0.5,
            (min[1] + max[1]) * 0.5,
            (min[2] + max[2]) * 0.5,
        ];
        let extent_x = (max[0] - min[0]).abs();
        let extent_y = (max[1] - min[1]).abs();
        let extent_z = (max[2] - min[2]).abs();

        let diagonal = (extent_x * extent_x + extent_y * extent_y + extent_z * extent_z).sqrt();
        let safe_diagonal = if diagonal < 0.001 { 5.0 } else { diagonal };

        self.target = center;
        // 计算使完整包围盒融入视锥范围的最佳摄像机距离
        let half_fov_tan = (self.fov * 0.5).tan().max(0.1);
        self.distance = ((safe_diagonal * 0.5) / half_fov_tan * 1.6).clamp(0.5, 20000.0);
    }

    /// 切换为快捷预设视角
    pub fn set_preset(&mut self, preset: PreviewPresetView) {
        match preset {
            PreviewPresetView::Front => {
                self.yaw = 0.0;
                self.pitch = 0.0;
            }
            PreviewPresetView::Back => {
                self.yaw = PI;
                self.pitch = 0.0;
            }
            PreviewPresetView::Left => {
                self.yaw = -PI * 0.5;
                self.pitch = 0.0;
            }
            PreviewPresetView::Right => {
                self.yaw = PI * 0.5;
                self.pitch = 0.0;
            }
            PreviewPresetView::Top => {
                self.yaw = 0.0;
                self.pitch = 88.0 * PI / 180.0;
            }
            PreviewPresetView::Isometric => {
                self.yaw = 45.0 * PI / 180.0;
                self.pitch = 30.0 * PI / 180.0;
            }
        }
    }

    /// 计算摄像机当前在世界坐标系下的视点坐标 (Eye Position)
    pub fn compute_eye(&self) -> [f32; 3] {
        let cos_pitch = self.pitch.cos();
        let sin_pitch = self.pitch.sin();
        let cos_yaw = self.yaw.cos();
        let sin_yaw = self.yaw.sin();

        [
            self.target[0] + self.distance * cos_pitch * sin_yaw,
            self.target[1] + self.distance * sin_pitch,
            self.target[2] + self.distance * cos_pitch * cos_yaw,
        ]
    }

    /// 计算摄像机的正交基准坐标系轴向量 (Right, Up, Forward)
    pub fn compute_basis(&self) -> ([f32; 3], [f32; 3], [f32; 3]) {
        let eye = self.compute_eye();
        // 前向向量 Forward: 从眼睛指向目标
        let forward_raw = [
            self.target[0] - eye[0],
            self.target[1] - eye[1],
            self.target[2] - eye[2],
        ];
        let forward = normalize_vec3(forward_raw);

        // 世界向上参考向量
        let world_up = if self.pitch.abs() > 87.0 * PI / 180.0 {
            [0.0, 0.0, -1.0]
        } else {
            [0.0, 1.0, 0.0]
        };

        // 右向向量 Right = Forward × WorldUp
        let right_raw = cross_vec3(forward, world_up);
        let right = normalize_vec3(right_raw);

        // 真实正交向上向量 Up = Right × Forward
        let up = cross_vec3(right, forward);

        (right, up, forward)
    }

    /// 构建预计算快速视口投影器 (消除每个顶点的三角函数与矩阵重复求值)
    pub fn projector(&self, viewport_w: f32, viewport_h: f32) -> FastCameraProjector {
        let eye = self.compute_eye();
        let (right, up, forward) = self.compute_basis();
        let tan_half_fov = (self.fov * 0.5).tan().max(0.001);
        let scale = (viewport_h.max(1.0) * 0.5) / tan_half_fov;

        FastCameraProjector {
            eye,
            forward,
            right,
            up,
            center_x: viewport_w * 0.5,
            center_y: viewport_h * 0.5,
            scale,
            near_z: 0.02,
        }
    }

    /// 将 3D 世界坐标单点透视投影至 2D 视口像素坐标
    /// 返回 `Some(([screen_x, screen_y], depth_z))`，若点位于相机近平面后方则返回 `None`
    pub fn project_point(
        &self,
        point: [f32; 3],
        viewport_w: f32,
        viewport_h: f32,
    ) -> Option<([f32; 2], f32)> {
        self.projector(viewport_w, viewport_h).project(point)
    }
}

/// 预计算的快速摄像机视口透视投影器
#[derive(Debug, Clone, Copy)]
pub struct FastCameraProjector {
    pub eye: [f32; 3],
    pub forward: [f32; 3],
    pub right: [f32; 3],
    pub up: [f32; 3],
    pub center_x: f32,
    pub center_y: f32,
    pub scale: f32,
    pub near_z: f32,
}

impl FastCameraProjector {
    /// 将 3D 世界坐标单点快速投影至 2D 视口像素坐标
    #[inline(always)]
    pub fn project(&self, p: [f32; 3]) -> Option<([f32; 2], f32)> {
        let dx = p[0] - self.eye[0];
        let dy = p[1] - self.eye[1];
        let dz = p[2] - self.eye[2];

        let cam_z = dx * self.forward[0] + dy * self.forward[1] + dz * self.forward[2];
        if cam_z <= self.near_z {
            return None;
        }

        let inv_z = 1.0 / cam_z;
        let cam_x = dx * self.right[0] + dy * self.right[1] + dz * self.right[2];
        let cam_y = dx * self.up[0] + dy * self.up[1] + dz * self.up[2];

        let sx = self.center_x + cam_x * inv_z * self.scale;
        let sy = self.center_y - cam_y * inv_z * self.scale;

        Some(([sx, sy], cam_z))
    }

    /// 快速测试 3D 轴对齐包围盒是否完全位于近平面后方 (仅在 8 个顶点全部在相机身后时剔除)
    pub fn is_all_behind_camera(&self, min: [f32; 3], max: [f32; 3]) -> bool {
        let corners = [
            [min[0], min[1], min[2]],
            [max[0], min[1], min[2]],
            [min[0], max[1], min[2]],
            [max[0], max[1], min[2]],
            [min[0], min[1], max[2]],
            [max[0], min[1], max[2]],
            [min[0], max[1], max[2]],
            [max[0], max[1], max[2]],
        ];

        corners.iter().all(|corner| {
            let dx = corner[0] - self.eye[0];
            let dy = corner[1] - self.eye[1];
            let dz = corner[2] - self.eye[2];
            let cam_z = dx * self.forward[0] + dy * self.forward[1] + dz * self.forward[2];
            cam_z <= self.near_z
        })
    }
}

/// 三维向量模长归一化
fn normalize_vec3(vector: [f32; 3]) -> [f32; 3] {
    let length = (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt();
    if length < 0.00001 {
        [0.0, 1.0, 0.0]
    } else {
        [vector[0] / length, vector[1] / length, vector[2] / length]
    }
}

/// 三维向量叉积
fn cross_vec3(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}
