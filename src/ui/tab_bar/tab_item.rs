use gpui::*;
use crate::model::OpenDocumentTab;
use crate::ui::state::AppState;

/// 渲染单项文件标签胶囊 (Tab Item)
pub fn render_tab_item(
    tab: &OpenDocumentTab,
    is_active: bool,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let tab_id = tab.id.clone();
    let tab_id_close = tab.id.clone();
    let model_click = model.clone();
    let model_close = model.clone();

    let icon_color = match tab.icon() {
        "💬" => hsla(215.0 / 360.0, 0.90, 0.65, 1.0),       // 消息对话：蓝色
        "⚙️" => hsla(270.0 / 360.0, 0.85, 0.65, 1.0),       // BYML/AAMP：紫色
        "{}" => hsla(45.0 / 360.0, 0.95, 0.55, 1.0),        // JSON：金黄色
        "⚡" => hsla(200.0 / 360.0, 0.95, 0.60, 1.0),       // BFEVFL：天蓝色
        "📜" => hsla(145.0 / 360.0, 0.85, 0.55, 1.0),       // YAML：翡翠绿
        "≡" => hsla(240.0 / 360.0, 0.05, 0.65, 1.0),        // TXT/LOG：浅灰色
        "📦" => hsla(150.0 / 360.0, 0.85, 0.55, 1.0),       // SARC：翡翠绿
        _ => hsla(240.0 / 360.0, 0.05, 0.75, 1.0),
    };

    div()
        .h(px(26.0))
        .px_2p5()
        .rounded_md()
        .flex()
        .items_center()
        .gap_2()
        .cursor_pointer()
        .bg(if is_active {
            hsla(240.0 / 360.0, 0.06, 0.20, 1.0)
        } else {
            hsla(240.0 / 360.0, 0.06, 0.10, 0.6)
        })
        .border_1()
        .border_color(if is_active {
            hsla(215.0 / 360.0, 0.60, 0.45, 0.8)
        } else {
            hsla(240.0 / 360.0, 0.06, 0.15, 0.5)
        })
        .hover(|s| {
            if !is_active {
                s.bg(hsla(240.0 / 360.0, 0.06, 0.15, 1.0))
            } else {
                s
            }
        })
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            let id = tab_id.clone();
            model_click.update(cx, |st, cx| {
                st.switch_document_tab(&id);
                cx.notify();
            });
        })
        // 文件类型图标
        .child(
            div()
                .font_weight(FontWeight::BOLD)
                .text_size(px(11.0))
                .text_color(icon_color)
                .child(tab.icon()),
        )
        // 文件标题
        .child(
            div()
                .text_size(px(11.5))
                .font_weight(if is_active {
                    FontWeight::BOLD
                } else {
                    FontWeight::NORMAL
                })
                .text_color(if is_active {
                    hsla(0.0, 0.0, 1.0, 1.0)
                } else {
                    hsla(240.0 / 360.0, 0.05, 0.70, 1.0)
                })
                .child(tab.title.clone()),
        )
        // 被修改状态小圆点提示 (Dirty Indicator ●)
        .children(tab.is_modified.then(|| {
            div()
                .size(px(6.5))
                .rounded_full()
                .bg(hsla(45.0 / 360.0, 0.95, 0.55, 1.0))
        }))
        // 关闭按钮 ✕
        .child(
            div()
                .px_1()
                .py_0p5()
                .rounded_xs()
                .text_size(px(9.5))
                .text_color(hsla(240.0 / 360.0, 0.05, 0.55, 1.0))
                .hover(|s| {
                    s.bg(hsla(0.0, 0.70, 0.40, 0.9))
                        .text_color(hsla(0.0, 0.0, 1.0, 1.0))
                })
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    let id = tab_id_close.clone();
                    model_close.update(cx, |st, cx| {
                        st.close_document_tab(&id);
                        cx.notify();
                    });
                })
                .child("✕"),
        )
}
