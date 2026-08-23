use gpui::*;
use gpui_component::{
    button::{Button, ButtonVariants},
    menu::{DropdownMenu, PopupMenu, PopupMenuItem},
    Sizable,
};
use crate::ui::state::AppState;

/// 渲染「编辑 (Edit)」下拉菜单
pub fn render_edit_menu(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let model_layout = model.clone();
    let model_expand = model.clone();
    let model_collapse = model.clone();
    let model_knife = model.clone();
    let model_delete = model.clone();

    let label_layout = state.t("menu.auto_layout").to_string();
    let label_expand = state.t("menu.expand_all_params").to_string();
    let label_collapse = state.t("menu.collapse_all_params").to_string();
    let label_knife = state.t("menu.knife_mode").to_string();
    let label_delete = state.t("menu.delete_selected").to_string();

    Button::new("titlebar_menu_edit")
        .ghost()
        .xsmall()
        .label(state.t("menu.edit"))
        .dropdown_menu(move |menu: PopupMenu, _window, _cx| {
            let m_layout = model_layout.clone();
            let m_expand = model_expand.clone();
            let m_collapse = model_collapse.clone();
            let m_knife = model_knife.clone();
            let m_delete = model_delete.clone();

            menu.min_w(px(200.0))
                .item(
                    PopupMenuItem::new(label_layout.clone())
                        .on_click(move |_, _, cx| {
                            m_layout.update(cx, |st, cx| {
                                st.auto_layout();
                                cx.notify();
                            });
                        }),
                )
                .separator()
                .item(
                    PopupMenuItem::new(label_expand.clone())
                        .on_click(move |_, _, cx| {
                            m_expand.update(cx, |st, cx| {
                                st.expand_all_params = true;
                                st.update_node_sizes();
                                cx.notify();
                            });
                        }),
                )
                .item(
                    PopupMenuItem::new(label_collapse.clone())
                        .on_click(move |_, _, cx| {
                            m_collapse.update(cx, |st, cx| {
                                st.expand_all_params = false;
                                st.update_node_sizes();
                                cx.notify();
                            });
                        }),
                )
                .separator()
                .item(
                    PopupMenuItem::new(label_knife.clone())
                        .on_click(move |_, _, cx| {
                            m_knife.update(cx, |st, cx| {
                                st.knife_mode = !st.knife_mode;
                                cx.notify();
                            });
                        }),
                )
                .separator()
                .item(
                    PopupMenuItem::new(label_delete.clone())
                        .on_click(move |_, _, cx| {
                            m_delete.update(cx, |st, cx| {
                                st.delete_selected_node();
                                cx.notify();
                            });
                        }),
                )
        })
}
