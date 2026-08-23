pub mod edit_menu;
pub mod file_menu;
pub mod help_menu;
pub mod selection_menu;
pub mod view_menu;

use gpui::*;
use gpui_component::TitleBar;
use crate::ui::state::AppState;
use edit_menu::render_edit_menu;
use file_menu::render_file_menu;
use help_menu::render_help_menu;
use selection_menu::render_selection_menu;
use view_menu::render_view_menu;

/// 渲染顶部现代化玻璃拟态标题栏与菜单栏 (App Icon + File/Edit/Select/View/Help + Centered Title + Window Controls)
pub fn render_title_bar(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let active_doc_name = state
        .active_tab_id
        .as_ref()
        .and_then(|id| state.open_tabs.iter().find(|t| &t.id == id))
        .map(|tab| tab.title.clone())
        .unwrap_or_else(|| "ZEvent Studio - GPU-Accelerated Nodeflow Editor".to_string());

    TitleBar::new()
        .bg(pal.panel_surface)
        .border_color(pal.panel_border)
        // 1. 左侧区域: 应用程序标识 (Logo) + 顶部核心菜单栏 (文件 / 编辑 / 选择 / 查看 / 帮助)
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                // 应用程序微标
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .mr_2()
                        .child(
                            div()
                                .size(px(18.0))
                                .rounded_md()
                                .bg(pal.card_surface)
                                .border_1()
                                .border_color(pal.card_border)
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_size(px(10.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(pal.secondary_accent)
                                .child("⚡"),
                        )
                        .child(
                            div()
                                .text_size(px(12.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(pal.text_primary)
                                .child("ZEvent"),
                        ),
                )
                // 核心菜单选项: 文件 / 编辑 / 选择 / 查看 / 帮助
                .child(render_file_menu(state, model))
                .child(render_edit_menu(state, model))
                .child(render_selection_menu(state, model))
                .child(render_view_menu(state, model))
                .child(render_help_menu(state, model)),
        )
        // 2. 中间区域: 当前打开文件/流程图标题 (兼顾视窗拖拽交互区域)
        .child(
            div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .text_size(px(11.5))
                        .text_color(pal.text_secondary)
                        .child(active_doc_name),
                ),
        )
}
