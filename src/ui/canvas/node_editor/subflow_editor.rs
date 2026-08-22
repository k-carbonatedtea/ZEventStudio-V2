use gpui::*;
use crate::ui::canvas::node_editor::params_editor::render_params_table_editor;
use crate::ui::canvas::node_editor::NodeEditorModalState;
use crate::ui::state::AppState;

/// 渲染 SubFlow 子流程属性与参数编辑器
pub fn render_subflow_editor(
    state: &AppState,
    modal: &NodeEditorModalState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let flow_name = if modal.res_flowchart_name.is_empty() {
        state.t("modal.flowchart_placeholder").to_string()
    } else {
        modal.res_flowchart_name.clone()
    };
    let ep_name = if modal.entry_point_name.is_empty() {
        state.t("modal.entry_point_placeholder").to_string()
    } else {
        modal.entry_point_name.clone()
    };

    div()
        .flex()
        .flex_col()
        .gap_3()
        // 流程图名称
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
                        .px_3()
                        .py_1p5()
                        .rounded_md()
                        .bg(hsla(240.0 / 360.0, 0.06, 0.14, 1.0))
                        .border_1()
                        .border_color(hsla(240.0 / 360.0, 0.06, 0.22, 1.0))
                        .text_size(px(12.0))
                        .text_color(if modal.res_flowchart_name.is_empty() {
                            hsla(240.0 / 360.0, 0.05, 0.45, 1.0)
                        } else {
                            hsla(0.0, 0.0, 1.0, 1.0)
                        })
                        .child(flow_name),
                ),
        )
        // 入口点名称
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
                        .px_3()
                        .py_1p5()
                        .rounded_md()
                        .bg(hsla(240.0 / 360.0, 0.06, 0.14, 1.0))
                        .border_1()
                        .border_color(hsla(240.0 / 360.0, 0.06, 0.22, 1.0))
                        .text_size(px(12.0))
                        .text_color(if modal.entry_point_name.is_empty() {
                            hsla(240.0 / 360.0, 0.05, 0.45, 1.0)
                        } else {
                            hsla(0.0, 0.0, 1.0, 1.0)
                        })
                        .child(ep_name),
                ),
        )
        .child(
            div()
                .text_size(px(11.0))
                .text_color(hsla(240.0 / 360.0, 0.05, 0.50, 1.0))
                .child(state.t("modal.subflow_hint")),
        )
        // 嵌入参数属性表格
        .child(render_params_table_editor(state, modal, model))
}
