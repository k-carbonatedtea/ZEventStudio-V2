use gpui::*;
use crate::ui::state::AppState;
use crate::ui::settings::switch_pane::render_path_field;
use crate::ui::state_settings::browse_platform_folder;
use crate::model::PlatformType;
use crate::ui::state::ActiveSettingsField;

/// 渲染 Wii U 设置面板
pub fn render_wiiu_pane(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();

    div()
        .id("settings_wiiu_scroll")
        .flex_1()
        .flex()
        .flex_col()
        .gap_4()
        .overflow_y_scroll()
        .p_1()
        // 顶部信息警告卡片 (WiiU 模拟器跨平台路径问题提示)
        .child(
            div()
                .p_3()
                .rounded_xl()
                .bg(pal.secondary_accent.opacity(0.1))
                .border_1()
                .border_color(pal.secondary_accent.opacity(0.4))
                .flex()
                .items_start()
                .gap_3()
                .child(
                    div()
                        .text_size(px(16.0))
                        .text_color(pal.secondary_accent)
                        .child("⚠️"),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(12.5))
                                .text_color(pal.secondary_accent)
                                .child(state.t("settings.wiiu_warn_title")),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .child(state.t("settings.wiiu_warn_desc")),
                        ),
                ),
        )
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
