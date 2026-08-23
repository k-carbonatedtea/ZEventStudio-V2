use gpui::*;
use crate::model::PlatformType;
use crate::ui::state::AppState;
use crate::ui::state_settings::browse_switch_folder;

/// 渲染 Nintendo Switch (NX) 平台路径配置面板 (Switch Settings Pane)
pub fn render_switch_pane(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let is_active = state.settings_draft.current_platform == PlatformType::Switch;

    div()
        .id("settings_switch_scroll")
        .flex_1()
        .flex()
        .flex_col()
        .gap_4()
        .overflow_y_scroll()
        .p_1()
        // 1. 顶部平台说明与生效激活状态栏
        .child(render_switch_tip_bar(state, model, is_active))
        // 2. 游戏本体 RomFS 根目录 (game_dir_nx)
        .child(render_switch_game_dir_card(state, model))
        // 3. DLC RomFS 目录 (dlc_dir_nx)
        .child(render_switch_dlc_dir_card(state, model))
        // 4. Atmosphere LayeredFS 模组导出目录 (export_dir_nx)
        .child(render_switch_export_dir_card(state, model))
}

/// 渲染顶部说明与生效状态条
fn render_switch_tip_bar(
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
                        .child(state.t("settings.switch_tip")),
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
                                    st.settings_draft.current_platform = PlatformType::Switch;
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

/// 渲染 Switch 游戏 RomFS 目录输入与浏览卡片
fn render_switch_game_dir_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let path_val = state.settings_draft.switch.game_dir.clone();
    let m = model.clone();

    render_path_field(
        state.t("settings.switch_game_dir"),
        state.t("settings.switch_game_dir_desc"),
        &path_val,
        "C:\\Game\\BOTW\\01007EF00011E000\\romfs",
        crate::ui::state::ActiveSettingsField::SwitchGameDir,
        state,
        model,
        move |cx| {
            browse_switch_folder(&m, "game_dir", "选择 Switch RomFS 根目录", cx);
        },
    )
}

/// 渲染 Switch DLC RomFS 目录输入与浏览卡片
fn render_switch_dlc_dir_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let path_val = state.settings_draft.switch.dlc_dir.clone();
    let m = model.clone();

    render_path_field(
        state.t("settings.switch_dlc_dir"),
        state.t("settings.switch_dlc_dir_desc"),
        &path_val,
        "C:\\Game\\BOTW\\01007EF00011F001\\romfs",
        crate::ui::state::ActiveSettingsField::SwitchDlcDir,
        state,
        model,
        move |cx| {
            browse_switch_folder(&m, "dlc_dir", "选择 Switch DLC RomFS 目录", cx);
        },
    )
}

/// 渲染 Atmosphere 模组导出目录卡片
fn render_switch_export_dir_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let path_val = state.settings_draft.switch.export_dir.clone();
    let m = model.clone();

    render_path_field(
        state.t("settings.switch_export_dir"),
        state.t("settings.switch_export_dir_desc"),
        &path_val,
        "E:\\Atmosphere\\contents\\01007EF00011E000\\romfs",
        crate::ui::state::ActiveSettingsField::SwitchExportDir,
        state,
        model,
        move |cx| {
            browse_switch_folder(&m, "export_dir", "选择 Switch 模组导出路径", cx);
        },
    )
}

pub fn render_path_field<F>(
    title: &str,
    description: &str,
    current_val: &str,
    placeholder: &'static str,
    field_id: crate::ui::state::ActiveSettingsField,
    state: &AppState,
    model: &Entity<AppState>,
    on_browse: F,
) -> impl IntoElement
where
    F: Fn(&mut App) + 'static,
{
    let pal = state.theme();
    let is_empty = current_val.trim().is_empty();
    let is_active = state.active_settings_field == field_id;
    let m = model.clone();

    div()
        .p_4()
        .rounded_xl()
        .bg(pal.card_surface)
        .border_1()
        .border_color(pal.card_border)
        .flex()
        .flex_col()
        .gap_2p5()
        .child(
            div()
                .flex()
                .flex_col()
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(12.5))
                        .text_color(pal.text_primary)
                        .child(title.to_string()),
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .text_color(pal.text_secondary)
                        .child(description.to_string()),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                // 路径展示输入框区域
                .child(
                    div()
                        .flex_1()
                        .h(px(34.0))
                        .px_3()
                        .rounded_lg()
                        .bg(pal.app_bg)
                        .border_1()
                        .border_color(if is_active { pal.primary_accent } else { pal.panel_border })
                        .cursor_text()
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            m.update(cx, |st, cx| {
                                st.active_settings_field = field_id;
                                cx.notify();
                            });
                        })
                        .flex()
                        .items_center()
                        .overflow_hidden()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .text_size(px(11.5))
                                .text_color(if is_empty && !is_active { pal.text_secondary.opacity(0.6) } else { pal.text_primary })
                                .child(if is_empty && !is_active { placeholder.to_string() } else { current_val.to_string() })
                                .children(if is_active {
                                    Some(crate::ui::components::text_input::render_blinking_cursor(pal.primary_accent.into()))
                                } else {
                                    None
                                }),
                        ),
                )
                // 浏览文件夹选择按钮
                .child(
                    div()
                        .h(px(34.0))
                        .px_3p5()
                        .rounded_lg()
                        .bg(pal.primary_accent.opacity(0.12))
                        .border_1()
                        .border_color(pal.primary_accent.opacity(0.40))
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .cursor_pointer()
                        .hover(|s| s.bg(pal.primary_accent.opacity(0.24)))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            on_browse(cx);
                        })
                        .child(div().text_size(px(12.0)).child("📁"))
                        .child(
                            div()
                                .text_size(px(11.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(pal.primary_accent)
                                .child(state.t("settings.browse_btn")),
                        ),
                ),
        )
}
