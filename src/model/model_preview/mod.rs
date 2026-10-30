//! BFRES 3D 模型预览文档状态与核心管理模型

pub mod camera;
pub mod texture_loader;
pub mod types;

#[allow(unused_imports)]
pub use camera::{FastCameraProjector, OrbitCamera, PreviewPresetView};
pub use texture_loader::load_companion_textures;
pub use types::{PreviewMeshItem, RenderMode, RgbaTexture};

use rbfres_library::{BfresFile, ExtractedMesh};

/// 3D 模型预览标签页专属文档状态
#[derive(Debug, Clone, PartialEq)]
pub struct ModelDocumentState {
    /// 物理文件路径或 SARC 虚拟路径
    pub file_path: String,
    /// 模型归档内部主名称
    pub model_name: String,
    /// 平台格式标识：true 为 Switch NX 64位格式，false 为 Wii U Cafe 32位格式
    pub is_switch: bool,
    /// 从 BFRES 模型字典中提炼抽取的几何网格列表 (使用 Arc 避免 UI 重绘高频深拷贝数十万顶点)
    pub meshes: std::sync::Arc<Vec<ExtractedMesh>>,
    /// 已加载解码的标准 RGBA8 纹理字典 (纹理名 -> RgbaTexture)
    pub textures: std::collections::HashMap<String, std::sync::Arc<RgbaTexture>>,
    /// 子网格 UI 条目元数据列表 (管理显隐与统计)
    pub mesh_items: Vec<PreviewMeshItem>,
    /// 模型全量总顶点数
    pub total_vertices: usize,
    /// 模型全量总三角面数
    pub total_triangles: usize,
    /// 全局轴对齐包围盒最小角点 [X, Y, Z]
    pub bounding_box_min: [f32; 3],
    /// 全局轴对齐包围盒最大角点 [X, Y, Z]
    pub bounding_box_max: [f32; 3],
    /// 3D 交互式视口环绕摄像机
    pub camera: OrbitCamera,
    /// 当前渲染表现形态模式
    pub render_mode: RenderMode,
    /// 是否在画布底层绘制三维地面空间参考网格
    pub show_grid: bool,
    /// 是否绘制三维世界坐标轴指示线 (XYZ)
    pub show_axes: bool,
    /// 是否绘制模型三维外接长方体包围盒
    pub show_bounds: bool,
    /// 是否在左上方浮层悬浮展示高精模型技术指标面板 (HUD)
    pub show_stats: bool,
    /// 当前被鼠标悬浮或单选聚焦的子网格索引
    pub selected_mesh_idx: Option<usize>,
    /// 侧边栏子网格快速检索过滤关键词
    pub filter_query: String,
    /// 搜索过滤输入框是否当前获得焦点
    pub search_focused: bool,
    /// 鼠标左键环绕旋转按压拖拽交互中
    pub is_orbiting: bool,
    /// 鼠标右键/中键视口平移按压拖拽交互中
    pub is_panning: bool,
    /// 上一帧鼠标按压或滑动的全局屏幕坐标 (X, Y)
    pub last_mouse_pos: (f32, f32),
}

