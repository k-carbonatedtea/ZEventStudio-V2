use gpui::*;
use crate::model::{NodeKind, PropertyItem};
use crate::ui::state::AppState;

pub mod actor_action_selectors;
pub mod fork_editor;
pub mod param_row;
pub mod params_editor;
pub mod subflow_editor;
pub mod switch_editor;

#[allow(unused_imports)]
pub use fork_editor::render_fork_editor;
#[allow(unused_imports)]
pub use param_row::render_param_row;
#[allow(unused_imports)]
pub use params_editor::render_params_table_editor;
#[allow(unused_imports)]
pub use subflow_editor::render_subflow_editor;
#[allow(unused_imports)]
pub use switch_editor::render_switch_editor;





/// 分支项目（用于 Switch 分支或 Fork 并行流）
#[derive(Debug, Clone, PartialEq)]
pub struct BranchItem {
    pub value: String,
    pub target_id: String,
}

/// 节点编辑模态框当前激活的键盘可输入字段
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveEditField {

    None,
    FlowchartName,
    EntryPointName,
    BranchValue(usize),
    BranchTarget(usize),
    ParamKey(usize),
    ParamVal(usize),
}

/// 节点编辑模态框全局状态
#[derive(Debug, Clone, PartialEq)]
pub struct NodeEditorModalState {
    pub node_id: String,
    pub node_kind: NodeKind,
    pub title: String,
    pub subtitle: String,
    pub actor_name: String,
    pub action_name: String,
    pub show_actor_dropdown: bool,
    pub show_action_dropdown: bool,
    pub res_flowchart_name: String,
    pub entry_point_name: String,
    pub join_node_id: Option<String>,
    pub branches: Vec<BranchItem>,
    pub properties: Vec<PropertyItem>,
    pub active_field: ActiveEditField,
    pub is_adding_param: bool,
    pub new_param_type: String,
    pub new_param_key: String,
    pub new_param_val: String,
}



