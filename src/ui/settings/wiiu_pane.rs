use gpui::*;
use crate::model::PlatformType;
use crate::ui::settings::switch_pane::render_path_field;
use crate::ui::state::AppState;
use crate::ui::state_settings::browse_wiiu_folder;

/// 渲染 Wii U (Cemu) 平台路径配置面板 (Wii U Settings Pane)
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
        // 1. 顶部平台说明与生效激活状态栏
        .child(render_wiiu_tip_bar(state, model, is_active))
        // 2. 游戏本体 Content 目录 (game_dir)
        .child(render_wiiu_game_dir_card(state, model))
        // 3. 补丁 Content 目录 (update_dir)
        .child(render_wiiu_update_dir_card(state, model))
        // 4. DLC 0010 Content 目录 (dlc_dir)
        .child(render_wiiu_dlc_dir_card(state, model))
        // 5. 模组导出目录 (export_dir)
        .child(render_wiiu_export_dir_card(state, model))
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

/// 渲染 Wii U 游戏本体 Content 目录卡片
fn render_wiiu_game_dir_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let path_val = state.settings_draft.wiiu.game_dir.clone();
    let m = model.clone();

    render_path_field(
        state.t("settings.wiiu_game_dir"),
        state.t("settings.wiiu_game_dir_desc"),
        &path_val,
        "E:\\Botw_Dump\\Games\\The Legend of Zelda-Breath of the Wild\\content",
        state,
        move |cx| {
            browse_wiiu_folder(&m, "game_dir", "选择 Wii U 本体 content 目录", cx);
        },
    )
}

/// 渲染 Wii U 补丁 Content 目录卡片
fn render_wiiu_update_dir_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let path_val = state.settings_draft.wiiu.update_dir.clone();
    let m = model.clone();

    render_path_field(
        state.t("settings.wiiu_update_dir"),
        state.t("settings.wiiu_update_dir_desc"),
        &path_val,
        "E:\\Botw_Dump\\MLC\\usr\title\\0005000E\\101C9300\\content",
        state,
        move |cx| {
            browse_wiiu_folder(&m, "update_dir", "选择 Wii U 补丁 content 目录", cx);
        },
    )
}

/// 渲染 Wii U DLC Content 目录卡片
fn render_wiiu_dlc_dir_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let path_val = state.settings_draft.wiiu.dlc_dir.clone();
    let m = model.clone();

    render_path_field(
        state.t("settings.wiiu_dlc_dir"),
        state.t("settings.wiiu_dlc_dir_desc"),
        &path_val,
        "E:\\Botw_Dump\\MLC\\usr\\title\\0005000c\\101c9300\\content\\0010",
        state,
        move |cx| {
            browse_wiiu_folder(&m, "dlc_dir", "选择 Wii U DLC content 目录", cx);
        },
    )
}

/// 渲染 Wii U 模组导出路径卡片
fn render_wiiu_export_dir_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let path_val = state.settings_draft.wiiu.export_dir.clone();
    let m = model.clone();

    render_path_field(
        state.t("settings.wiiu_export_dir"),
        state.t("settings.wiiu_export_dir_desc"),
        &path_val,
        "C:\\Users\\xxxx\\Documents\\EventEditor\\Wiiu",
        state,
        move |cx| {
            browse_wiiu_folder(&m, "export_dir", "选择 Wii U 模组导出路径", cx);
        },
    )
}
