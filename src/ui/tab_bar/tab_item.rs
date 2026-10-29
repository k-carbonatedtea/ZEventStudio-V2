use gpui::*;
use gpui_component::tooltip::Tooltip;
use crate::model::OpenDocumentTab;
use crate::ui::state::AppState;

/// 渲染单项文件标签胶囊 (Tab Item)
pub fn render_tab_item(
    tab: &OpenDocumentTab,
    is_active: bool,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let tab_id = tab.id.clone();
    let tab_id_close = tab.id.clone();
    let tab_id_right = tab.id.clone();
    let file_path = tab.file_path.clone();
    let model_click = model.clone();
    let model_close = model.clone();
    let model_right_click = model.clone();

    let icon_color = match tab.icon() {
        "💬" => hsla(215.0 / 360.0, 0.90, 0.65, 1.0),       // 消息对话：蓝色
        "⚙️" => hsla(270.0 / 360.0, 0.85, 0.65, 1.0),       // BYML/AAMP：紫色
        "{}" => hsla(45.0 / 360.0, 0.95, 0.55, 1.0),        // JSON：金黄色
        "⚡" => hsla(200.0 / 360.0, 0.95, 0.60, 1.0),       // BFEVFL：天蓝色
        "🎬" => hsla(330.0 / 360.0, 0.90, 0.65, 1.0),       // BFEVTM：洋红色
        "📜" => hsla(145.0 / 360.0, 0.85, 0.55, 1.0),       // YAML：翡翠绿
        "≡" => hsla(240.0 / 360.0, 0.05, 0.65, 1.0),        // TXT/LOG：浅灰色
        "📦" => hsla(150.0 / 360.0, 0.85, 0.55, 1.0),       // SARC：翡翠绿
        _ => hsla(240.0 / 360.0, 0.05, 0.75, 1.0),
    };

    let tooltip_path = file_path.clone();
    let is_menu_open = state.tab_context_menu.is_some() || state.unsaved_changes_modal.is_some();

    let tab_element = div()
        .id(ElementId::Name(format!("tab-item-{}", tab.id).into()));

    let tab_element = if !is_menu_open {
        tab_element.tooltip(move |window, cx| {
            let path_for_el = tooltip_path.clone();
            Tooltip::element(move |_window, _cx| {
                div()
                    .text_size(px(10.5))
                    .child(path_for_el.clone())
            })
            .build(window, cx)
        })
    } else {
        tab_element
    };

    tab_element
        .h(px(26.0))
        .px_2p5()
        .rounded_md()
        .flex()
        .items_center()
        .gap_2()
        .cursor_pointer()
        .bg(if is_active {
            pal.card_surface // 浅层蓝灰 Card / Hover
        } else {
            rgba(0x00000000)
        })
        .border_1()
        .border_color(if is_active {
            pal.primary_accent // 遗迹青蓝 Primary Accent
        } else {
            pal.panel_border // 蓝灰边框
        })
        .hover(|s| {
            if !is_active {
                s.bg(pal.card_surface)
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
        .on_mouse_down(MouseButton::Right, move |event, _, cx| {
            cx.stop_propagation();
            let id = tab_id_right.clone();
            let pos = (event.position.x.into(), event.position.y.into());
            model_right_click.update(cx, |st, cx| {
                st.tab_context_menu = Some((id, pos));
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
                    pal.text_primary // 亮白灰 Text Primary
                } else {
                    pal.text_secondary // 蓝灰白 Text Secondary
                })
                .child(tab.title.clone()),
        )
        // 被修改状态小圆点提示 (Dirty Indicator ●)
        .children(tab.is_modified.then(|| {
            div()
                .size(px(6.5))
                .rounded_full()
                .bg(pal.secondary_accent) // 琥珀金 Secondary Accent
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
                        st.request_close_document_tab(&id);
                        cx.notify();
                    });
                })
                .child("✕"),
        )
}
