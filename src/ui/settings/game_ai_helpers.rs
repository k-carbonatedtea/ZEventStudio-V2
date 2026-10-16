use gpui::*;
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
        "游戏 AI 参数目录 (Game AI Directory)",
        "自定义覆盖查找字典: 用于优先搜索并解析 .sbyml / .baiprog 事件属性定义。若留空将回退在 Switch/Wii U 游戏目录下搜索",
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
