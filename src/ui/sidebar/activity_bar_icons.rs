use gpui::*;

/// 渲染资源管理器矢量图标 (VS Code 风格重叠双文件页)
pub fn render_explorer_icon(color: Rgba, pal: &crate::ui::theme::ThemePalette) -> AnyElement {
    div()
        .size(px(22.0))
        .relative()
        // 底层文件页
        .child(
            div()
                .absolute()
                .top(px(1.5))
                .left(px(1.5))
                .w(px(13.0))
                .h(px(15.0))
                .rounded(px(1.5))
                .border_1()
                .border_color(color),
        )
        // 顶层文件页
        .child(
            div()
                .absolute()
                .top(px(5.0))
                .left(px(5.5))
                .w(px(14.0))
                .h(px(15.0))
                .bg(pal.panel_surface)
                .rounded(px(1.5))
                .border_1()
                .border_color(color),
        )
        // 顶层右上角折角切线
        .child(
            div()
                .absolute()
                .top(px(5.0))
                .right(px(2.5))
                .size(px(4.0))
                .border_b_1()
                .border_l_1()
                .border_color(color),
        )
        .into_any_element()
}

/// 渲染全局搜索矢量图标 (VS Code 风格 45度倾斜放大镜)
pub fn render_search_icon(color: Rgba) -> AnyElement {
    div()
        .size(px(22.0))
        .relative()
        // 放大镜圆形镜面
        .child(
            div()
                .absolute()
                .top(px(2.0))
                .left(px(2.0))
                .size(px(12.5))
                .rounded_full()
                .border_1()
                .border_color(color),
        )
        // 放大镜 45 度斜手柄 (平滑步进像素)
        .child(
            div()
                .absolute()
                .top(px(12.0))
                .left(px(12.0))
                .size(px(2.5))
                .rounded_xs()
                .bg(color),
        )
        .child(
            div()
                .absolute()
                .top(px(14.0))
                .left(px(14.0))
                .size(px(2.5))
                .rounded_xs()
                .bg(color),
        )
        .child(
            div()
                .absolute()
                .top(px(16.0))
                .left(px(16.0))
                .size(px(2.5))
                .rounded_xs()
                .bg(color),
        )
        .child(
            div()
                .absolute()
                .top(px(18.0))
                .left(px(18.0))
                .size(px(2.5))
                .rounded_xs()
                .bg(color),
        )
        .into_any_element()
}

/// 渲染源文件管理矢量图标 (VS Code 风格 Git 分支图与 3 个节点)
pub fn render_source_control_icon(color: Rgba, pal: &crate::ui::theme::ThemePalette) -> AnyElement {
    div()
        .size(px(22.0))
        .relative()
        // 主干坚线
        .child(
            div()
                .absolute()
                .top(px(5.5))
                .left(px(5.25))
                .w(px(1.5))
                .h(px(11.0))
                .bg(color),
        )
        // 主干顶部节点
        .child(
            div()
                .absolute()
                .top(px(2.5))
                .left(px(3.25))
                .size(px(5.5))
                .rounded_full()
                .border_1()
                .border_color(color)
                .bg(pal.panel_surface),
        )
        // 主干底部节点
        .child(
            div()
                .absolute()
                .top(px(14.5))
                .left(px(3.25))
                .size(px(5.5))
                .rounded_full()
                .border_1()
                .border_color(color)
                .bg(pal.panel_surface),
        )
        // 分支横向折线
        .child(
            div()
                .absolute()
                .top(px(10.5))
                .left(px(6.0))
                .w(px(6.5))
                .h(px(1.5))
                .bg(color),
        )
        // 分支向上竖折线
        .child(
            div()
                .absolute()
                .top(px(7.5))
                .left(px(12.5))
                .w(px(1.5))
                .h(px(4.0))
                .bg(color),
        )
        // 分支右侧节点
        .child(
            div()
                .absolute()
                .top(px(4.0))
                .left(px(10.5))
                .size(px(5.5))
                .rounded_full()
                .border_1()
                .border_color(color)
                .bg(pal.panel_surface),
        )
        .into_any_element()
}

/// 渲染插件扩展中心矢量图标 (VS Code 经典 4 积木块拼图图标，右上块凸出漂浮)
pub fn render_plugins_icon(color: Rgba, _pal: &crate::ui::theme::ThemePalette) -> AnyElement {
    div()
        .size(px(22.0))
        .relative()
        // 左上块 (原位)
        .child(
            div()
                .absolute()
                .top(px(3.0))
                .left(px(3.0))
                .size(px(6.5))
                .rounded(px(1.0))
                .border_1()
                .border_color(color),
        )
        // 左下块 (原位)
        .child(
            div()
                .absolute()
                .top(px(12.0))
                .left(px(3.0))
                .size(px(6.5))
                .rounded(px(1.0))
                .border_1()
                .border_color(color),
        )
        // 右下块 (原位)
        .child(
            div()
                .absolute()
                .top(px(12.0))
                .left(px(12.0))
                .size(px(6.5))
                .rounded(px(1.0))
                .border_1()
                .border_color(color),
        )
        // 右上扩展块 (向外漂浮拼装态)
        .child(
            div()
                .absolute()
                .top(px(1.5))
                .left(px(13.5))
                .size(px(6.5))
                .rounded(px(1.0))
                .bg(color),
        )
        .into_any_element()
}

/// 渲染任务管理器/冒险笔记图标 (任务卷轴 / 列表与勾选标记)
pub fn render_quests_icon(color: Rgba, _pal: &crate::ui::theme::ThemePalette) -> AnyElement {
    div()
        .size(px(22.0))
        .relative()
        // 卷轴主体边框
        .child(
            div()
                .absolute()
                .top(px(2.5))
                .left(px(4.0))
                .w(px(14.0))
                .h(px(17.0))
                .rounded(px(2.0))
                .border_1()
                .border_color(color),
        )
        // 第一条任务文字线
        .child(
            div()
                .absolute()
                .top(px(6.0))
                .left(px(7.0))
                .w(px(8.0))
                .h(px(1.5))
                .rounded(px(0.5))
                .bg(color),
        )
        // 第二条任务文字线
        .child(
            div()
                .absolute()
                .top(px(9.5))
                .left(px(7.0))
                .w(px(8.0))
                .h(px(1.5))
                .rounded(px(0.5))
                .bg(color),
        )
        // 第三条任务文字线 (短线)
        .child(
            div()
                .absolute()
                .top(px(13.0))
                .left(px(7.0))
                .w(px(5.0))
                .h(px(1.5))
                .rounded(px(0.5))
                .bg(color),
        )
        .into_any_element()
}
