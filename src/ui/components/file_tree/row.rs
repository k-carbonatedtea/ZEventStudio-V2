use gpui::*;
use crate::ui::state::AppState;
use super::types::FlatFileEntry;

pub(crate) fn render_flat_file_row(
    item: &FlatFileEntry,
    model: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let is_dir = item.is_dir;
    let is_sarc = item.is_sarc;
    let is_container = is_dir || is_sarc;
    let is_selected = item.is_selected;
    let is_modified = item.is_modified;
    let path = item.path.clone();
    let name_clone = item.name.clone();
    let model_click = model.clone();
    let model_right = model.clone();

    let guide_line_color = if pal.is_dark {
        hsla(215.0 / 360.0, 0.20, 0.40, 0.25)
    } else {
        hsla(215.0 / 360.0, 0.20, 0.60, 0.25)
    };

    let item_path_id = path.clone();
    let r_path = path.clone();
    let r_name = name_clone.clone();

    div()
        .id(ElementId::Name(item_path_id.into()))
        .h(px(26.0))
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .pl_1()
        .pr_2()
        .rounded_md()
        .cursor_pointer()
        .bg(if is_selected {
            pal.card_surface
        } else {
            rgba(0x00000000)
        })
        .border_l_2()
        .border_color(if is_selected {
            pal.primary_accent
        } else {
            rgba(0x00000000)
        })
        .hover(move |s| {
            if is_selected {
                s.bg(pal.card_hover)
            } else {
                s.bg(pal.card_surface)
            }
        })
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            let path_clone = path.clone();
            model_click.update(cx, |st, cx| {
                st.clear_active_text_inputs();
                st.file_tree_sort_menu_open = false;
                if is_container {
                    st.toggle_tree_path(&path_clone);
                } else {
                    st.load_file_by_path(&path_clone);
                }
                cx.notify();
            });
        })
        .on_mouse_down(MouseButton::Right, move |e, _window, cx| {
            cx.stop_propagation();
            let p = r_path.clone();
            let n = r_name.clone();
            let pos = (e.position.x.into(), e.position.y.into());
            model_right.update(cx, |st, cx| {
                st.open_file_tree_context_menu(p, n, is_dir, is_sarc, pos);
                cx.notify();
            });
        })
        .child(
            div()
                .flex()
                .items_center()
                .flex_1()
                .overflow_hidden()
                .whitespace_nowrap()
                // 1. 层级缩进竖向引导提示线 (Indent Guides)
                .children((0..item.level).map(|_| {
                    div()
                        .w(px(14.0))
                        .h(px(26.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .flex_shrink_0()
                        .child(
                            div()
                                .w(px(1.0))
                                .h(px(26.0))
                                .bg(guide_line_color),
                        )
                }))
                // 2. 展开/折叠三角指示符 (Expand / Collapse Chevron)
                .child(
                    div()
                        .w(px(14.0))
                        .h(px(26.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .flex_shrink_0()
                        .text_size(px(10.0))
                        .text_color(pal.text_secondary)
                        .child(if is_container {
                            if item.is_expanded { "⌄" } else { "›" }
                        } else {
                            " "
                        }),
                )
                // 3. 节点类型图标 (Node Type Icon)
                .child(
                    div()
                        .mr_1()
                        .text_size(px(12.0))
                        .flex_shrink_0()
                        .child(get_node_icon_by_name(&item.name, item.is_dir, item.is_sarc)),
                )
                // 4. 节点名称与修改标记 (Node Name & Dirty Indicator)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .child(
                            div()
                                .text_size(px(11.5))
                                .font_weight(if is_selected || is_modified {
                                    FontWeight::BOLD
                                } else {
                                    FontWeight::NORMAL
                                })
                                .text_color(if is_modified {
                                    pal.secondary_accent // 琥珀金：已被修改
                                } else if is_selected {
                                    pal.primary_accent // 遗迹青蓝：当前选中
                                } else if is_container {
                                    pal.text_primary // 亮白灰：目录
                                } else {
                                    pal.text_secondary // 蓝灰白：普通文件
                                })
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .child(item.name.clone()),
                        )
                        .children(is_modified.then(|| {
                            div()
                                .text_size(px(8.0))
                                .text_color(pal.secondary_accent)
                                .child("●")
                        })),
                ),
        )
}

/// 根据文件/封包属性获取对应展示图标
fn get_node_icon_by_name(name: &str, is_dir: bool, is_sarc: bool) -> &'static str {
    if is_sarc {
        "🗃️"
    } else if is_dir {
        "📁"
    } else {
        let lower = name.to_lowercase();
        if lower.ends_with(".bfevfl") || lower.ends_with(".evfl") {
            "⚡"
        } else if lower.ends_with(".bfevtm") || lower.ends_with(".evtm") {
            "🎬"
        } else if lower.ends_with(".msbt") || lower.ends_with(".smsbt") {
            "💬"
        } else if lower.ends_with(".byml")
            || lower.ends_with(".bgyml")
            || lower.ends_with(".sbyml")
            || lower.ends_with(".aamp")
            || lower.ends_with(".bxml")
            || lower.ends_with(".baev")
            || lower.ends_with(".baglbl")
            || lower.ends_with(".bglcl")
            || lower.ends_with(".bactparam")
            || lower.ends_with(".baiprog")
            || lower.ends_with(".sbaiprog")
        {
            "⚙️"
        } else if lower.ends_with(".json") {
            "{}"
        } else if lower.ends_with(".yaml") || lower.ends_with(".yml") {
            "📜"
        } else if lower.ends_with(".txt") || lower.ends_with(".log") {
            "≡"
        } else {
            "📄"
        }
    }
}