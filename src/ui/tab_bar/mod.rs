use gpui::*;
use crate::ui::state::AppState;

pub mod tab_item;
pub use tab_item::render_tab_item;

/// 渲染顶部多文件/文档标签栏 (Document Tabs Bar)
pub fn render_document_tab_bar(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let active_id = state.active_tab_id.as_deref().unwrap_or("");
    let model_new = model.clone();

    let active_file_path = state
        .open_tabs
        .iter()
        .find(|t| t.id == active_id)
        .map(|t| t.file_path.clone())
        .unwrap_or_else(|| state.graph.name.clone());

    div()
        .id("document_tab_bar")
        .h(px(32.0))
        .w_full()
        .bg(hsla(240.0 / 360.0, 0.06, 0.09, 1.0)) // 高级深黑底色
        .border_b_1()
        .border_color(hsla(240.0 / 360.0, 0.06, 0.16, 1.0))
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
                    render_tab_item(tab, is_active, model)
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
                        .text_color(hsla(240.0 / 360.0, 0.05, 0.55, 1.0))
                        .hover(|s| {
                            s.bg(hsla(240.0 / 360.0, 0.06, 0.16, 1.0))
                                .text_color(hsla(0.0, 0.0, 1.0, 1.0))
                        })

                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            model_new.update(cx, |st, cx| {
                                st.new_blank_flow();
                                cx.notify();
                            });
                        })
                        .child("+"),
                ),
        )
        // 右侧：当前活跃文件虚拟路径提示 / 统计徽章
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .pr_2()
                .child(
                    div()
                        .text_size(px(11.0))
                        .text_color(hsla(240.0 / 360.0, 0.05, 0.40, 1.0))
                        .child(active_file_path),
                ),
        )
}
