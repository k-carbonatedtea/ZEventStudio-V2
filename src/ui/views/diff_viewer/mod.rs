use gpui::*;
use crate::model::DocumentTabKind;
use crate::ui::state::AppState;

pub mod hatch_pattern;
pub mod header;
pub mod split_table;

pub use header::render_diff_header;
pub use split_table::render_diff_split_table;

/// 渲染 IDE 风格的 Side-by-Side 双栏代码差异对比主工作区视图 (Diff Viewer View)
/// 完全对应图2的界面设计与交互规范
pub fn render_diff_viewer_view(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();

    // 寻找当前活跃的对比模式标签页
    let active_diff_tab = state
        .active_tab_id
        .as_deref()
        .and_then(|id| state.open_tabs.iter().find(|t| t.id == id && t.tab_kind == DocumentTabKind::DiffViewer))
        .or_else(|| state.open_tabs.iter().find(|t| t.tab_kind == DocumentTabKind::DiffViewer));

    let tab_id = active_diff_tab.map(|t| t.id.as_str()).unwrap_or("");
    let diff_state = match active_diff_tab.and_then(|t| t.diff_state.as_ref()) {
        Some(ds) => ds,
        None => {
            return div()
                .size_full()
                .bg(pal.panel_surface)
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_2()
                .child(div().text_size(px(32.0)).child("🌿"))
                .child(
                    div()
                        .text_size(px(13.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.text_primary)
                        .child(state.t("diff.no_file_selected")),
                )
                .into_any_element();
        }
    };

    div()
        .id("diff_viewer_view_container")
        .size_full()
        .bg(pal.panel_surface)
        .flex()
        .flex_col()
        .overflow_hidden()
        // 1. 顶部面包屑导航与差异统计条
        .child(render_diff_header(diff_state, model, tab_id, &pal))
        // 2. 双栏 Side-by-Side 差异对比表格
        .child(render_diff_split_table(state, diff_state, &pal))
        .into_any_element()
}
