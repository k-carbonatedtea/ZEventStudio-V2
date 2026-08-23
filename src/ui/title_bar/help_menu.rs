use gpui::*;
use gpui_component::{
    button::{Button, ButtonVariants},
    menu::{DropdownMenu, PopupMenu, PopupMenuItem},
    Sizable,
};
use crate::ui::components::Toast;
use crate::ui::state::AppState;

/// 渲染「帮助 (Help)」下拉菜单
pub fn render_help_menu(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let model_shortcuts = model.clone();
    let model_evfl = model.clone();
    let model_lang = model.clone();
    let model_about = model.clone();

    let label_shortcuts = state.t("menu.shortcuts").to_string();
    let label_evfl = state.t("menu.evfl_spec").to_string();
    let label_lang = state.t("menu.switch_lang").to_string();
    let label_about = state.t("menu.about").to_string();

    let info_shortcuts = state.t("menu.shortcuts_info").to_string();
    let info_evfl = state.t("welcome.guide_evfl_desc").to_string();
    let info_about = state.t("menu.about_info").to_string();

    Button::new("titlebar_menu_help")
        .ghost()
        .xsmall()
        .label(state.t("menu.help"))
        .dropdown_menu(move |menu: PopupMenu, _window, _cx| {
            let m_sc = model_shortcuts.clone();
            let m_ev = model_evfl.clone();
            let m_lg = model_lang.clone();
            let m_ab = model_about.clone();

            let sc_text = info_shortcuts.clone();
            let ev_text = info_evfl.clone();
            let ab_text = info_about.clone();

            menu.min_w(px(220.0))
                .item(
                    PopupMenuItem::new(label_shortcuts.clone())
                        .on_click(move |_, _, cx| {
                            let text = sc_text.clone();
                            m_sc.update(cx, |st, cx| {
                                st.toast = Some(Toast::info(text));
                                cx.notify();
                            });
                        }),
                )
                .item(
                    PopupMenuItem::new(label_evfl.clone())
                        .on_click(move |_, _, cx| {
                            let text = ev_text.clone();
                            m_ev.update(cx, |st, cx| {
                                st.toast = Some(Toast::info(text));
                                cx.notify();
                            });
                        }),
                )
                .separator()
                .item(
                    PopupMenuItem::new(label_lang.clone())
                        .on_click(move |_, _, cx| {
                            m_lg.update(cx, |st, cx| {
                                st.lang = match st.lang {
                                    crate::i18n::Language::Zh => crate::i18n::Language::En,
                                    crate::i18n::Language::En => crate::i18n::Language::Zh,
                                };
                                cx.notify();
                            });
                        }),
                )
                .separator()
                .item(
                    PopupMenuItem::new(label_about.clone())
                        .on_click(move |_, _, cx| {
                            let text = ab_text.clone();
                            m_ab.update(cx, |st, cx| {
                                st.toast = Some(Toast::info(text));
                                cx.notify();
                            });
                        }),
                )
        })
}
