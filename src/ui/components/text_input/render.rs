use gpui::*;
use super::ime::render_blinking_cursor;
use super::key::find_word_range_at;
use crate::ui::state::AppState;

/// 渲染具备字符级光标、鼠标划选、IME 预输入与点击寻址的高级交互文本输入组件
#[allow(clippy::too_many_arguments)]
pub fn render_interactive_input_text(
    text: &str,
    is_focused: bool,
    cursor: usize,
    select_range: Option<(usize, usize)>,
    marked_range: Option<(usize, usize)>,
    cursor_color: Hsla,
    text_color: Rgba,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let chars: Vec<char> = text.chars().collect();
    let total = chars.len();
    let cur = cursor.min(total);

    let mut elements = Vec::with_capacity(total + 2);

    for (idx, &ch) in chars.iter().enumerate() {
        if is_focused && idx == cur {
            elements.push(render_blinking_cursor(cursor_color).into_any_element());
        }

        let is_selected = is_focused && select_range.is_some_and(|(s, e)| idx >= s && idx < e);
        let is_marked = is_focused && marked_range.is_some_and(|(s, e)| idx >= s && idx < e);

        let mut ch_el = div()
            .text_color(text_color)
            .cursor_text()
            .child(ch.to_string());

        if is_focused {
            let model_char = model.clone();
            let model_move = model.clone();
            let chars_clone = chars.clone();
            ch_el = ch_el
                .on_mouse_down(MouseButton::Left, move |e, _, cx| {
                    cx.stop_propagation();
                    let click_count = e.click_count;
                    let chs = chars_clone.clone();
                    model_char.update(cx, |st, cx| {
                        st.text_input_marked_range = None;
                        if click_count >= 2 {
                            let (w_s, w_e) = find_word_range_at(&chs, idx);
                            st.text_input_select_anchor = Some(w_s);
                            st.text_input_cursor = w_e;
                        } else {
                            st.text_input_cursor = idx;
                            st.text_input_select_anchor = Some(idx);
                            st.text_input_is_dragging = true;
                        }
                        cx.notify();
                    });
                })
                .on_mouse_move(move |_, _, cx| {
                    model_move.update(cx, |st, cx| {
                        if st.text_input_is_dragging && st.text_input_cursor != idx {
                            st.text_input_cursor = idx;
                            cx.notify();
                        }
                    });
                });
        }

        if is_selected {
            ch_el = ch_el.bg(rgba(0x3B82F655)).rounded_xs();
        } else if is_marked {
            ch_el = ch_el.border_b_1().border_color(cursor_color);
        }

        elements.push(ch_el.into_any_element());
    }

    if is_focused && cur == total {
        elements.push(render_blinking_cursor(cursor_color).into_any_element());
    }

    div().flex().items_center().children(elements)
}

/// 快速渲染单行输入框文本与光标 (自动拉取 AppState 主题与光标/选区/IME组合态)
pub fn render_input_text(
    text: &str,
    is_focused: bool,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    render_interactive_input_text(
        text,
        is_focused,
        if is_focused { state.text_input_cursor } else { 0 },
        if is_focused { state.text_input_selection_range() } else { None },
        if is_focused { state.text_input_marked_range } else { None },
        pal.primary_accent.into(),
        pal.text_primary,
        model,
    )
}
