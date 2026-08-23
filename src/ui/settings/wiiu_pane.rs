use gpui::*;
use crate::ui::state::AppState;
use crate::ui::settings::switch_pane::render_path_field;
use crate::ui::state_settings::browse_platform_folder;
use crate::model::PlatformType;
use crate::ui::state::ActiveSettingsField;

/// 渲染 Wii U 设置面板
pub fn render_wiiu_pane(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let is_active = state.settings_draft.current_platform == PlatformType::WiiU;

    div()
        .id("settings_wiiu_scroll")
        .flex_1()
        .flex()
        .flex_col()
        .gap_4()
        .overflow_y_scroll()
        .p_1()
        // 顶部说明与生效激活状态栏
        .child(render_wiiu_tip_bar(state, model, is_active))
        // 1. 游戏本体目录 (Base Game)
        .child(render_path_field(
            &state.t("settings.wiiu_game_dir"),
            &state.t("settings.wiiu_game_dir_desc"),
            &state.settings_draft.wiiu.game_dir,
            "C:\\Cemu\\Games\\The Legend of Zelda Breath of the Wild\\content",
            ActiveSettingsField::WiiuGameDir,
            state,
            model,
            {
                let m = model.clone();
                move |cx| {
                    browse_platform_folder(&m, PlatformType::WiiU, "game_dir", "选择 Wii U 游戏本体目录", cx);
                }
            }
        ))
        // 2. 游戏更新目录 (Update)
        .child(render_path_field(
            &state.t("settings.wiiu_update_dir"),
            &state.t("settings.wiiu_update_dir_desc"),
            &state.settings_draft.wiiu.update_dir,
            "C:\\Cemu\\mlc01\\usr\\title\\00050000\\101C9500\\content",
            ActiveSettingsField::WiiuUpdateDir,
            state,
            model,
            {
                let m = model.clone();
                move |cx| {
                    browse_platform_folder(&m, PlatformType::WiiU, "update_dir", "选择 Wii U 游戏更新目录", cx);
                }
            }
        ))
        // 3. 游戏 DLC 目录 (DLC)
        .child(render_path_field(
            &state.t("settings.wiiu_dlc_dir"),
            &state.t("settings.wiiu_dlc_dir_desc"),
            &state.settings_draft.wiiu.dlc_dir,
            "C:\\Cemu\\mlc01\\usr\\title\\00050000\\101C9500\\aoc\\content",
            ActiveSettingsField::WiiuDlcDir,
            state,
            model,
            {
                let m = model.clone();
                move |cx| {
                    browse_platform_folder(&m, PlatformType::WiiU, "dlc_dir", "选择 Wii U 游戏 DLC 目录", cx);
                }
            }
        ))
        // 4. Mod 导出路径 (Export)
        .child(render_path_field(
            &state.t("settings.wiiu_export_dir"),
            &state.t("settings.wiiu_export_dir_desc"),
            &state.settings_draft.wiiu.export_dir,
            "C:\\Cemu\\graphicPacks\\ZeldaBOTW_Mods\\MyNodeFlowMod",
            ActiveSettingsField::WiiuExportDir,
            state,
            model,
            {
                let m = model.clone();
                move |cx| {
                    browse_platform_folder(&m, PlatformType::WiiU, "export_dir", "选择 Cemu 模组导出路径", cx);
                }
            }
        ))
}

/// 渲染顶部说明与生效状态条
fn render_wiiu_tip_bar(
    state: &AppState,
    model: &Entity<AppState>,
    is_active: bool,
) -> impl IntoElement {
    let pal = state.theme();
    let model_set = model.clone();

    div()
        .p_3p5()
        .rounded_xl()
        .bg(pal.card_surface)
        .border_1()
        .border_color(pal.card_border)
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .items_center()
                .gap_2()
                .child(div().flex_shrink_0().text_size(px(14.0)).child("💡"))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_size(px(11.5))
                        .text_color(pal.text_secondary)
                        .child(state.t("settings.wiiu_tip")),
                ),
        )
        .child(
            div()
                .flex_shrink_0()
                .child(
                    if is_active {
                        div()
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .bg(pal.primary_accent.opacity(0.18))
                            .border_1()
                            .border_color(pal.primary_accent)
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(pal.primary_accent)
                                    .child("✓"),
                            )
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(pal.primary_accent)
                                    .child(state.t("settings.active_badge")),
                            )
                            .into_any_element()
                    } else {
                        div()
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .bg(pal.app_bg)
                            .border_1()
                            .border_color(pal.panel_border)
                            .cursor_pointer()
                            .hover(|s| s.border_color(pal.primary_accent))
                            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                cx.stop_propagation();
                                model_set.update(cx, |st, cx| {
                                    st.settings_draft.current_platform = PlatformType::WiiU;
                                    cx.notify();
                                });
                            })
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .text_color(pal.text_secondary)
                                    .child(state.t("settings.set_active_btn")),
                            )
                            .into_any_element()
                    },
                ),
        )
}
