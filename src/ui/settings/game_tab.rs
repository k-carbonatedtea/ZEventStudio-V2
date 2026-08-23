use gpui::prelude::FluentBuilder;
use gpui::*;
use crate::ui::state::AppState;
use crate::ui::state_settings::TargetEngine;

/// 渲染游戏与封包导出环境设置面板 (Game & Mod Environment Settings Tab)
pub fn render_game_tab(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    div()
        .id("settings_game_scroll")
        .flex_1()
        .flex()
        .flex_col()
        .gap_4()
        .overflow_y_scroll()
        .p_1()
        // 1. 目标游戏引擎版本
        .child(render_target_engine_card(state, model))
        // 2. 默认二进制字节序
        .child(render_endian_card(state))
        // 3. 安全备份与保存策略
        .child(render_backup_card(state, model))
}

/// 渲染目标游戏版本设置卡片
fn render_target_engine_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let current_engine = state.target_engine;

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
                                .child(state.t("settings.engine_title")),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .child(state.t("settings.engine_desc")),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .gap_2p5()
                .child(render_game_choice_chip(
                    state.t("settings.engine_botw"),
                    "🗡️",
                    current_engine == TargetEngine::BotW,
                    &pal,
                    {
                        let m = model.clone();
                        move |cx| {
                            m.update(cx, |st, cx| {
                                st.target_engine = TargetEngine::BotW;
                                cx.notify();
                            });
                        }
                    },
                ))
                .child(render_game_choice_chip(
                    state.t("settings.engine_totk"),
                    "🏹",
                    current_engine == TargetEngine::TotK,
                    &pal,
                    {
                        let m = model.clone();
                        move |cx| {
                            m.update(cx, |st, cx| {
                                st.target_engine = TargetEngine::TotK;
                                cx.notify();
                            });
                        }
                    },
                )),
        )
}

/// 渲染默认二进制字节序说明卡片
fn render_endian_card(state: &AppState) -> impl IntoElement {
    let pal = state.theme();

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
                                .child(state.t("settings.endian_title")),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .child(state.t("settings.endian_desc")),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .gap_2p5()
                .child(
                    div()
                        .flex_1()
                        .h(px(38.0))
                        .px_3()
                        .rounded_lg()
                        .bg(pal.primary_accent.opacity(0.18))
                        .border_1()
                        .border_color(pal.primary_accent)
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(12.0))
                                .text_color(pal.primary_accent)
                                .child(state.t("settings.endian_little")),
                        )
                        .child(
                            div()
                                .text_size(px(11.5))
                                .text_color(pal.primary_accent)
                                .child("✓"),
                        ),
                )
                .child(
                    div()
                        .flex_1()
                        .h(px(38.0))
                        .px_3()
                        .rounded_lg()
                        .bg(pal.app_bg)
                        .border_1()
                        .border_color(pal.panel_border)
                        .flex()
                        .items_center()
                        .child(
                            div()
                                .text_size(px(12.0))
                                .text_color(pal.text_secondary)
                                .child(state.t("settings.endian_big")),
                        ),
                ),
        )
}

/// 渲染保存时自动备份开关卡片
fn render_backup_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let is_backup_enabled = state.auto_backup;
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
                .child(div().text_size(px(16.0)).child("🛡️"))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.0))
                                .text_color(pal.text_primary)
                                .child(state.t("settings.backup_title")),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .child(state.t("settings.backup_desc")),
                        ),
                ),
        )
        .child(
            div()
                .w(px(40.0))
                .h(px(22.0))
                .rounded_full()
                .bg(if is_backup_enabled { pal.primary_accent } else { pal.app_bg })
                .border_1()
                .border_color(if is_backup_enabled { pal.primary_accent } else { pal.card_border })
                .p_0p5()
                .flex()
                .items_center()
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    model_toggle.update(cx, |st, cx| {
                        st.auto_backup = !st.auto_backup;
                        cx.notify();
                    });
                })
                .child(
                    div()
                        .size(px(16.0))
                        .rounded_full()
                        .bg(if is_backup_enabled { rgb(0x0A0C10) } else { pal.text_secondary })
                        .when(is_backup_enabled, |s| s.ml(px(18.0))),
                ),
        )
}

/// 渲染选项单选胶囊
fn render_game_choice_chip<F>(
    label: &str,
    icon: &'static str,
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
        pal.text_secondary
    };

    div()
        .flex_1()
        .h(px(38.0))
        .px_3()
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
                .items_center()
                .gap_2()
                .child(div().font_weight(FontWeight::BOLD).text_size(px(12.0)).child(icon))
                .child(
                    div()
                        .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
                        .text_size(px(12.0))
                        .text_color(text_color)
                        .child(label.to_string()),
                ),
        )
        .child(
            if is_active {
                div()
                    .text_size(px(11.5))
                    .text_color(pal.primary_accent)
                    .child("✓")
            } else {
                div()
            },
        )
}
