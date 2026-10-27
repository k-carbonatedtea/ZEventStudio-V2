use std::rc::Rc;
use gpui::*;
use crate::ui::state::AppState;
use super::render::render_interactive_input_text;

/// 状态变动回调函数类型别名
pub type StateCallback = Rc<dyn Fn(&mut AppState)>;
/// 状态文本更新回调函数类型别名
pub type StateChangeCallback = Rc<dyn Fn(&mut AppState, String)>;

/// 现代化通用交互式文本输入小组件 (TextInput)
///
/// 具备：
/// - 统一响应式受控模式 (Controlled Component)
/// - 自动处理容器点击对焦、焦点排他与光标移至末尾
/// - 强制点击外部区域失焦 (`on_mouse_down_out`)
/// - 字符级鼠标点击精确定标、划选、双击选词与选区严格隔离
/// - 独立可选的一键清空按钮 (✕)
/// - 前缀 (prefix) 与后缀 (suffix) 自由挂载能力 (如搜索图标、状态指示器)
/// - 单行与多行形态自适应支持
#[derive(IntoElement)]
pub struct TextInput {
    id: ElementId,
    value: String,
    is_focused: bool,
    placeholder: Option<SharedString>,
    cleanable: bool,
    bg: Option<Rgba>,
    height: Option<DefiniteLength>,
    min_h: Option<DefiniteLength>,
    multiline: bool,
    prefix: Option<AnyElement>,
    suffix: Option<AnyElement>,
    model: Entity<AppState>,
    on_focus: Option<StateCallback>,
    on_blur: Option<StateCallback>,
    on_clear: Option<StateCallback>,
    on_change: Option<StateChangeCallback>,
}

#[allow(dead_code)]
impl TextInput {
    /// 创建一个全新的通用输入框组件
    pub fn new(
        id: impl Into<ElementId>,
        value: impl Into<String>,
        is_focused: bool,
        model: Entity<AppState>,
    ) -> Self {
        Self {
            id: id.into(),
            value: value.into(),
            is_focused,
            placeholder: None,
            cleanable: true,
            bg: None,
            height: None,
            min_h: None,
            multiline: false,
            prefix: None,
            suffix: None,
            model,
            on_focus: None,
            on_blur: None,
            on_clear: None,
            on_change: None,
        }
    }

    /// 自定义输入框背景色 (默认取 pal.panel_surface)
    pub fn bg(mut self, bg: impl Into<Rgba>) -> Self {
        self.bg = Some(bg.into());
        self
    }

    /// 设置输入框占位符提示文本
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// 设置是否显示一键清空按钮 (默认为 true)
    pub fn cleanable(mut self, cleanable: bool) -> Self {
        self.cleanable = cleanable;
        self
    }

    /// 设置输入框固定高度 (单行默认为 32px)
    pub fn h(mut self, height: impl Into<DefiniteLength>) -> Self {
        self.height = Some(height.into());
        self
    }

    /// 设置多行或展开时的最小高度
    pub fn min_h(mut self, min_h: impl Into<DefiniteLength>) -> Self {
        self.min_h = Some(min_h.into());
        self
    }

    /// 设置是否为多行编辑形态
    pub fn multiline(mut self, multiline: bool) -> Self {
        self.multiline = multiline;
        self
    }

    /// 设置左侧前缀元素 (如搜索放大镜图标)
    pub fn prefix(mut self, el: impl IntoElement) -> Self {
        self.prefix = Some(el.into_any_element());
        self
    }

    /// 设置右侧后缀元素 (如快捷模式切换按钮组)
    pub fn suffix(mut self, el: impl IntoElement) -> Self {
        self.suffix = Some(el.into_any_element());
        self
    }

    /// 注册获得焦点时的回调
    pub fn on_focus(mut self, cb: impl Fn(&mut AppState) + 'static) -> Self {
        self.on_focus = Some(Rc::new(cb));
        self
    }

    /// 注册失去焦点时的回调
    pub fn on_blur(mut self, cb: impl Fn(&mut AppState) + 'static) -> Self {
        self.on_blur = Some(Rc::new(cb));
        self
    }

