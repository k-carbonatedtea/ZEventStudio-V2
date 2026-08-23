use gpui::*;
use gpui_component::{
    button::{Button, ButtonVariants},
    menu::{DropdownMenu, PopupMenu, PopupMenuItem},
    Sizable,
};
use crate::model::GameLanguage;
use crate::ui::state::AppState;
use crate::ui::state_settings::SettingsTab;

/// 渲染「选项 (Options)」下拉菜单与子菜单 (UI语言, 游戏语言, 首选项设置)
pub fn render_options_menu(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let current_lang = state.lang;
    let current_game_lang = state.game_lang;

    let model_zh = model.clone();
    let model_en = model.clone();
    let model_game_lang = model.clone();
    let model_settings = model.clone();

    let label_ui_lang = state.t("menu.ui_language").to_string();
    let label_game_lang = state.t("menu.game_language").to_string();
    let label_settings = state.t("menu.settings").to_string();

    Button::new("titlebar_menu_options")
        .ghost()
        .xsmall()
        .label(state.t("menu.options"))
        .dropdown_menu(move |menu: PopupMenu, window, cx| {
            let m_zh = model_zh.clone();
            let m_en = model_en.clone();
            let m_gl = model_game_lang.clone();
            let m_set = model_settings.clone();

            menu.min_w(px(210.0))
                // 1. UI 界面语言子菜单
                .submenu(label_ui_lang.clone(), window, cx, move |sub, _win, _cx| {
                    let m_z = m_zh.clone();
                    let m_e = m_en.clone();
                    sub.min_w(px(180.0))
                        .item(
                            PopupMenuItem::new("简体中文 (Chinese)")
                                .checked(current_lang == crate::i18n::Language::Zh)
                                .on_click(move |_, _, cx| {
                                    m_z.update(cx, |st, cx| {
                                        st.lang = crate::i18n::Language::Zh;
                                        cx.notify();
                                    });
                                }),
                        )
                        .item(
                            PopupMenuItem::new("English (English)")
                                .checked(current_lang == crate::i18n::Language::En)
                                .on_click(move |_, _, cx| {
                                    m_e.update(cx, |st, cx| {
                                        st.lang = crate::i18n::Language::En;
                                        cx.notify();
                                    });
                                }),
                        )
                })
                // 2. 游戏文本语言子菜单
                .submenu(label_game_lang.clone(), window, cx, move |mut sub, _win, _cx| {
                    sub = sub.min_w(px(220.0));
                    for &gl in GameLanguage::all() {
                        let m = m_gl.clone();
                        let is_checked = current_game_lang == gl;
                        sub = sub.item(
                            PopupMenuItem::new(gl.display_name())
                                .checked(is_checked)
                                .on_click(move |_, _, cx| {
                                    m.update(cx, |st, cx| {
                                        st.game_lang = gl;
                                        cx.notify();
                                    });
                                }),
                        );
                    }
                    sub
                })
                .separator()
                // 3. 首选项与设置模态框入口
                .item(
                    PopupMenuItem::new(label_settings.clone())
                        .on_click(move |_, _, cx| {
                            m_set.update(cx, |st, cx| {
                                st.open_settings(SettingsTab::General);
                                cx.notify();
                            });
                        }),
                )
        })
}
