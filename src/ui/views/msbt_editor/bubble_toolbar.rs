use gpui::*;
use crate::ui::state::AppState;
use crate::ui::views::msbt_editor::types::{BubbleType, MsbtEntry};

/// 渲染气泡编辑器顶部快捷工具栏 (气泡形态、预设动作、音效、停顿与格式标签快捷注入)
pub fn render_msbt_bubble_toolbar(
    active_entry: Option<&MsbtEntry>,
    bubble_type: BubbleType,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();

    div()
        .h(px(40.0))
        .w_full()
        .px_4()
        .bg(pal.panel_surface)
        .border_b_1()
        .border_color(pal.panel_border)
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .overflow_hidden()
        // 左侧：气泡形态选择器与预设动作/音效
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                // 气泡形态选择按钮组
                .child(
                    div()
                        .flex()
                        .items_center()
                        .rounded_md()
                        .bg(pal.card_surface)
                        .border_1()
                        .border_color(pal.card_border)
                        .p_0p5()
                        .gap_0p5()
                        .children(BubbleType::all().iter().map(|btype| {
                            let is_active = bubble_type == *btype;
                            let target_type = *btype;
                            let model_type = model.clone();

                            div()
                                .px_2()
                                .py_0p5()
                                .rounded_sm()
                                .cursor_pointer()
                                .text_size(px(10.5))
                                .font_weight(FontWeight::BOLD)
                                .bg(if is_active {
                                    pal.primary_accent
                                } else {
                                    rgba(0x00000000)
                                })
                                .text_color(if is_active {
                                    if pal.is_dark { rgb(0x0F1117) } else { rgb(0xFFFFFF) }
                                } else {
                                    pal.text_secondary
                                })
                                .hover(|s| s.opacity(0.85))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_type.update(cx, |st, cx| {
                                        st.msbt_bubble_type = target_type;
                                        cx.notify();
                                    });
                                })
                                .child(match btype {
                                    BubbleType::Dialogue => state.t("msbt_editor.type_dialogue"),
                                    BubbleType::Signboard => state.t("msbt_editor.type_signboard"),
                                    BubbleType::Item => state.t("msbt_editor.type_item"),
                                    BubbleType::Compendium => state.t("msbt_editor.type_compendium"),
                                    BubbleType::Quest => state.t("msbt_editor.type_quest"),
                                    BubbleType::Tip => state.t("msbt_editor.type_tip"),
                                })
                        })),
                ),
        )
        // 右侧：快速插入颜色、大小与停顿标签
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                // 红色标签
                .child(render_quick_tag_button(
                    "🔴 红字",
                    "<red>",
                    "</color>",
                    rgb(0xF84858),
                    active_entry.is_some(),
                    model,
                ))
                // 蓝色标签
                .child(render_quick_tag_button(
                    "🔷 蓝字",
                    "<blue>",
                    "</color>",
                    rgb(0x38DFD8),
                    active_entry.is_some(),
                    model,
                ))
                // 灰色标签
                .child(render_quick_tag_button(
                    "🔘 灰字",
                    "<grey>",
                    "</color>",
                    rgb(0xAFAFAA),
                    active_entry.is_some(),
                    model,
                ))
                // 短停顿
                .child(render_quick_insert_button(
                    "⏱️ 短停顿",
                    "<pause:short>",
                    active_entry.is_some(),
                    model,
                ))
                // 长停顿
                .child(render_quick_insert_button(
                    "⏱️ 长停顿",
                    "<pause:long>",
                    active_entry.is_some(),
                    model,
                ))
                // 大号字体 125%
                .child(render_quick_tag_button(
                    "📐 大字",
                    "<size:125>",
                    "</size>",
                    pal.text_primary,
                    active_entry.is_some(),
                    model,
                ))
                // 小号字体 80%
                .child(render_quick_tag_button(
                    "📏 小字",
                    "<size:80>",
                    "</size>",
                    pal.text_secondary,
                    active_entry.is_some(),
                    model,
                )),
        )
}

/// 渲染包裹式快捷格式标签按钮 (如 <red>...</color>)
fn render_quick_tag_button(
    label: &'static str,
    prefix: &'static str,
    suffix: &'static str,
    fg: Rgba,
    enabled: bool,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let model_tag = model.clone();
    div()
        .px_2()
        .py_1()
        .rounded_md()
        .bg(rgba(0xFFFFFF0D))
        .border_1()
        .border_color(rgba(0xFFFFFF1F))
        .text_size(px(10.5))
        .font_weight(FontWeight::BOLD)
        .text_color(if enabled { fg } else { rgb(0x6B7280) })
        .cursor_pointer()
        .hover(|s| s.bg(rgba(0xFFFFFF1F)))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            if enabled {
                model_tag.update(cx, |st, cx| {
                    st.wrap_msbt_draft_with_tag(prefix, suffix);
                    cx.notify();
                });
            }
        })
        .child(label)
}

/// 渲染插入式快捷标签按钮 (如 <pause:short>)
fn render_quick_insert_button(
    label: &'static str,
    snippet: &'static str,
    enabled: bool,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let model_insert = model.clone();
    div()
        .px_2()
        .py_1()
        .rounded_md()
        .bg(rgba(0xFFFFFF0D))
        .border_1()
        .border_color(rgba(0xFFFFFF1F))
        .text_size(px(10.5))
        .font_weight(FontWeight::BOLD)
        .text_color(if enabled { rgb(0xFCD34D) } else { rgb(0x6B7280) })
        .cursor_pointer()
        .hover(|s| s.bg(rgba(0xFFFFFF1F)))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            if enabled {
                model_insert.update(cx, |st, cx| {
                    st.insert_snippet_to_msbt_draft(snippet);
                    cx.notify();
                });
            }
        })
        .child(label)
}
