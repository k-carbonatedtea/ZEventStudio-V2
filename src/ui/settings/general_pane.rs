use gpui::*;
use crate::model::PlatformType;
use crate::ui::state::AppState;
use super::general_helpers::{
    render_bcml_sync_card, render_choice_chip, render_game_language_card,
    render_platform_radio_box, render_storage_cache_card,
};

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
                .flex_wrap()
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
