use gpui::*;
use crate::ui::state::AppState;
use crate::ui::views::msbt_editor::types::{BubbleType, MsbtEntry};

/// 渲染高精度塞尔达荒野之息 / 王国之泪游戏内风格的对话气泡预览区 (参考 Bubble-Wrap)
pub fn render_msbt_bubble_preview(
    entry: Option<&MsbtEntry>,
    bubble_type: BubbleType,
    state: &AppState,
) -> impl IntoElement {
    let pal = state.theme();

    let (_clean_text, line_count, char_count, anim, sound) = match entry {
        Some(e) => (
            e.clean_text.as_str(),
            e.line_count,
            e.char_count,
            e.animation.as_str(),
            e.sound.as_str(),
        ),
        None => ("", 0, 0, "none", "none"),
    };

    let line_limit = bubble_type.line_limit();
    let char_limit = bubble_type.char_limit();
    let is_line_overflow = line_count > line_limit;
    let is_char_overflow = char_limit.is_some_and(|lim| char_count > lim);
    let is_overflow = is_line_overflow || is_char_overflow;

    // 气泡卡片尺寸与圆角配置
    let (card_w, min_h, rounded_radius, pad_x, pad_y) = match bubble_type {
        BubbleType::Dialogue => (px(566.0), px(112.0), px(56.0), px(38.0), px(20.0)),
        BubbleType::Signboard => (px(566.0), px(112.0), px(8.0), px(36.0), px(20.0)),
        BubbleType::Item => (px(508.0), px(150.0), px(10.0), px(26.0), px(20.0)),
        BubbleType::Compendium => (px(508.0), px(240.0), px(10.0), px(26.0), px(20.0)),
        BubbleType::Quest => (px(508.0), px(280.0), px(10.0), px(26.0), px(26.0)),
        BubbleType::Tip => (px(634.0), px(100.0), px(10.0), px(30.0), px(18.0)),
    };

    div()
        .w_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .py_6()
        .px_4()
        .gap_3()
        // 1. 溢出告警条 (当超出游戏字数或建议行数限制时)
        .children(is_overflow.then(|| {
            div()
                .flex()
                .items_center()
                .gap_2()
                .px_3()
                .py_1()
                .rounded_md()
                .bg(rgba(0xEF444433))
                .border_1()
                .border_color(rgb(0xEF4444))
                .text_size(px(11.0))
                .text_color(rgb(0xFCA5A5))
                .child("⚠️")
                .child(if is_char_overflow {
                    format!(
                        "{} (当前: {} / 上限: {})",
                        state.t("msbt_editor.char_overflow_warning"),
                        char_count,
                        char_limit.unwrap_or(0)
                    )
                } else {
                    format!(
                        "{} (当前: {} / 建议: {})",
                        state.t("msbt_editor.line_overflow_warning"),
                        line_count,
                        line_limit
                    )
                })
        }))
        // 2. 核心气泡本体 (以游戏真实背景与圆角渲染)
        .child(
            div()
                .w(card_w)
                .min_h(min_h)
                .rounded(rounded_radius)
                .bg(if is_overflow {
                    rgba(0x450A0A99)
                } else {
                    rgba(0x0A0E14DD)
                })
                .border_1()
                .border_color(if is_overflow {
                    rgb(0xEF4444)
                } else {
                    rgba(0xFFFFFF1F)
                })
                .shadow_lg()
                .px(pad_x)
                .py(pad_y)
                .flex()
                .flex_col()
                .justify_center()
                .gap_2()
                // 气泡内部标签栏 (动作与音效提示)
                .children((anim != "none" || sound != "none").then(|| {
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .children((anim != "none").then(|| {
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .px_2()
                                .py_0p5()
                                .rounded_full()
                                .bg(rgba(0xFFFFFF15))
                                .text_size(px(10.0))
                                .text_color(rgb(0x38DFD8))
                                .child("🎭")
                                .child(format!("{}: {}", state.t("msbt_editor.anim_label"), anim))
                        }))
                        .children((sound != "none").then(|| {
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .px_2()
                                .py_0p5()
                                .rounded_full()
                                .bg(rgba(0xFFFFFF15))
                                .text_size(px(10.0))
                                .text_color(rgb(0xFCD34D))
                                .child("🔊")
                                .child(format!("{}: {}", state.t("msbt_editor.sound_label"), sound))
                        }))
                }))
                // 对白文本与富文本 Spans 渲染
                .child(
                    div()
                        .w_full()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .line_height(relative(1.35))
                        .children(if let Some(e) = entry {
                            if e.spans.is_empty() {
                                vec![div()
                                    .text_size(px(18.0))
                                    .text_color(rgb(0xF5F5EE))
                                    .child(if e.clean_text.is_empty() {
                                        state.t("msbt_editor.empty_text_placeholder").to_string()
                                    } else {
                                        e.clean_text.clone()
                                    })
                                    .into_any_element()]
                            } else {
                                e.spans
                                    .iter()
                                    .map(|span| {
                                        if span.is_pause {
                                            div()
                                                .mx_1()
                                                .px_1p5()
                                                .py_0p5()
                                                .rounded_sm()
                                                .bg(rgba(0xFFFFFF22))
                                                .border_1()
                                                .border_color(rgba(0xFFFFFF44))
                                                .text_size(px(10.0))
                                                .text_color(rgb(0xFCD34D))
                                                .child(format!("⏱️ {}", span.pause_label.as_deref().unwrap_or("停顿")))
                                                .into_any_element()
                                        } else if span.is_chip {
                                            div()
                                                .mx_1()
                                                .px_1p5()
                                                .py_0p5()
                                                .rounded_sm()
                                                .bg(rgba(0x38DFD822))
                                                .border_1()
                                                .border_color(rgba(0x38DFD855))
                                                .text_size(px(10.0))
                                                .text_color(rgb(0x38DFD8))
                                                .child(format!("⚡ {}", span.chip_label.as_deref().unwrap_or("控制")))
                                                .into_any_element()
                                        } else {
                                            div()
                                                .text_size(px(span.size.font_size_px()))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(span.color.rgb_color())
                                                .child(span.text.clone())
                                                .into_any_element()
                                        }
                                    })
                                    .collect()
                            }
                        } else {
                            vec![div()
                                .text_size(px(16.0))
                                .text_color(pal.text_secondary)
                                .child(state.t("msbt_editor.select_entry_hint"))
                                .into_any_element()]
                        }),
                ),
        )
        // 3. 统计状态胶囊：字符数、行数与建议限制
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .text_size(px(11.0))
                .text_color(pal.text_secondary)
                .child(
                    div()
                        .child(format!(
                            "{}: {} / {} {}",
                            state.t("msbt_editor.lines"),
                            line_count,
                            line_limit,
                            state.t("code_editor.lines")
                        )),
                )
                .child(div().child("•"))
                .child(
                    div()
                        .child(if let Some(lim) = char_limit {
                            format!("{}: {} / {} {}", state.t("msbt_editor.chars"), char_count, lim, state.t("code_editor.chars"))
                        } else {
                            format!("{}: {} {}", state.t("msbt_editor.chars"), char_count, state.t("code_editor.chars"))
                        }),
                ),
        )
}
