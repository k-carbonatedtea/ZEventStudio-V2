use gpui::*;

/// ZEvent Studio 统一设计系统调色板规范 (Design System Theme Tokens)
///
/// UI 层级 / 元素       | 颜色名称   | HEX 色值   | 使用场景说明
/// --------------------|-----------|------------|--------------------------------------------
/// App Background      | 深石墨黑   | #0F1117    | 最底层背景、画布空白区
/// Panel Surface       | 蓝灰表面   | #161922    | 左侧边栏、顶部菜单栏、右侧属性面板、顶部工具栏
/// Card / Hover        | 浅层蓝灰   | #1E2330    | 欢迎页卡片底色、列表项 Hover 态、容器底色
/// Primary Accent      | 遗迹青蓝   | #00E5BA    | 核心选中高亮、连线、主要按钮、聚焦边框
/// Secondary Accent    | 琥珀金     | #F59E0B    | 节点标题指示、修改提示、Logo 闪电
/// Text Primary        | 亮白灰     | #F1F5F9    | 一级标题、选中文字
/// Text Secondary      | 蓝灰白     | #94A3B8    | 副标题、快捷键提示、路径文字
#[allow(dead_code)]
pub struct ThemeColors;

#[allow(dead_code)]
impl ThemeColors {
    /// App Background: 深石墨黑 #0F1117 - 最底层背景、画布空白区
    pub const APP_BG: Rgba = Rgba {
        r: 15.0 / 255.0,
        g: 17.0 / 255.0,
        b: 23.0 / 255.0,
        a: 1.0,
    };

    /// Panel Surface: 蓝灰表面 #161922 - 左侧边栏、顶部菜单栏、右侧属性面板、顶部工具栏
    pub const PANEL_SURFACE: Rgba = Rgba {
        r: 22.0 / 255.0,
        g: 25.0 / 255.0,
        b: 34.0 / 255.0,
        a: 1.0,
    };

    /// Card / Hover: 浅层蓝灰 #1E2330 - 欢迎页卡片底色、列表项 Hover 态
    pub const CARD_SURFACE: Rgba = Rgba {
        r: 30.0 / 255.0,
        g: 35.0 / 255.0,
        b: 48.0 / 255.0,
        a: 1.0,
    };

    /// Primary Accent: 遗迹青蓝 #00E5BA - 核心选中高亮、连线、主要按钮、聚焦边框
    pub const ACCENT_PRIMARY: Rgba = Rgba {
        r: 0.0 / 255.0,
        g: 229.0 / 255.0,
        b: 186.0 / 255.0,
        a: 1.0,
    };

    /// Primary Accent Alternate: 遗迹天蓝 #00C4FF
    pub const ACCENT_CYAN: Rgba = Rgba {
        r: 0.0 / 255.0,
        g: 196.0 / 255.0,
        b: 1.0,
        a: 1.0,
    };

    /// Secondary Accent: 琥珀金 #F59E0B - 节点标题指示、修改提示、Logo 闪电
    pub const ACCENT_AMBER: Rgba = Rgba {
        r: 245.0 / 255.0,
        g: 158.0 / 255.0,
        b: 11.0 / 255.0,
        a: 1.0,
    };

    /// Text Primary: 亮白灰 #F1F5F9 - 一级标题、选中文字
    pub const TEXT_PRIMARY: Rgba = Rgba {
        r: 241.0 / 255.0,
        g: 245.0 / 255.0,
        b: 249.0 / 255.0,
        a: 1.0,
    };

    /// Text Secondary: 蓝灰白 #94A3B8 - 副标题、快捷键提示、路径文字
    pub const TEXT_SECONDARY: Rgba = Rgba {
        r: 148.0 / 255.0,
        g: 163.0 / 255.0,
        b: 184.0 / 255.0,
        a: 1.0,
    };

    /// Panel Border: 蓝灰边框 #232838
    pub const PANEL_BORDER: Rgba = Rgba {
        r: 35.0 / 255.0,
        g: 40.0 / 255.0,
        b: 56.0 / 255.0,
        a: 1.0,
    };

    /// Card Border: 卡片浅边框 #2D3548
    pub const CARD_BORDER: Rgba = Rgba {
        r: 45.0 / 255.0,
        g: 53.0 / 255.0,
        b: 72.0 / 255.0,
        a: 1.0,
    };
}
