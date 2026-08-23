use gpui::*;
use gpui_component::{
    button::{Button, ButtonVariants},
    menu::{DropdownMenu, PopupMenu, PopupMenuItem},
    Sizable,
};
use crate::ui::state::AppState;

/// 渲染「文件 (File)」下拉菜单
pub fn render_file_menu(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let model_new = model.clone();
    let model_open_file = model.clone();
    let model_open_mod = model.clone();
    let model_open_pack = model.clone();
    let model_save = model.clone();
    let model_revert = model.clone();
    let model_close_tab = model.clone();
    let model_close_all = model.clone();
    let model_exit = model.clone();

    let label_new = state.t("menu.new_flow").to_string();
    let label_open_file = state.t("menu.open_file").to_string();
    let label_open_mod = state.t("menu.open_mod").to_string();
    let label_open_pack = state.t("menu.open_pack").to_string();
    let label_save = state.t("menu.save").to_string();
    let label_revert = state.t("menu.revert").to_string();
    let label_close_tab = state.t("menu.close_tab").to_string();
    let label_close_all = state.t("menu.close_all_tabs").to_string();
    let label_exit = state.t("menu.exit").to_string();

    Button::new("titlebar_menu_file")
        .ghost()
        .xsmall()
        .label(state.t("menu.file"))
        .dropdown_menu(move |menu: PopupMenu, _window, _cx| {
            let m_new = model_new.clone();
            let m_open_file = model_open_file.clone();
            let m_open_mod = model_open_mod.clone();
            let m_open_pack = model_open_pack.clone();
            let m_save = model_save.clone();
            let m_revert = model_revert.clone();
            let m_close_tab = model_close_tab.clone();
            let m_close_all = model_close_all.clone();
            let m_exit = model_exit.clone();

            menu.min_w(px(220.0))
                .item(
                    PopupMenuItem::new(label_new.clone())
                        .on_click(move |_, _, cx| {
                            m_new.update(cx, |st, cx| {
                                st.new_blank_flow();
                                cx.notify();
                            });
                        }),
                )
                .item(
                    PopupMenuItem::new(label_open_file.clone())
                        .on_click(move |_, _, cx| {
                            crate::ui::views::welcome::quick_tab::pick_and_load_file(&m_open_file, cx);
                        }),
                )
                .item(
                    PopupMenuItem::new(label_open_mod.clone())
                        .on_click(move |_, _, cx| {
                            crate::ui::views::welcome::quick_tab::pick_and_load_mod_folder(&m_open_mod, cx);
                        }),
                )
                .item(
                    PopupMenuItem::new(label_open_pack.clone())
                        .on_click(move |_, _, cx| {
                            crate::ui::views::welcome::quick_tab::pick_and_load_pack_file(&m_open_pack, cx);
                        }),
                )
                .separator()
                .item(
                    PopupMenuItem::new(label_save.clone())
                        .on_click(move |_, _, cx| {
                            m_save.update(cx, |st, cx| {
                                st.save_active_document();
                                cx.notify();
                            });
                        }),
                )
                .item(
                    PopupMenuItem::new(label_revert.clone())
                        .on_click(move |_, _, cx| {
                            m_revert.update(cx, |st, cx| {
                                st.revert_active_code();
                                cx.notify();
                            });
                        }),
                )
                .separator()
                .item(
                    PopupMenuItem::new(label_close_tab.clone())
                        .on_click(move |_, _, cx| {
                            m_close_tab.update(cx, |st, cx| {
                                if let Some(id) = st.active_tab_id.clone() {
                                    st.request_close_document_tab(&id);
                                }
                                cx.notify();
                            });
                        }),
                )
                .item(
                    PopupMenuItem::new(label_close_all.clone())
                        .on_click(move |_, _, cx| {
                            m_close_all.update(cx, |st, cx| {
                                st.close_all_document_tabs();
                                cx.notify();
                            });
                        }),
                )
                .separator()
                .item(
                    PopupMenuItem::new(label_exit.clone())
                        .on_click(move |_, _window, cx| {
                            m_exit.update(cx, |st, cx| {
                                let allow_exit = st.request_exit_application();
                                if allow_exit {
                                    cx.quit();
                                } else {
                                    cx.notify();
                                }
                            });
                        }),
                )
        })
}
