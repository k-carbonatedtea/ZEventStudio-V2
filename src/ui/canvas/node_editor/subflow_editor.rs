use gpui::*;
use crate::ui::canvas::node_editor::params_editor::render_params_table_editor;
use crate::ui::canvas::node_editor::{ActiveEditField, NodeEditorModalState};
use crate::ui::components::render_blinking_cursor;
use crate::ui::state::AppState;

/// 渲染 SubFlow 子流程属性与参数编辑器
pub fn render_subflow_editor(
    state: &AppState,
    modal: &NodeEditorModalState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let is_flow_active = modal.active_field == ActiveEditField::FlowchartName;
    let is_ep_active = modal.active_field == ActiveEditField::EntryPointName;

    let model_flow = model.clone();
    let model_ep = model.clone();

    let flow_name = if modal.res_flowchart_name.is_empty() {
        if is_flow_active {
            String::new()
        } else {
            state.t("modal.flowchart_placeholder").to_string()
        }
    } else {
        modal.res_flowchart_name.clone()
    };

    let ep_name = if modal.entry_point_name.is_empty() {
        if is_ep_active {
            String::new()
        } else {
            state.t("modal.entry_point_placeholder").to_string()
        }
    } else {
        modal.entry_point_name.clone()
    };

    div()
        .flex()
        .flex_col()
        .gap_3()
        // 流程图名称 (Flowchart)
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .w(px(80.0))
                        .text_size(px(12.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(hsla(240.0 / 360.0, 0.05, 0.80, 1.0))
                        .child(state.t("modal.flowchart_name")),
                )
                .child(
                    div()
                        .flex_1()
                        .h(px(32.0))
                        .px_3()
                        .rounded_md()
                        .bg(hsla(240.0 / 360.0, 0.06, 0.14, 1.0))
                        .border_1()
                        .border_color(if is_flow_active {
                            hsla(215.0 / 360.0, 0.90, 0.55, 1.0)
                        } else {
                            hsla(240.0 / 360.0, 0.06, 0.22, 1.0)
                        })
                        .cursor_text()
                        .flex()
                        .items_center()
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            model_flow.update(cx, |st, cx| {
                                if let Some(ref mut m) = st.active_node_editor_modal {
                                    m.active_field = ActiveEditField::FlowchartName;
                                }
                                cx.notify();
                            });
                        })
                        .child(
                            div()
                                .text_size(px(12.0))
                                .text_color(if modal.res_flowchart_name.is_empty() && !is_flow_active {
                                    hsla(240.0 / 360.0, 0.05, 0.45, 1.0)
                                } else {
                                    hsla(0.0, 0.0, 1.0, 1.0)
                                })
                                .child(flow_name),
                        )
                        .children(is_flow_active.then(|| {
                            render_blinking_cursor(hsla(215.0 / 360.0, 0.90, 0.55, 1.0))
                        })),
                ),
        )
        // 入口点名称 (EntryPoint)
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .w(px(80.0))
                        .text_size(px(12.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(hsla(240.0 / 360.0, 0.05, 0.80, 1.0))
                        .child(state.t("modal.entry_point_name")),
                )
                .child(
                    div()
                        .flex_1()
                        .h(px(32.0))
                        .px_3()
                        .rounded_md()
                        .bg(hsla(240.0 / 360.0, 0.06, 0.14, 1.0))
                        .border_1()
                        .border_color(if is_ep_active {
                            hsla(215.0 / 360.0, 0.90, 0.55, 1.0)
                        } else {
                            hsla(240.0 / 360.0, 0.06, 0.22, 1.0)
                        })
                        .cursor_text()
                        .flex()
                        .items_center()
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            model_ep.update(cx, |st, cx| {
                                if let Some(ref mut m) = st.active_node_editor_modal {
                                    m.active_field = ActiveEditField::EntryPointName;
                                }
                                cx.notify();
                            });
                        })
                        .child(
                            div()
                                .text_size(px(12.0))
                                .text_color(if modal.entry_point_name.is_empty() && !is_ep_active {
                                    hsla(240.0 / 360.0, 0.05, 0.45, 1.0)
                                } else {
                                    hsla(0.0, 0.0, 1.0, 1.0)
                                })
                                .child(ep_name),
                        )
                        .children(is_ep_active.then(|| {
                            render_blinking_cursor(hsla(215.0 / 360.0, 0.90, 0.55, 1.0))
                        })),
                ),
        )
        // 提示说明
        .child(
            div()
                .text_size(px(11.0))
                .text_color(hsla(240.0 / 360.0, 0.05, 0.50, 1.0))
                .child(state.t("modal.subflow_hint")),
        )
        // 嵌入参数属性表格 (已过滤 Flowchart / EntryPoint 伪属性)
        .child(render_params_table_editor(state, modal, model))
}