/// 渲染节点属性与分支编辑模态框 (Node Editor Modal)
pub fn render_node_editor_modal(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let modal = match &state.active_node_editor_modal {
        Some(m) => m.clone(),
        None => return div().into_any_element(),
    };

    let model_cancel = model.clone();
    let model_save = model.clone();
    let model_close = model.clone();

    let modal_title = match modal.node_kind {
        NodeKind::Switch => state.t("modal.edit_switch").to_string(),
        NodeKind::Fork => state.t("modal.edit_fork").to_string(),
        NodeKind::SubFlow => state.t("modal.edit_subflow").to_string(),
        _ => format!(
            "{} ({})",
            state.t("modal.edit_event"),
            modal.node_kind.display_name()
        ),
    };

    div()
        .id("node_editor_modal_overlay")
        .absolute()
        .inset_0()
        .occlude()
        .bg(hsla(0.0, 0.0, 0.0, 0.70))
        .flex()
        .items_center()
        .justify_center()
        .on_scroll_wheel(|_, _, cx| {
            cx.stop_propagation();
        })
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            cx.stop_propagation();
            model_cancel.update(cx, |st, cx| {
                st.close_node_editor();
                cx.notify();
            });
        })
        .on_mouse_down(MouseButton::Right, |_, _, cx| {
            cx.stop_propagation();
        })
        .on_mouse_up(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        })
        .on_mouse_up(MouseButton::Right, |_, _, cx| {
            cx.stop_propagation();
        })
        .on_click(|_, _, cx| {
            cx.stop_propagation();
        })
        .child(
            div()
                .id("node_editor_modal_card")
                .w(px(760.0))
                .max_h(px(720.0))
                .bg(hsla(240.0 / 360.0, 0.06, 0.10, 0.98))
                .border_1()
                .border_color(hsla(240.0 / 360.0, 0.06, 0.22, 1.0))
                .rounded_xl()
                .shadow_2xl()
                .flex()
                .flex_col()
                .overflow_hidden()

                .on_scroll_wheel(|_, _, cx| {
                    cx.stop_propagation();
                })
                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                .on_mouse_down(MouseButton::Right, |_, _, cx| {
                    cx.stop_propagation();
                })
                .on_mouse_up(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                .on_mouse_up(MouseButton::Right, |_, _, cx| {
                    cx.stop_propagation();
                })
                .on_click(|_, _, cx| {
                    cx.stop_propagation();
                })
                // 1. 模态框顶部标题栏
                .child(
                    div()
                        .h(px(46.0))
                        .px_4()
                        .bg(hsla(240.0 / 360.0, 0.06, 0.07, 1.0))
                        .border_b_1()
                        .border_color(hsla(240.0 / 360.0, 0.06, 0.18, 1.0))
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2p5()
                                .child(
                                    div()
                                        .w(px(12.0))
                                        .h(px(12.0))
                                        .rounded_sm()
                                        .bg(modal.node_kind.badge_color()),
                                )
                                .child(
                                    div()
                                        .font_weight(FontWeight::BOLD)
                                        .text_size(px(13.5))
                                        .text_color(hsla(0.0, 0.0, 1.0, 1.0))
                                        .child(modal_title),
                                ),
                        )
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_md()
                                .cursor_pointer()
                                .text_color(hsla(240.0 / 360.0, 0.05, 0.60, 1.0))
                                .hover(|s| {
                                    s.bg(hsla(0.0, 0.70, 0.35, 0.9))
                                        .text_color(hsla(0.0, 0.0, 1.0, 1.0))
                                })
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    cx.stop_propagation();
                                    model_close.update(cx, |st, cx| {
                                        st.close_node_editor();
                                        cx.notify();
                                    });
                                })
                                .child("✕"),
                        ),
                )
                // 2. 模态框主体内容区 (根据节点类型分别渲染分支/并行/子流程/参数)
                .child(
                    div()
                        .id("node_editor_modal_body")
                        .flex_1()
                        .overflow_y_scroll()
                        .p_4()
                        .flex()
                        .flex_col()
                        .gap_4()
                        .child(match modal.node_kind {
                            NodeKind::Switch => {
                                render_switch_editor(state, &modal, model).into_any_element()
                            }
                            NodeKind::Fork => {
                                render_fork_editor(state, &modal, model).into_any_element()
                            }
                            NodeKind::SubFlow => {
                                render_subflow_editor(state, &modal, model).into_any_element()
                            }
                            _ => render_params_table_editor(state, &modal, model).into_any_element(),
                        }),
                )
                // 3. 模态框底部操作栏 (取消 / 保存)
                .child(
                    div()
                        .h(px(52.0))
                        .px_4()
                        .bg(hsla(240.0 / 360.0, 0.06, 0.07, 1.0))
                        .border_t_1()
                        .border_color(hsla(240.0 / 360.0, 0.06, 0.18, 1.0))
                        .flex()
                        .items_center()
                        .justify_end()
                        .gap_2p5()
                        .child(
                            div()
                                .px_4()
                                .py_1p5()
                                .rounded_md()
                                .bg(hsla(240.0 / 360.0, 0.06, 0.14, 1.0))
                                .border_1()
                                .border_color(hsla(240.0 / 360.0, 0.06, 0.22, 1.0))
                                .text_size(px(12.0))
                                .text_color(hsla(240.0 / 360.0, 0.05, 0.85, 1.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(hsla(240.0 / 360.0, 0.06, 0.20, 1.0)))
                                .on_mouse_down(MouseButton::Left, {
                                    let m_cancel = model.clone();
                                    move |_, _, cx| {
                                        cx.stop_propagation();
                                        m_cancel.update(cx, |st, cx| {
                                            st.close_node_editor();
                                            cx.notify();
                                        });
                                    }
                                })
                                .child(state.t("modal.cancel")),
                        )

                        .child(
                            div()
                                .px_4()
                                .py_1p5()
                                .rounded_md()
                                .bg(hsla(215.0 / 360.0, 0.90, 0.55, 1.0))
                                .text_size(px(12.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(hsla(0.0, 0.0, 1.0, 1.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(hsla(215.0 / 360.0, 0.95, 0.62, 1.0)))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    cx.stop_propagation();
                                    model_save.update(cx, |st, cx| {
                                        st.save_node_editor();
                                        cx.notify();
                                    });
                                })
                                .child(format!("💾 {}", state.t("modal.save"))),
                        ),
                ),
        )
        .into_any_element()
}
