use gpui::*;
use crate::ui::components::Toast;
use crate::ui::state::AppState;

pub fn browse_ai_dump_folder(model: &Entity<AppState>, cx: &mut App) {
    let m = model.clone();
    cx.spawn(async move |cx| {
        let handle = rfd::AsyncFileDialog::new()
            .set_title("选择包含 .sbyml / .baiprog 的 AI 参数导出目录")
            .pick_folder()
            .await;

        if let Some(folder) = handle {
            let path_str = folder.path().to_string_lossy().to_string();
            cx.update(|cx| {
                m.update(cx, |st, cx| {
                    st.settings_draft.custom_ai_dump_dir = path_str;
                    crate::model::ai_def::AiCacheManager::global().clear();
                    cx.notify();
                });
            });
        }
    })
    .detach();
}

pub fn render_ai_dump_dir_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let path_val = state.settings_draft.custom_ai_dump_dir.clone();
    let m = model.clone();

    crate::ui::settings::switch_pane::render_path_field(
        state.t("settings.game_ai_dir_title"),
        state.t("settings.game_ai_dir_desc"),
        &path_val,
        "D:\\Game_Dump\\content\\Pack\\Bootup.pack (extracted)",
        crate::ui::state::ActiveSettingsField::AiDumpDir,
        state,
        model,
        move |cx| {
            browse_ai_dump_folder(&m, cx);
        },
    )
}

/// 渲染游戏 AI 当前状态与诊断卡片
pub fn render_ai_status_card(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let status = state.get_game_ai_status();
    let model_rescan = model.clone();

    let is_ready = status.is_loaded;
    let badge_bg = if is_ready {
        hsla(152.0 / 360.0, 0.70, 0.35, 0.25)
    } else {
        hsla(38.0 / 360.0, 0.85, 0.40, 0.25)
    };
    let badge_border = if is_ready {
        hsla(152.0 / 360.0, 0.75, 0.45, 0.6)
    } else {
        hsla(38.0 / 360.0, 0.85, 0.50, 0.6)
    };
    let badge_text = if is_ready {
        hsla(152.0 / 360.0, 0.85, 0.65, 1.0)
    } else {
        hsla(38.0 / 360.0, 0.95, 0.65, 1.0)
    };

    div()
        .p_4()
        .rounded_xl()
        .bg(pal.card_surface)
        .border_1()
        .border_color(pal.card_border)
        .flex()
        .flex_col()
        .gap_3()
        // 1. 卡片标题与状态标签
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
                        .child(div().text_size(px(16.0)).child("🤖"))
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(13.0))
                                .text_color(pal.text_primary)
                                .child(state.t("settings.game_ai_status_title")),
                        ),
                )
                .child(
                    div()
                        .px_2p5()
                        .py_0p5()
                        .rounded_full()
                        .bg(badge_bg)
                        .border_1()
                        .border_color(badge_border)
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .child(
                            div()
                                .w(px(6.0))
                                .h(px(6.0))
                                .rounded_full()
                                .bg(badge_text),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(badge_text)
                                .child(if is_ready {
                                    state.t("settings.game_ai_status_ready")
                                } else {
                                    state.t("settings.game_ai_status_not_found")
                                }),
                        ),
                ),
        )
        // 2. 状态详情信息
        .child(
            div()
                .p_3()
                .rounded_lg()
                .bg(pal.card_surface)
                .border_1()
                .border_color(pal.card_border)
                .flex()
                .flex_col()
                .gap_2()
                .children(if is_ready {
                    vec![
                        // 来源文件
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .text_size(px(11.5))
                            .child(
                                div()
                                    .text_color(pal.text_secondary)
                                    .child(state.t("settings.game_ai_status_source")),
                            )
                            .child(
                                div()
                                    .max_w(px(380.0))
                                    .overflow_hidden()
                                    .text_ellipsis()
                                    .text_color(pal.primary_accent)
                                    .font_family("Consolas")
                                    .child(status.source_path.clone()),
                            ),
                        // 动作 / 查询数量统计
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .text_size(px(11.5))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .child(div().text_color(pal.text_secondary).child(state.t("settings.game_ai_status_actions")))
                                    .child(div().font_weight(FontWeight::BOLD).text_color(pal.text_primary).child(format!("{}", status.actions_count))),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .child(div().text_color(pal.text_secondary).child(state.t("settings.game_ai_status_queries")))
                                    .child(div().font_weight(FontWeight::BOLD).text_color(pal.text_primary).child(format!("{}", status.queries_count))),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .child(div().text_color(pal.text_secondary).child(state.t("settings.game_ai_status_actors")))
                                    .child(div().font_weight(FontWeight::BOLD).text_color(pal.text_primary).child(format!("{}", status.actors_count))),
                            ),
                    ]
                } else {
                    vec![
                        div()
                            .text_size(px(11.5))
                            .text_color(pal.text_secondary)
                            .line_height(relative(1.5))
                            .child(state.t("settings.game_ai_status_help")),
                    ]
                }),
        )
        // 3. 重新扫描操作按钮
        .child(
            div()
                .flex()
                .justify_end()
                .child(
                    div()
                        .px_3()
                        .py_1p5()
                        .rounded_lg()
                        .bg(hsla(215.0 / 360.0, 0.25, 0.18, 1.0))
                        .hover(|s| s.bg(hsla(215.0 / 360.0, 0.40, 0.26, 1.0)))
                        .border_1()
                        .border_color(hsla(215.0 / 360.0, 0.35, 0.32, 1.0))
                        .cursor_pointer()
                        .text_size(px(11.5))
                        .text_color(pal.text_primary)
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_rescan.update(cx, |st, cx| {
                                crate::model::ai_def::AiCacheManager::global().clear();
                                let new_st = st.get_game_ai_status();
                                if new_st.is_loaded {
                                    st.toast = Some(Toast::success(format!(
                                        "{} (Actions: {}, Queries: {})",
                                        st.t("settings.game_ai_status_ready"),
                                        new_st.actions_count,
                                        new_st.queries_count
                                    )));
                                } else {
                                    st.toast = Some(Toast::warning(st.t("settings.game_ai_status_not_found")));
                                }
                                cx.notify();
                            });
                        })
                        .child(state.t("settings.game_ai_status_rescan")),
                ),
        )
}

/// 渲染关于游戏 AI 自动填充说明卡片
pub fn render_ai_autofill_info_card(state: &AppState) -> impl IntoElement {
    let pal = state.theme();

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
                .items_center()
                .gap_2p5()
                .child(div().text_size(px(16.0)).child("💡"))
                .child(
                    div()
                        .font_weight(FontWeight::BOLD)
                        .text_size(px(13.0))
                        .text_color(pal.text_primary)
                        .child(state.t("settings.game_ai_info_title")),
                ),
        )
        .child(
            div()
                .text_size(px(11.5))
                .text_color(pal.text_secondary)
                .line_height(relative(1.5))
                .child(state.t("settings.game_ai_info_desc")),
        )
}
