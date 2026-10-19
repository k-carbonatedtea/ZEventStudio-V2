use gpui::*;
use crate::ui::state::AppState;
use crate::ui::state_types::FileTreeContextMenu;

/// 渲染资源管理器文件树右键上下文菜单
pub fn render_file_tree_context_menu(
    state: &AppState,
    menu: &FileTreeContextMenu,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let model_close = model.clone();
    let model_c2 = model.clone();

    let menu_w = 210.0;
    let menu_h = 240.0;
    let pos_x = (menu.position.0).clamp(10.0, 2400.0 - menu_w);
    let pos_y = (menu.position.1 + 4.0).clamp(10.0, 1400.0 - menu_h);

    let path_clone = menu.path.clone();
    let name_clone = menu.name.clone();
    let is_dir = menu.is_dir;
    let is_sarc = menu.is_sarc;
    let is_inside_sarc = menu.is_inside_sarc;

    div()
        .id("file_tree_context_menu_overlay")
        .absolute()
        .inset_0()
        .bg(hsla(0.0, 0.0, 0.0, 0.15))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            model_close.update(cx, |st, cx| {
                st.close_file_tree_context_menu();
                cx.notify();
            });
        })
        .on_mouse_down(MouseButton::Right, move |_, _, cx| {
            model_c2.update(cx, |st, cx| {
                st.close_file_tree_context_menu();
                cx.notify();
            });
        })
        .child(
            div()
                .id("file_tree_context_menu_card")
                .absolute()
                .left(px(pos_x))
                .top(px(pos_y))
                .w(px(menu_w))
                .rounded_xl()
                .bg(pal.panel_surface)
                .border_1()
                .border_color(pal.panel_border)
                .shadow_2xl()
                .p_1p5()
                .flex()
                .flex_col()
                .gap_0p5()
                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                .on_mouse_down(MouseButton::Right, |_, _, cx| {
                    cx.stop_propagation();
                })
                // 1. 顶部文件概要标题
                .child(
                    div()
                        .px_2()
                        .py_1p5()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .border_b_1()
                        .border_color(pal.card_border)
                        .child(
                            div()
                                .text_size(px(12.0))
                                .child(if is_dir {
                                    "📁"
                                } else if is_sarc {
                                    "🗃️"
                                } else if is_inside_sarc {
                                    "📦"
                                } else {
                                    "📄"
                                }),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_size(px(11.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(pal.text_primary)
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .child(name_clone.clone()),
                        ),
                )
                // 2. 菜单操作项列表
                // 替换文件 (针对 SARC 内部文件)
                .children((is_inside_sarc && !is_dir).then(|| {
                    let m = model.clone();
                    let p = path_clone.clone();
                    let n = name_clone.clone();
                    render_menu_item("🔄", state.t("context_menu.replace"), &pal, move |cx| {
                        crate::ui::sidebar::pick_and_replace_file(&m, p.clone(), n.clone(), cx);
                    })
                }))
                // 导出单个文件 (针对所有单文件，包含 SARC 内部文件与普通物理文件)
                .children((!is_dir).then(|| {
                    let m = model.clone();
                    let p = path_clone.clone();
                    let n = name_clone.clone();
                    render_menu_item("📤", state.t("context_menu.export_file"), &pal, move |cx| {
                        crate::ui::sidebar::pick_and_export_file(&m, p.clone(), n.clone(), cx);
                    })
                }))
                // 添加/导入文件 (针对容器：普通文件夹、SARC 封包或 SARC 内部虚拟目录)
                .children((is_dir || is_sarc).then(|| {
                    let m = model.clone();
                    let p = path_clone.clone();
                    render_menu_item("➕", state.t("context_menu.add_file"), &pal, move |cx| {
                        crate::ui::sidebar::pick_and_add_files(&m, p.clone(), is_dir, is_sarc, cx);
                    })
                }))
                // 导出/解包整个文件夹或 SARC 封包 (针对容器)
                .children((is_dir || is_sarc).then(|| {
                    let m = model.clone();
                    let p = path_clone.clone();
                    render_menu_item("📦", state.t("context_menu.export_container"), &pal, move |cx| {
                        crate::ui::sidebar::pick_and_export_container(&m, p.clone(), is_dir, is_sarc, cx);
                    })
                }))
                // 重命名
                .child({
                    let m = model.clone();
                    let p = path_clone.clone();
                    let n = name_clone.clone();
                    render_menu_item("✏️", state.t("context_menu.rename"), &pal, move |cx| {
                        m.update(cx, |st, cx| {
                            st.trigger_rename_item(&p, &n, is_dir, is_inside_sarc);
                            cx.notify();
                        });
                    })
                })
                // 删除
                .child({
                    let m = model.clone();
                    let p = path_clone.clone();
                    let n = name_clone.clone();
                    render_danger_menu_item("🗑️", state.t("context_menu.delete"), &pal, move |cx| {
                        m.update(cx, |st, cx| {
                            st.trigger_delete_item(&p, &n, is_dir, is_inside_sarc);
                            cx.notify();
                        });
                    })
                })
                // 在资源管理器中显示
                .child({
                    let m = model.clone();
                    let p = path_clone.clone();
                    render_menu_item("📂", state.t("context_menu.reveal_in_explorer"), &pal, move |cx| {
                        m.update(cx, |st, cx| {
                            st.close_file_tree_context_menu();
                            st.reveal_item_in_file_explorer(&p);
                            cx.notify();
                        });
                    })
                }),
        )
}

fn render_menu_item(
    icon: &'static str,
    label: &str,
    pal: &crate::ui::theme::ThemePalette,
    on_click: impl Fn(&mut App) + 'static,
) -> impl IntoElement {
    let lbl = label.to_string();
    div()
        .px_2()
        .py_1p5()
        .rounded_lg()
        .flex()
        .items_center()
        .gap_2()
        .cursor_pointer()
        .hover(|s| s.bg(pal.card_hover))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            on_click(cx);
        })
        .child(div().text_size(px(12.0)).child(icon))
        .child(
            div()
                .text_size(px(11.5))
                .text_color(pal.text_primary)
                .child(lbl),
        )
}

fn render_danger_menu_item(
    icon: &'static str,
    label: &str,
    _pal: &crate::ui::theme::ThemePalette,
    on_click: impl Fn(&mut App) + 'static,
) -> impl IntoElement {
    let lbl = label.to_string();
    div()
        .px_2()
        .py_1p5()
        .rounded_lg()
        .flex()
        .items_center()
        .gap_2()
        .cursor_pointer()
        .hover(|s| s.bg(rgba(0xe06c7526)))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            on_click(cx);
        })
        .child(div().text_size(px(12.0)).child(icon))
        .child(
            div()
                .text_size(px(11.5))
                .text_color(rgb(0xEF4444))
                .child(lbl),
        )
}
