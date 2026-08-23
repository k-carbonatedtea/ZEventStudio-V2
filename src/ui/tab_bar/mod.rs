use gpui::*;
use crate::ui::state::AppState;

pub mod context_menu;
pub mod tab_item;
pub mod unsaved_modal;
pub use context_menu::render_tab_context_menu;
pub use tab_item::render_tab_item;
pub use unsaved_modal::render_unsaved_changes_modal;

/// 渲染顶部多文件/文档标签栏 (Document Tabs Bar)
pub fn render_document_tab_bar(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let active_id = state.active_tab_id.as_deref().unwrap_or("");
    let model_new = model.clone();
    let pal = state.theme();

    div()
        .id("document_tab_bar")
        .h(px(32.0))
        .w_full()
        .bg(pal.panel_surface)
        .border_b_1()
        .border_color(pal.panel_border)
        .flex()
        .items_center()
        .justify_between()
        .px_2()
        .child(
            div()
                .flex_1()
                .flex()
                .items_center()
                .gap_1p5()
                .overflow_hidden()
                // 标签项列表
                .children(state.open_tabs.iter().map(|tab| {
                    let is_active = tab.id == active_id;
                    render_tab_item(tab, is_active, state, model)
                }))
                // + 新建空白标签按钮
                .child(
                    div()
                        .w(px(24.0))
                        .h(px(24.0))
                        .rounded_md()
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .text_size(px(12.0))
                        .text_color(pal.text_secondary)
                        .hover(|s| {
                            s.bg(pal.card_surface)
                                .text_color(pal.text_primary)
                        })
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_new.update(cx, |st, cx| {
                                st.new_blank_flow();
                                cx.notify();
                            });
                        })
                        .child("+"),
                ),
        )
}