    /// 注册点击一键清空时的专属回调
    pub fn on_clear(mut self, cb: impl Fn(&mut AppState) + 'static) -> Self {
        self.on_clear = Some(Rc::new(cb));
        self
    }

    /// 注册内容变更时的回调
    pub fn on_change(mut self, cb: impl Fn(&mut AppState, String) + 'static) -> Self {
        self.on_change = Some(Rc::new(cb));
        self
    }
}

impl RenderOnce for TextInput {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (pal, cursor, select_range, marked_range) = {
            let state = self.model.read(cx);
            let pal = state.theme();
            let cursor = if self.is_focused { state.text_input_cursor } else { 0 };
            let select_range = if self.is_focused { state.text_input_selection_range() } else { None };
            let marked_range = if self.is_focused { state.text_input_marked_range } else { None };
            (pal, cursor, select_range, marked_range)
        };

        let val_len = self.value.chars().count();
        let model_click = self.model.clone();
        let model_out = self.model.clone();
        let model_clear = self.model.clone();
        let on_focus = self.on_focus.clone();
        let on_blur = self.on_blur.clone();
        let on_clear = self.on_clear.clone();
        let on_change = self.on_change.clone();
        let is_focused = self.is_focused;

        let mut container = div()
            .id(self.id)
            .cursor_text()
            .rounded_sm()
            .border_1()
            .border_color(if is_focused { pal.primary_accent } else { pal.panel_border })
            .bg(self.bg.unwrap_or(pal.panel_surface))
            .flex()
            .items_center()
            .justify_between()
            .px_2()
            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                model_click.update(cx, |st, cx| {
                    st.clear_active_text_inputs();
                    if let Some(ref cb) = on_focus {
                        cb(st);
                    }
                    st.text_input_cursor = val_len;
                    st.text_input_select_anchor = None;
                    cx.notify();
                });
            })
            .on_mouse_down_out(move |_, _, cx| {
                model_out.update(cx, |st, cx| {
                    if is_focused {
                        if let Some(ref cb) = on_blur {
                            cb(st);
                        }
                        cx.notify();
                    }
                });
            });

        if let Some(h) = self.height {
            container = container.h(h);
        } else if !self.multiline {
            container = container.h(px(32.0));
        }

        if let Some(mh) = self.min_h {
            container = container.min_h(mh);
        }

        // 内部文本区域：前缀 + 内容/占位符 + 后缀
        let inner_content = div()
            .flex_1()
            .flex()
            .items_center()
            .gap_1p5()
            .overflow_hidden()
            .children(self.prefix)
            .child(
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .overflow_hidden()
                    .child(if self.value.is_empty() && !is_focused {
                        div()
                            .text_size(px(12.0))
                            .text_color(pal.text_secondary.opacity(0.6))
                            .child(self.placeholder.unwrap_or_default())
                            .into_any_element()
                    } else {
                        render_interactive_input_text(
                            &self.value,
                            is_focused,
                            cursor,
                            select_range,
                            marked_range,
                            pal.primary_accent.into(),
                            pal.text_primary,
                            &self.model,
                        )
                        .into_any_element()
                    }),
            )
            .children(self.suffix);

        container = container.child(inner_content);

        // 一键清空按钮 (阻止冒泡，自动同步光标与选区)
        if self.cleanable && !self.value.is_empty() {
            container = container.child(
                div()
                    .cursor_pointer()
                    .px_1()
                    .text_size(px(11.0))
                    .text_color(pal.text_secondary)
                    .hover(|s| s.text_color(pal.text_primary))
                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                        cx.stop_propagation();
                        model_clear.update(cx, |st, cx| {
                            if let Some(ref cb) = on_clear {
                                cb(st);
                            } else if let Some(ref cb) = on_change {
                                cb(st, String::new());
                            }
                            st.text_input_cursor = 0;
                            st.text_input_select_anchor = None;
                            cx.notify();
                        });
                    })
                    .child("✕"),
            );
        }

        container
    }
}
