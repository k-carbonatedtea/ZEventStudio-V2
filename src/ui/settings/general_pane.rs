use gpui::*;
use crate::model::{GameLanguage, PlatformType};
use crate::ui::state::AppState;

/// 渲染通用偏好设置面板 (General Settings Pane)
pub fn render_general_pane(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    div()
        .id("settings_general_scroll")
        .flex_1()
        .flex()
        .flex_col()
        .gap_4()
        .overflow_y_scroll()
        .p_1()
        // 1. 界面显示语言设置
        .child(render_ui_language_card(state, model))
        // 2. 默认运行主平台 (Switch RomFS vs Wii U Content)
        .child(render_platform_card(state, model))
        // 3. 游戏台词多语言包 (BotW / TotK MSBT 语言)
        .child(render_game_language_card(state, model))
        // 4. BCML 路径一键同步
        .child(render_bcml_sync_card(state, model))
        // 5. 临时缓存与自动保存快照清理
        .child(render_storage_cache_card(state, model))
}

/// 渲染 UI 界面语言选择卡片
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
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                .child(render_choice_chip(
                    "简体中文 (Simplified Chinese)",
                    current_lang == crate::i18n::Language::Zh,
                    &pal,
                    move |cx| {
                        model_zh.update(cx, |st, cx| {
                            st.lang = crate::i18n::Language::Zh;
                            cx.notify();
                        });
                    },
                ))
                .child(render_choice_chip(
                    "English (US / Global)",
                    current_lang == crate::i18n::Language::En,
                    &pal,
                    move |cx| {
                        model_en.update(cx, |st, cx| {
                            st.lang = crate::i18n::Language::En;
                            cx.notify();
                        });
                    },
                )),
        )
}

/// 渲染默认主平台单选卡片组 (Switch RomFS / Wii U Content)
fn render_platform_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let current_platform = state.settings_draft.current_platform;
    let model_sw = model.clone();
    let model_wu = model.clone();

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
                .child(div().text_size(px(16.0)).child("🎮"))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.0))
                                .text_color(pal.text_primary)
                                .child(state.t("settings.default_platform")),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .child(state.t("settings.default_platform_desc")),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .gap_3()
                .child(render_platform_radio_box(
                    "Nintendo Switch (NX)",
                    state.t("settings.platform_switch_sub"),
                    "🎮",
                    current_platform == PlatformType::Switch,
                    &pal,
                    move |cx| {
                        model_sw.update(cx, |st, cx| {
                            st.settings_draft.current_platform = PlatformType::Switch;
                            cx.notify();
                        });
                    },
                ))
                .child(render_platform_radio_box(
                    "Wii U (Cemu)",
                    state.t("settings.platform_wiiu_sub"),
                    "🖥️",
                    current_platform == PlatformType::WiiU,
                    &pal,
                    move |cx| {
                        model_wu.update(cx, |st, cx| {
                            st.settings_draft.current_platform = PlatformType::WiiU;
                            cx.notify();
                        });
                    },
                )),
        )
}

/// 渲染平台选择大单选框
fn render_platform_radio_box<F>(
    title: &'static str,
    subtitle: &'static str,
    icon: &'static str,
    is_active: bool,
    pal: &crate::ui::theme::ThemePalette,
    on_select: F,
) -> impl IntoElement
where
    F: Fn(&mut App) + 'static,
{
    let bg = if is_active {
        pal.primary_accent.opacity(0.16)
    } else {
        pal.app_bg
    };
    let border = if is_active {
        pal.primary_accent
    } else {
        pal.panel_border
    };
    let title_color = if is_active {
        pal.primary_accent
    } else {
        pal.text_primary
    };

    div()
        .flex_1()
        .p_3()
        .rounded_lg()
        .bg(bg)
        .border_1()
        .border_color(border)
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
                .items_center()
                .gap_2p5()
                .child(div().text_size(px(18.0)).child(icon))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(12.5))
                                .text_color(title_color)
                                .child(title),
                        )
                        .child(
                            div()
                                .text_size(px(10.5))
                                .text_color(pal.text_secondary)
                                .child(subtitle),
                        ),
                ),
        )
        .child(
            if is_active {
                div()
                    .size(px(18.0))
                    .rounded_full()
                    .bg(pal.primary_accent)
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(11.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(if pal.is_dark { rgb(0x0A0C10) } else { rgb(0xFFFFFF) })
                    .child("✓")
            } else {
                div()
                    .size(px(18.0))
                    .rounded_full()
                    .border_1()
                    .border_color(pal.card_border)
            },
        )
}

/// 渲染游戏台词语言包卡片
fn render_game_language_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let current_game_lang = state.settings_draft.language;

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
                                st.settings_draft.language = gl;
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

/// 渲染 BCML 路径一键读取与同步卡片
fn render_bcml_sync_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let model_sync = model.clone();

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
                .child(div().text_size(px(16.0)).child("📁"))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.0))
                                .text_color(pal.text_primary)
                                .child(state.t("settings.import_bcml")),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .child(state.t("settings.import_bcml_desc")),
                        ),
                ),
        )
        .child(
            div()
                .h(px(30.0))
                .px_3()
                .rounded_md()
                .bg(hsla(152.0, 0.65, 0.45, 0.15))
                .border_1()
                .border_color(hsla(152.0, 0.65, 0.45, 0.45))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .text_size(px(11.5))
                .font_weight(FontWeight::BOLD)
                .text_color(hsla(152.0, 0.75, 0.60, 1.0))
                .hover(|s| s.bg(hsla(152.0, 0.65, 0.45, 0.28)))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    model_sync.update(cx, |st, cx| {
                        st.sync_from_bcml();
                        cx.notify();
                    });
                })
                .child(state.t("settings.import_bcml_btn")),
        )
}

/// 渲染临时自动保存历史与快照清理卡片
fn render_storage_cache_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let model_clean = model.clone();

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
                .child(div().text_size(px(16.0)).child("💾"))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.0))
                                .text_color(pal.text_primary)
                                .child(state.t("settings.storage_cache")),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .child(state.t("settings.storage_cache_desc")),
                        ),
                ),
        )
        .child(
            div()
                .h(px(30.0))
                .px_3()
                .rounded_md()
                .bg(hsla(0.0, 0.65, 0.45, 0.15))
                .border_1()
                .border_color(hsla(0.0, 0.65, 0.45, 0.45))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .text_size(px(11.5))
                .text_color(hsla(0.0, 0.75, 0.70, 1.0))
                .hover(|s| s.bg(hsla(0.0, 0.65, 0.45, 0.28)))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    model_clean.update(cx, |st, cx| {
                        st.clean_autosave_cache();
                        cx.notify();
                    });
                })
                .child(state.t("settings.clean_autosave_btn")),
        )
}

/// 渲染选项单选胶囊
pub fn render_choice_chip<F>(
    title: &'static str,
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
        .h(px(38.0))
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
                .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
                .text_size(px(12.0))
                .text_color(text_color)
                .child(title),
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
