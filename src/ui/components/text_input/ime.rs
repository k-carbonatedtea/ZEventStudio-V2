use gpui::*;
use std::ops::Range;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::ui::state::AppState;

static CARET_BOUNDS: RwLock<Option<Bounds<Pixels>>> = RwLock::new(None);

/// 更新输入光标在窗口客户区中的物理包围盒，供 Windows IME 输入法精确定位候选框
pub fn update_caret_bounds(bounds: Bounds<Pixels>) {
    if let Ok(mut lock) = CARET_BOUNDS.write() {
        *lock = Some(bounds);
    }
}

/// 获取当前活跃输入框光标位置
pub fn get_caret_bounds() -> Option<Bounds<Pixels>> {
    CARET_BOUNDS.read().ok().and_then(|lock| *lock)
}

/// UTF-16 偏移量转换为 Unicode 字符偏移量 (防止多字节或 Emoji 截断)
pub fn utf16_offset_to_char_offset(text: &str, offset_u16: usize) -> usize {
    let mut current_u16 = 0;
    for (char_idx, c) in text.chars().enumerate() {
        if current_u16 >= offset_u16 {
            return char_idx;
        }
        current_u16 += c.len_utf16();
    }
    text.chars().count()
}

/// Unicode 字符偏移量转换为 UTF-16 偏移量 (供 Windows IME 原生 TSF/IMM 对齐)
pub fn char_offset_to_utf16_offset(text: &str, char_offset: usize) -> usize {
    text.chars().take(char_offset).map(|c| c.len_utf16()).sum()
}

/// 渲染带平滑时间闪烁效果的输入光标，并实时追踪光标物理位置以对齐输入法候选框
pub fn render_blinking_cursor(color: Hsla) -> impl IntoElement {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let is_visible = (now_ms / 530).is_multiple_of(2);

    div()
        .w(px(2.0))
        .h(px(13.0))
        .rounded_xs()
        .bg(if is_visible { color } else { hsla(0.0, 0.0, 0.0, 0.0) })
        .child(
            canvas(
                |bounds, _window, _cx| bounds,
                move |bounds, _, _window, _cx| {
                    update_caret_bounds(Bounds {
                        origin: bounds.origin,
                        size: size(bounds.size.width, px(16.0)),
                    });
                },
            )
            .size_full(),
        )
}

/// GPUI 全局输入法处理器 (实现 gpui::InputHandler 以支持 Windows 原生 IME 中文输入)
pub struct AppInputHandler {
    pub model: Entity<AppState>,
}

