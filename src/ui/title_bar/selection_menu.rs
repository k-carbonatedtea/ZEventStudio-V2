use gpui::*;
use gpui_component::{
    button::{Button, ButtonVariants},
    menu::{DropdownMenu, PopupMenu, PopupMenuItem},
    Sizable,
};
use crate::model::NodeKind;
use crate::ui::state::AppState;

/// 渲染「选择 (Selection)」下拉菜单
pub fn render_selection_menu(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let model_select_all = model.clone();
    let model_deselect = model.clone();
    let model_actions = model.clone();
    let model_switches = model.clone();
    let model_queries = model.clone();
    let model_subflows = model.clone();
    let model_entries = model.clone();

    let label_select_all = state.t("menu.select_all").to_string();
    let label_deselect = state.t("menu.deselect_all").to_string();
    let label_actions = state.t("menu.select_actions").to_string();
    let label_switches = state.t("menu.select_switches").to_string();
    let label_queries = state.t("menu.select_queries").to_string();
    let label_subflows = state.t("menu.select_subflows").to_string();
    let label_entries = state.t("menu.select_entries").to_string();

    Button::new("titlebar_menu_selection")
        .ghost()
        .xsmall()
        .label(state.t("menu.selection"))
        .dropdown_menu(move |menu: PopupMenu, _window, _cx| {
            let m_sel_all = model_select_all.clone();
            let m_desel = model_deselect.clone();
            let m_act = model_actions.clone();
            let m_sw = model_switches.clone();
            let m_qu = model_queries.clone();
            let m_sub = model_subflows.clone();
            let m_ent = model_entries.clone();

            menu.min_w(px(240.0))
                .item(
                    PopupMenuItem::new(label_select_all.clone())
                        .on_click(move |_, _, cx| {
                            m_sel_all.update(cx, |st, cx| {
                                if let Some(first) = st.graph.nodes.first() {
                                    let id = first.id.clone();
                                    st.focus_node(&id);
                                }
                                cx.notify();
                            });
                        }),
                )
                .item(
                    PopupMenuItem::new(label_deselect.clone())
                        .on_click(move |_, _, cx| {
                            m_desel.update(cx, |st, cx| {
                                st.selected_node_id = None;
                                cx.notify();
                            });
                        }),
                )
                .separator()
                .item(
                    PopupMenuItem::new(label_entries.clone())
                        .on_click(move |_, _, cx| {
                            m_ent.update(cx, |st, cx| {
                                if let Some(node) = st.graph.nodes.iter().find(|n| n.kind == NodeKind::EntryPoint) {
                                    let id = node.id.clone();
                                    st.focus_node(&id);
                                }
                                cx.notify();
                            });
                        }),
                )
                .item(
                    PopupMenuItem::new(label_actions.clone())
                        .on_click(move |_, _, cx| {
                            m_act.update(cx, |st, cx| {
                                if let Some(node) = st.graph.nodes.iter().find(|n| n.kind == NodeKind::Action) {
                                    let id = node.id.clone();
                                    st.focus_node(&id);
                                }
                                cx.notify();
                            });
                        }),
                )
                .item(
                    PopupMenuItem::new(label_switches.clone())
                        .on_click(move |_, _, cx| {
                            m_sw.update(cx, |st, cx| {
                                if let Some(node) = st.graph.nodes.iter().find(|n| n.kind == NodeKind::Switch) {
                                    let id = node.id.clone();
                                    st.focus_node(&id);
                                }
                                cx.notify();
                            });
                        }),
                )
                .item(
                    PopupMenuItem::new(label_queries.clone())
                        .on_click(move |_, _, cx| {
                            m_qu.update(cx, |st, cx| {
                                if let Some(node) = st.graph.nodes.iter().find(|n| n.kind == NodeKind::Query) {
                                    let id = node.id.clone();
                                    st.focus_node(&id);
                                }
                                cx.notify();
                            });
                        }),
                )
                .item(
                    PopupMenuItem::new(label_subflows.clone())
                        .on_click(move |_, _, cx| {
                            m_sub.update(cx, |st, cx| {
                                if let Some(node) = st.graph.nodes.iter().find(|n| n.kind == NodeKind::SubFlow) {
                                    let id = node.id.clone();
                                    st.focus_node(&id);
                                }
                                cx.notify();
                            });
                        }),
                )
        })
}
