use gpui::prelude::FluentBuilder;
use gpui::*;
use crate::model::GameLanguage;
use crate::ui::state::AppState;

/// 渲染常规与多语言设置面板 (General & Localization Settings Tab)
pub fn render_general_tab(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    div()
        .id("settings_general_scroll")
        .flex_1()
        .flex()
        .flex_col()
        .gap_4()
        .overflow_y_scroll()
        .p_1()
        // 1. UI 界面语言配置卡片
        .child(render_ui_language_card(state, model))
        // 2. 游戏文本与对白本地化语言配置卡片
        .child(render_game_language_card(state, model))
        // 3. 启动行为设置卡片
        .child(render_startup_card(state, model))
}

/// 渲染 UI 界面语言设置卡片
fn render_ui_language_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let current_lang = state.lang;
    let model_zh = model.clone();
    let model_en = model.clone();

    div()
        .p_4()
        .rounded_xl()
        .bg(pal.card_surface)
        .border_1()
        .border_color(pal.card_border)
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2p5()
                        .child(div().text_size(px(16.0)).child("🌐"))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .font_weight(FontWeight::BOLD)
                                        .text_size(px(13.0))
                                        .text_color(pal.text_primary)
                                        .child(state.t("settings.ui_lang_title")),
                                )
                                .child(
                                    div()
                                        .text_size(px(11.0))
                                        .text_color(pal.text_secondary)
                                        .child(state.t("settings.ui_lang_desc")),
                                ),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                .child(
                    render_lang_option_chip(
                        "简体中文",
                        "Chinese",
                        current_lang == crate::i18n::Language::Zh,
                        &pal,
                        move |cx| {
                            model_zh.update(cx, |st, cx| {
                                st.lang = crate::i18n::Language::Zh;
                                cx.notify();
                            });
                        },
                    ),
                )
                .child(
                    render_lang_option_chip(
                        "English",
                        "English",
                        current_lang == crate::i18n::Language::En,
                        &pal,
                        move |cx| {
                            model_en.update(cx, |st, cx| {
                                st.lang = crate::i18n::Language::En;
                                cx.notify();
                            });
                        },
                    ),
                ),
        )
}

/// 渲染可切换语言选项胶囊
fn render_lang_option_chip<F>(
    title: &'static str,
    subtitle: &'static str,
    is_active: bool,
    pal: &crate::ui::theme::ThemePalette,
    on_select: F,
) -> impl IntoElement
where
    F: Fn(&mut App) + 'static,
{
    let bg_color = if is_active {
        pal.primary_accent.opacity(0.18)
    } else {
        pal.app_bg
    };
    let border_color = if is_active {
        pal.primary_accent
    } else {
        pal.panel_border
    };
    let text_color = if is_active {
        pal.primary_accent
    } else {
        pal.text_primary
    };

    div()
        .flex_1()
        .h(px(42.0))
        .px_3p5()
        .rounded_lg()
        .bg(bg_color)
        .border_1()
        .border_color(border_color)
        .flex()
        .items_center()
        .justify_between()
        .cursor_pointer()
        .hover(|s| s.border_color(pal.primary_accent))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            cx.stop_propagation();
            on_select(cx);
        })
        .child(
            div()
                .flex()
                .flex_col()
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(12.5))
                        .text_color(text_color)
                        .child(title),
                )
                .child(
                    div()
                        .text_size(px(10.0))
                        .text_color(pal.text_secondary)
                        .child(subtitle),
                ),
        )
        .child(
            if is_active {
                div()
                    .text_size(px(12.0))
                    .text_color(pal.primary_accent)
                    .child("✓")
            } else {
                div()
            },
        )
}

/// 渲染游戏文本语言 (BotW / TotK MSBT) 配置卡片
fn render_game_language_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let current_game_lang = state.game_lang;

    div()
        .p_4()
        .rounded_xl()
        .bg(pal.card_surface)
        .border_1()
        .border_color(pal.card_border)
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                .child(div().text_size(px(16.0)).child("💬"))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.0))
                                .text_color(pal.text_primary)
                                .child(state.t("settings.game_lang_title")),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .child(state.t("settings.game_lang_desc")),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_2()
                .children(GameLanguage::all().iter().map(|&gl| {
                    let is_active = current_game_lang == gl;
                    let m = model.clone();
                    let chip_bg = if is_active {
                        pal.primary_accent.opacity(0.18)
                    } else {
                        pal.app_bg
                    };
                    let chip_border = if is_active {
                        pal.primary_accent
                    } else {
                        pal.panel_border
                    };
                    let chip_text = if is_active {
                        pal.primary_accent
                    } else {
                        pal.text_secondary
                    };

                    div()
                        .px_3()
                        .py_1p5()
                        .rounded_md()
                        .bg(chip_bg)
                        .border_1()
                        .border_color(chip_border)
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .cursor_pointer()
                        .hover(|s| s.border_color(pal.primary_accent))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            m.update(cx, |st, cx| {
                                st.game_lang = gl;
                                cx.notify();
                            });
                        })
                        .child(
                            div()
                                .text_size(px(11.5))
                                .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
                                .text_color(chip_text)
                                .child(gl.display_name()),
                        )
                })),
        )
}

/// 渲染启动行为卡片
fn render_startup_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let is_welcome_enabled = state.show_welcome_on_startup;
    let model_toggle = model.clone();

    div()
        .p_4()
        .rounded_xl()
        .bg(pal.card_surface)
        .border_1()
        .border_color(pal.card_border)
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                .child(div().text_size(px(16.0)).child("🚀"))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.0))
                                .text_color(pal.text_primary)
                                .child(state.t("settings.startup_welcome")),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .child(state.t("settings.startup_welcome_desc")),
                        ),
                ),
        )
        .child(
            div()
                .w(px(40.0))
                .h(px(22.0))
                .rounded_full()
                .bg(if is_welcome_enabled { pal.primary_accent } else { pal.app_bg })
                .border_1()
                .border_color(if is_welcome_enabled { pal.primary_accent } else { pal.card_border })
                .p_0p5()
                .flex()
                .items_center()
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    model_toggle.update(cx, |st, cx| {
                        st.show_welcome_on_startup = !st.show_welcome_on_startup;
                        cx.notify();
                    });
                })
                .child(
                    div()
                        .size(px(16.0))
                        .rounded_full()
                        .bg(if is_welcome_enabled { rgb(0x0A0C10) } else { pal.text_secondary })
                        .when(is_welcome_enabled, |s| s.ml(px(18.0))),
                ),
        )
}
