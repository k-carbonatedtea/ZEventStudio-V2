use gpui::*;
use crate::model::PlatformType;
use crate::ui::state::AppState;
use crate::ui::state_settings::SettingsTab;

/// 渲染左侧分类选项卡列表
pub fn render_settings_category_sidebar(
    state: &AppState,
    model: &Entity<AppState>,
    current_tab: SettingsTab,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let cur_plat = state.settings_draft.current_platform;
    let categories = [
        (SettingsTab::General, "⚙️", state.t("settings.tab_general"), false),
        (SettingsTab::GameAi, "🤖", state.t("settings.tab_game_ai"), false),
        (
            SettingsTab::Switch,
            "🎮",
            state.t("settings.tab_switch"),
            cur_plat == PlatformType::Switch,
        ),
        (
            SettingsTab::WiiU,
            "🖥️",
            state.t("settings.tab_wiiu"),
            cur_plat == PlatformType::WiiU,
        ),
        (SettingsTab::Canvas, "🎨", state.t("settings.tab_canvas"), false),
    ];

    div()
        .w(px(215.0))
        .h_full()
        .bg(pal.app_bg)
        .border_r_1()
        .border_color(pal.panel_border)
        .p_3()
        .flex()
        .flex_col()
        .gap_1p5()
        .children(categories.iter().map(|&(tab, icon, label, is_active_plat)| {
            let is_selected = current_tab == tab;
            let m = model.clone();
            let bg_color = if is_selected {
                pal.primary_accent.opacity(0.18)
            } else {
                gpui::rgba(0x00000000)
            };
            let text_color = if is_selected {
                pal.primary_accent
            } else {
                pal.text_secondary
            };

            div()
                .h(px(38.0))
                .px_2p5()
                .rounded_lg()
                .bg(bg_color)
                .flex()
                .items_center()
                .justify_between()
                .gap_1p5()
                .cursor_pointer()
                .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    m.update(cx, |st, cx| {
                        st.settings_active_tab = tab;
                        if st.active_settings_field != crate::ui::state::ActiveSettingsField::None {
                            st.active_settings_field = crate::ui::state::ActiveSettingsField::None;
                        }
                        cx.notify();
                    });
                })
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().flex_shrink_0().text_size(px(14.0)).child(icon))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .font_weight(if is_selected {
                                    FontWeight::BOLD
                                } else {
                                    FontWeight::NORMAL
                                })
                                .text_size(px(12.0))
                                .text_color(text_color)
                                .child(label.to_string()),
                        ),
                )
                .child(
                    if is_active_plat {
                        div()
                            .flex_shrink_0()
                            .px_1p5()
                            .py_0p5()
                            .rounded_sm()
                            .bg(pal.primary_accent.opacity(0.15))
                            .text_size(px(9.5))
                            .font_weight(FontWeight::BOLD)
                            .text_color(pal.primary_accent)
                            .child("Active")
                            .into_any_element()
                    } else if is_selected {
                        div()
                            .flex_shrink_0()
                            .size(px(5.0))
                            .rounded_full()
                            .bg(pal.primary_accent)
                            .into_any_element()
                    } else {
                        div().into_any_element()
                    },
                )
        }))
}