impl InputHandler for AppInputHandler {
    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        cx: &mut App,
    ) -> Option<UTF16Selection> {
        let state = self.model.read(cx);
        if let Some(text) = state.active_text_input() {
            let total_chars = text.chars().count();
            let cursor = state.text_input_cursor.min(total_chars);
            let cursor_u16: usize = text.chars().take(cursor).map(|c| c.len_utf16()).sum();

            if let Some((sel_start, sel_end)) = state.text_input_selection_range() {
                let s_u16: usize = text.chars().take(sel_start).map(|c| c.len_utf16()).sum();
                let e_u16: usize = text.chars().take(sel_end).map(|c| c.len_utf16()).sum();
                Some(UTF16Selection {
                    range: s_u16..e_u16,
                    reversed: cursor < state.text_input_select_anchor.unwrap_or(0),
                })
            } else {
                Some(UTF16Selection {
                    range: cursor_u16..cursor_u16,
                    reversed: false,
                })
            }
        } else {
            None
        }
    }

    fn marked_text_range(&mut self, _window: &mut Window, cx: &mut App) -> Option<Range<usize>> {
        let state = self.model.read(cx);
        let text = state.active_text_input()?;
        let (start, end) = state.text_input_marked_range?;
        let s_u16 = char_offset_to_utf16_offset(text, start);
        let e_u16 = char_offset_to_utf16_offset(text, end);
        Some(s_u16..e_u16)
    }

    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut App,
    ) -> Option<String> {
        let state = self.model.read(cx);
        if let Some(text) = state.active_text_input() {
            let u16_chars: Vec<u16> = text.encode_utf16().collect();
            let start = range_utf16.start.min(u16_chars.len());
            let end = range_utf16.end.min(u16_chars.len());
            adjusted_range.replace(start..end);
            String::from_utf16(&u16_chars[start..end]).ok()
        } else {
            None
        }
    }

    fn replace_text_in_range(
        &mut self,
        replacement_range: Option<Range<usize>>,
        text: &str,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.model.update(cx, |state, cx| {
            let active_text = match state.active_text_input() {
                Some(t) => t.to_string(),
                None => return,
            };

            let char_range = if let Some(r_u16) = replacement_range {
                (
                    utf16_offset_to_char_offset(&active_text, r_u16.start),
                    utf16_offset_to_char_offset(&active_text, r_u16.end),
                )
            } else if let Some(m) = state.text_input_marked_range {
                m
            } else if let Some(sel) = state.text_input_selection_range() {
                sel
            } else {
                (state.text_input_cursor, state.text_input_cursor)
            };

            state.replace_range_in_active_input(char_range, text);
            state.text_input_marked_range = None;
            cx.notify();
        });
        window.invalidate_character_coordinates();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range: Option<Range<usize>>,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.model.update(cx, |state, cx| {
            let active_text = match state.active_text_input() {
                Some(t) => t.to_string(),
                None => return,
            };

            let char_range = if let Some(r_u16) = range_utf16 {
                (
                    utf16_offset_to_char_offset(&active_text, r_u16.start),
                    utf16_offset_to_char_offset(&active_text, r_u16.end),
                )
            } else if let Some(m) = state.text_input_marked_range {
                m
            } else if let Some(sel) = state.text_input_selection_range() {
                sel
            } else {
                (state.text_input_cursor, state.text_input_cursor)
            };

            let (new_start, new_end) = state.replace_range_in_active_input(char_range, new_text);

            if !new_text.is_empty() {
                state.text_input_marked_range = Some((new_start, new_end));
                if let Some(sel_u16) = new_selected_range {
                    let sel_char_start = utf16_offset_to_char_offset(new_text, sel_u16.start);
                    let sel_char_end = utf16_offset_to_char_offset(new_text, sel_u16.end);
                    state.text_input_cursor = new_start + sel_char_end;
                    if sel_char_start != sel_char_end {
                        state.text_input_select_anchor = Some(new_start + sel_char_start);
                    } else {
                        state.text_input_select_anchor = None;
                    }
                } else {
                    state.text_input_cursor = new_end;
                    state.text_input_select_anchor = None;
                }
            } else {
                state.text_input_marked_range = None;
                state.text_input_cursor = new_start;
                state.text_input_select_anchor = None;
            }

            cx.notify();
        });
        window.invalidate_character_coordinates();
    }

    fn unmark_text(&mut self, window: &mut Window, cx: &mut App) {
        self.model.update(cx, |state, cx| {
            state.text_input_marked_range = None;
            cx.notify();
        });
        window.invalidate_character_coordinates();
    }

    fn bounds_for_range(
        &mut self,
        _range_utf16: Range<usize>,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Option<Bounds<Pixels>> {
        get_caret_bounds()
    }

    fn character_index_for_point(
        &mut self,
        _point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Option<usize> {
        None
    }

    fn accepts_text_input(&mut self, _window: &mut Window, cx: &mut App) -> bool {
        let state = self.model.read(cx);
        state.has_active_text_input()
    }
}

/// 全局 IME 桥接 Element，挂载到应用根视图中并在 paint 阶段向当前窗口注册 handle_input
pub struct ImeInputBridge {
    model: Entity<AppState>,
    focus_handle: FocusHandle,
}

impl ImeInputBridge {
    pub fn new(model: Entity<AppState>, focus_handle: FocusHandle) -> Self {
        Self { model, focus_handle }
    }
}

impl IntoElement for ImeInputBridge {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for ImeInputBridge {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some("ime_input_bridge".into())
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        Some(std::panic::Location::caller())
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = px(0.0).into();
        style.size.height = px(0.0).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let handler = AppInputHandler {
            model: self.model.clone(),
        };
        window.handle_input(&self.focus_handle, handler, cx);
    }
}
