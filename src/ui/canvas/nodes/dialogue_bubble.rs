use gpui::*;
use crate::model::dialogue::DialogueSource;
use crate::model::{extract_node_message_target, resolve_node_dialogue, NodeData};
use crate::ui::state::AppState;

/// 渲染事件卡片中的游戏本地化对白预览卡片
pub fn render_dialogue_preview_card(
    state: &AppState,
    node: &NodeData,
    z: f32,
    is_expanded: bool,
) -> Option<AnyElement> {
    let (file_key, label) = extract_node_message_target(node)?;

    // 缩放过小时隐藏详细文本以优化渲染开销
    if z < 0.28 {
        return None;
    }

    let dialogue = resolve_node_dialogue(
        node,
        state.game_lang,
        state.project_path.as_deref(),
        &state.game_paths,
    );

    let pal = state.theme();
    let font_header = (8.5 * z).clamp(2.0, 12.0);
    let font_text = (9.0 * z).clamp(2.0, 13.0);
    let pad = (5.0 * z).clamp(2.0, 8.0);
    let radius = (6.0 * z).clamp(2.0, 8.0);

    let short_file = file_key.split('/').next_back().unwrap_or(&file_key);
    let lang_code = state.game_lang.code();

    Some(
        div()
            .mt(px((4.0 * z).max(2.0)))
            .p(px(pad))
            .rounded(px(radius))
            .bg(hsla(220.0 / 360.0, 0.45, 0.11, 0.95))
            .border_1()
            .border_color(hsla(215.0 / 360.0, 0.35, 0.32, 0.75))
            .flex()
            .flex_col()
            .gap(px((3.0 * z).max(1.0)))
            // 1. 标题栏：对白图标 + 文件与词条标识 + 来源徽章
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .pb(px((2.0 * z).max(1.0)))
                    .border_b_1()
                    .border_color(hsla(215.0 / 360.0, 0.25, 0.25, 0.60))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px((4.0 * z).max(1.0)))
                            .text_size(px(font_header))
                            .child(div().child("💬"))
                            .child(
                                div()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(pal.secondary_accent)
                                    .child(format!("{}: {}", short_file, label)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px((3.0 * z).max(1.0)))
                            // 语言徽章 (如 [CNzh])
                            .child(
                                div()
                                    .px(px((3.5 * z).max(1.0)))
                                    .py(px((0.8 * z).max(0.5)))
                                    .rounded_sm()
                                    .bg(hsla(215.0 / 360.0, 0.30, 0.25, 0.80))
                                    .text_size(px(font_header * 0.9))
                                    .text_color(pal.primary_accent)
                                    .child(lang_code),
                            )
                            // 来源标识徽章 (如 [项目] 或 [Dump])
                            .children(dialogue.as_ref().map(|item| {
                                let (source_label, badge_bg, badge_fg) = match item.source {
                                    DialogueSource::Project => (
                                        state.t("node.dialogue_source_project"),
                                        hsla(160.0 / 360.0, 0.40, 0.20, 0.85),
                                        hsla(160.0 / 360.0, 0.90, 0.70, 1.0),
                                    ),
                                    DialogueSource::GameDump => (
                                        state.t("node.dialogue_source_dump"),
                                        hsla(38.0 / 360.0, 0.40, 0.20, 0.85),
                                        hsla(38.0 / 360.0, 0.95, 0.70, 1.0),
                                    ),
                                };
                                div()
                                    .px(px((3.5 * z).max(1.0)))
                                    .py(px((0.8 * z).max(0.5)))
                                    .rounded_sm()
                                    .bg(badge_bg)
                                    .text_size(px(font_header * 0.9))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(badge_fg)
                                    .child(source_label)
                            })),
                    ),
            )
            // 2. 对白正文内容
            .child(if let Some(item) = dialogue {
                let lines: Vec<&str> = item.text.lines().collect();
                let max_lines = if is_expanded { 6 } else { 3 };
                div()
                    .flex()
                    .flex_col()
                    .gap(px((1.5 * z).max(0.5)))
                    .text_size(px(font_text))
                    .line_height(px((13.5 * z).max(4.0)))
                    .text_color(rgb(0xF1F5F9))
                    .children(lines.into_iter().take(max_lines).map(|line| {
                        div().child(line.to_string())
                    }))
                    .into_any_element()
            } else {
                div()
                    .text_size(px(font_text * 0.9))
                    .text_color(pal.text_secondary)
                    .italic()
                    .child(state.t("node.dialogue_not_found"))
                    .into_any_element()
            })
            .into_any_element(),
    )
}
