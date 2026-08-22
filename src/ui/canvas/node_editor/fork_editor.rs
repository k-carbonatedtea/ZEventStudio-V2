use gpui::*;
use crate::ui::canvas::node_editor::{BranchItem, NodeEditorModalState};
use crate::ui::state::AppState;

/// 渲染 Fork 并行分支与 Join 汇合编辑器
pub fn render_fork_editor(
    state: &AppState,
    modal: &NodeEditorModalState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let model_add = model.clone();
    let has_branches = !modal.branches.is_empty();
    let join_node = modal
        .join_node_id
        .clone()
        .unwrap_or_else(|| state.t("modal.no_join").to_string());

    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .flex()
                .flex_col()
                .border_1()
                .border_color(hsla(240.0 / 360.0, 0.06, 0.20, 1.0))
                .rounded_lg()
                .overflow_hidden()
                // 表头
                .child(
                    div()
                        .flex()
                        .h(px(32.0))
                        .bg(hsla(240.0 / 360.0, 0.06, 0.08, 1.0))
                        .border_b_1()
                        .border_color(hsla(240.0 / 360.0, 0.06, 0.20, 1.0))
                        .items_center()
                        .px_3()
                        .child(
                            div()
                                .flex_1()
                                .text_size(px(11.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(hsla(240.0 / 360.0, 0.05, 0.70, 1.0))
                                .child(state.t("modal.target_event")),
                        )
                        .child(
                            div()
                                .w(px(36.0))
                                .text_size(px(11.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(hsla(240.0 / 360.0, 0.05, 0.70, 1.0))
                                .child(state.t("modal.action")),
                        ),
                )
                // 分支列表
                .children(if has_branches {
                    modal
                        .branches
                        .iter()
                        .enumerate()
                        .map(|(idx, branch)| {
                            let model_del = model.clone();
                            let b_tgt = branch.target_id.clone();
                            div()
                                .flex()
                                .h(px(38.0))
                                .items_center()
                                .px_3()
                                .border_b_1()
                                .border_color(hsla(240.0 / 360.0, 0.06, 0.16, 1.0))
                                .bg(if idx % 2 == 0 {
                                    hsla(240.0 / 360.0, 0.06, 0.11, 1.0)
                                } else {
                                    hsla(240.0 / 360.0, 0.06, 0.09, 1.0)
                                })
                                .child(
                                    div()
                                        .flex_1()
                                        .pr_2()
                                        .child(
                                            div()
                                                .px_2()
                                                .py_1()
                                                .rounded_md()
                                                .bg(hsla(240.0 / 360.0, 0.06, 0.15, 1.0))
                                                .border_1()
                                                .border_color(hsla(240.0 / 360.0, 0.06, 0.22, 1.0))
                                                .text_size(px(11.5))
                                                .text_color(if b_tgt.is_empty() {
                                                    hsla(240.0 / 360.0, 0.05, 0.50, 1.0)
                                                } else {
                                                    hsla(0.0, 0.0, 1.0, 1.0)
                                                })
                                                .child(if b_tgt.is_empty() {
                                                    state.t("modal.select_target_event").to_string()
                                                } else {
                                                    b_tgt
                                                }),
                                        ),
                                )
                                .child(
                                    div()
                                        .w(px(36.0))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(
                                            div()
                                                .p_1()
                                                .rounded_md()
                                                .cursor_pointer()
                                                .hover(|s| s.bg(hsla(0.0, 0.70, 0.35, 0.9)))
                                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                                    cx.stop_propagation();
                                                    model_del.update(cx, |st, cx| {
                                                        if let Some(ref mut m) =
                                                            st.active_node_editor_modal
                                                        {
                                                            if idx < m.branches.len() {
                                                                m.branches.remove(idx);
                                                            }
                                                        }
                                                        cx.notify();
                                                    });
                                                })
                                                .child("🗑️"),
                                        ),
                                )
                        })
                        .collect()
                } else {
                    vec![div()
                        .p_4()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(11.5))
                        .text_color(hsla(240.0 / 360.0, 0.05, 0.55, 1.0))
                        .child(state.t("modal.no_fork_branches"))]
                }),
        )
        // 添加并行分支按钮
        .child(
            div()
                .flex()
                .child(
                    div()
                        .px_3()
                        .py_1p5()
                        .rounded_md()
                        .bg(hsla(240.0 / 360.0, 0.06, 0.14, 1.0))
                        .border_1()
                        .border_color(hsla(240.0 / 360.0, 0.06, 0.22, 1.0))
                        .cursor_pointer()
                        .text_size(px(11.5))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(hsla(0.0, 0.0, 1.0, 1.0))
                        .hover(|s| s.bg(hsla(240.0 / 360.0, 0.06, 0.18, 1.0)))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            model_add.update(cx, |st, cx| {
                                if let Some(ref mut m) = st.active_node_editor_modal {
                                    let next_val = format!("Branch {}", m.branches.len() + 1);
                                    m.branches.push(BranchItem {
                                        value: next_val,
                                        target_id: String::new(),
                                    });
                                }
                                cx.notify();
                            });
                        })
                        .child(state.t("modal.add_parallel_branch")),
                ),
        )
        // 汇合目标节点 (Join Node)
        .child(
            div()
                .pt_3()
                .border_t_1()
                .border_color(hsla(240.0 / 360.0, 0.06, 0.18, 1.0))
                .flex()
                .flex_col()
                .gap_1p5()
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(12.5))
                        .text_color(hsla(0.0, 0.0, 1.0, 1.0))
                        .child(state.t("modal.join_node")),
                )
                .child(
                    div()
                        .px_3()
                        .py_2()
                        .rounded_md()
                        .bg(hsla(240.0 / 360.0, 0.06, 0.14, 1.0))
                        .border_1()
                        .border_color(hsla(240.0 / 360.0, 0.06, 0.22, 1.0))
                        .text_size(px(12.0))
                        .text_color(hsla(240.0 / 360.0, 0.05, 0.75, 1.0))
                        .child(join_node),
                ),
        )
}