impl ModelDocumentState {
    /// 从已解析的 `BfresFile` 资源归档提取全部网格构建模型预览文档状态
    pub fn from_bfres(file_path: &str, raw_name: &str, bfres: &BfresFile) -> Self {
        let is_switch = bfres.is_switch;
        let model_name = if !bfres.name.is_empty() {
            bfres.name.clone()
        } else if let Some((first_key, _)) = bfres.models.iter().next() {
            first_key.to_string()
        } else {
            raw_name.trim_end_matches(".sbfres").trim_end_matches(".bfres").to_string()
        };

        // 提取所有网格几何数据
        let meshes = bfres.extract_meshes();

        let mut total_vertices = 0;
        let mut total_triangles = 0;
        let mut min_bound = [f32::MAX, f32::MAX, f32::MAX];
        let mut max_bound = [f32::MIN, f32::MIN, f32::MIN];

        let mut mesh_items = Vec::with_capacity(meshes.len());

        for mesh in &meshes {
            let v_count = mesh.positions.len();
            let t_count = mesh.indices.len() / 3;
            total_vertices += v_count;
            total_triangles += t_count;

            let b_min = mesh.bounding_box.min();
            let b_max = mesh.bounding_box.max();
            let sub_min = [b_min.x, b_min.y, b_min.z];
            let sub_max = [b_max.x, b_max.y, b_max.z];

            for axis in 0..3 {
                min_bound[axis] = min_bound[axis].min(sub_min[axis]);
                max_bound[axis] = max_bound[axis].max(sub_max[axis]);
            }

            mesh_items.push(PreviewMeshItem {
                name: mesh.name.clone(),
                material_name: mesh.material_name.clone().unwrap_or_else(|| "Default".to_string()),
                texture_name: mesh.texture_name.clone(),
                vertex_count: v_count,
                triangle_count: t_count,
                visible: true,
                min: sub_min,
                max: sub_max,
            });
        }

        // 若无有效网格包围盒，设立保底范围 [-1, 1]
        if min_bound[0] > max_bound[0] {
            min_bound = [-1.0, 0.0, -1.0];
            max_bound = [1.0, 2.0, 1.0];
        }

        let mut camera = OrbitCamera::new();
        camera.fit_bounding_box(min_bound, max_bound);

        Self {
            file_path: file_path.to_string(),
            model_name,
            is_switch,
            meshes: std::sync::Arc::new(meshes),
            textures: std::collections::HashMap::new(),
            mesh_items,
            total_vertices,
            total_triangles,
            bounding_box_min: min_bound,
            bounding_box_max: max_bound,
            camera,
            render_mode: RenderMode::Shaded,
            show_grid: true,
            show_axes: true,
            show_bounds: false,
            show_stats: true,
            selected_mesh_idx: None,
            filter_query: String::new(),
            search_focused: false,
            is_orbiting: false,
            is_panning: false,
            last_mouse_pos: (0.0, 0.0),
        }
    }

    /// 切换指定子网格的可见性开关
    pub fn toggle_mesh_visibility(&mut self, idx: usize) {
        if let Some(item) = self.mesh_items.get_mut(idx) {
            item.visible = !item.visible;
        }
    }

    /// 批量设置所有子网格的可见性
    pub fn set_all_visibility(&mut self, visible: bool) {
        for item in &mut self.mesh_items {
            item.visible = visible;
        }
    }

    /// 重置并对焦摄像机至模型完整包围盒
    pub fn fit_camera(&mut self) {
        self.camera.fit_bounding_box(self.bounding_box_min, self.bounding_box_max);
    }

    /// 获取当前处于可见显示状态的子网格数量
    pub fn visible_meshes_count(&self) -> usize {
        self.mesh_items.iter().filter(|m| m.visible).count()
    }

    /// 获取当前处于可见状态的总顶点数
    pub fn visible_vertices_count(&self) -> usize {
        self.mesh_items.iter().filter(|m| m.visible).map(|m| m.vertex_count).sum()
    }

    /// 获取当前处于可见状态的总三角面数
    pub fn visible_triangles_count(&self) -> usize {
        self.mesh_items.iter().filter(|m| m.visible).map(|m| m.triangle_count).sum()
    }

    /// 获取模型各轴向尺寸跨度 [Width X, Height Y, Depth Z]
    pub fn dimensions(&self) -> [f32; 3] {
        [
            (self.bounding_box_max[0] - self.bounding_box_min[0]).abs(),
            (self.bounding_box_max[1] - self.bounding_box_min[1]).abs(),
            (self.bounding_box_max[2] - self.bounding_box_min[2]).abs(),
        ]
    }
}
