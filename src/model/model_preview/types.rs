//! 模型预览基础枚举类型与元数据定义

/// 3D 视口渲染渲染表现形态模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderMode {
    /// 实体受光平直着色 (Flat Directional Shading)
    #[default]
    Shaded,
    /// 纯矢量线框模式 (Wireframe)
    Wireframe,
    /// 实体着色叠加精细线框 (Shaded with Wireframe overlay)
    ShadedWireframe,
    /// 顶点散点云模式 (Point Cloud)
    Points,
}

impl RenderMode {
    /// 获取模式的简明图标
    pub fn icon(&self) -> &'static str {
        match self {
            RenderMode::Shaded => "🎨",
            RenderMode::Wireframe => "🕸️",
            RenderMode::ShadedWireframe => "💎",
            RenderMode::Points => "✨",
        }
    }

    /// 获取模式的多语言键后缀
    pub fn i18n_key(&self) -> &'static str {
        match self {
            RenderMode::Shaded => "model_preview.mode_shaded",
            RenderMode::Wireframe => "model_preview.mode_wireframe",
            RenderMode::ShadedWireframe => "model_preview.mode_shaded_wire",
            RenderMode::Points => "model_preview.mode_points",
        }
    }
}

/// 单个子网格元数据条目 (用于侧边栏分层与显隐控制)
#[derive(Debug, Clone, PartialEq)]
pub struct PreviewMeshItem {
    /// 子网格名称
    pub name: String,
    /// 关联材质标识名
    pub material_name: String,
    /// 绑定的主漫反射贴图名称
    pub texture_name: Option<String>,
    /// 包含的顶点数量
    pub vertex_count: usize,
    /// 包含的三角面数量
    pub triangle_count: usize,
    /// 是否在画布中可见
    pub visible: bool,
    /// 局部轴对齐包围盒最小端点 [X, Y, Z]
    pub min: [f32; 3],
    /// 局部轴对齐包围盒最大端点 [X, Y, Z]
    pub max: [f32; 3],
}

/// 解码后的标准 32 位 RGBA 像素纹理图像
#[derive(Debug, Clone, PartialEq)]
pub struct RgbaTexture {
    /// 图像宽度
    pub width: u32,
    /// 图像高度
    pub height: u32,
    /// 原始 RGBA8 像素切片 (每个像素 4 字节 [R, G, B, A])
    pub rgba: std::sync::Arc<Vec<u8>>,
}

impl RgbaTexture {
    /// 基于 UV 坐标采样原始 RGBA8 像素值 [R, G, B, A] (采用快速分支消除 rem_euclid 耗时)
    #[inline(always)]
    pub fn sample_rgba_raw(&self, u: f32, v: f32) -> [u8; 4] {
        if self.width == 0 || self.height == 0 || self.rgba.is_empty() {
            return [180, 180, 180, 255];
        }

        let mut u_frac = u.fract();
        if u_frac < 0.0 {
            u_frac += 1.0;
        }
        let mut v_frac = v.fract();
        if v_frac < 0.0 {
            v_frac += 1.0;
        }

        let px = (u_frac * (self.width - 1) as f32) as usize;
        let py = (v_frac * (self.height - 1) as f32) as usize;

        let idx = (py * self.width as usize + px) * 4;
        if idx + 4 <= self.rgba.len() {
            [
                self.rgba[idx],
                self.rgba[idx + 1],
                self.rgba[idx + 2],
                self.rgba[idx + 3],
            ]
        } else {
            [180, 180, 180, 255]
        }
    }

    /// 基于 UV 坐标 (0.0..1.0) 执行双线性/邻近采样，返回 gpui::Hsla 颜色
    #[allow(dead_code)]
    pub fn sample_uv(&self, u: f32, v: f32) -> gpui::Hsla {
        if self.width == 0 || self.height == 0 || self.rgba.is_empty() {
            return gpui::hsla(0.0, 0.0, 0.5, 1.0);
        }

        let mut u_wrap = u.fract();
        if u_wrap < 0.0 {
            u_wrap += 1.0;
        }
        let mut v_wrap = v.fract();
        if v_wrap < 0.0 {
            v_wrap += 1.0;
        }

        let px = ((u_wrap * (self.width as f32 - 1.0)).round() as u32).min(self.width - 1) as usize;
        let py = ((v_wrap * (self.height as f32 - 1.0)).round() as u32).min(self.height - 1) as usize;

        let idx = (py * self.width as usize + px) * 4;
        if idx + 4 <= self.rgba.len() {
            let r = self.rgba[idx] as f32 / 255.0;
            let g = self.rgba[idx + 1] as f32 / 255.0;
            let b = self.rgba[idx + 2] as f32 / 255.0;
            let a = self.rgba[idx + 3] as f32 / 255.0;
            gpui::Rgba { r, g, b, a }.into()
        } else {
            gpui::hsla(0.0, 0.0, 0.5, 1.0)
        }
    }
}

