use gpui::*;
use crate::ui::components::Toast;
use crate::ui::state::AppState;

/// 渲染顶部标签页右键上下文菜单 (Tab Context Menu)
pub fn render_tab_context_menu(
    state: &AppState,
    tab_id: &str,
    screen_pos: (f32, f32),
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let model_close = model.clone();
    let model_c2 = model.clone();

    let tab = state.open_tabs.iter().find(|t| t.id == tab_id).cloned();
    let (tab_title, tab_path, tab_icon, is_modified) = match &tab {
        Some(t) => (t.title.clone(), t.file_path.clone(), t.icon(), t.is_modified),
        None => ("Tab".to_string(), String::new(), "📄", false),
    };

    let menu_w = 210.0;
    let menu_h = 290.0;
    let pos_x = (screen_pos.0).clamp(10.0, 2400.0 - menu_w);
    let pos_y = (screen_pos.1 + 4.0).clamp(10.0, 1400.0 - menu_h);

    let t_id = tab_id.to_string();

    div()
        .id("tab_context_menu_overlay")
        .absolute()
        .inset_0()
        .bg(hsla(0.0, 0.0, 0.0, 0.20))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            model_close.update(cx, |st, cx| {
                st.tab_context_menu = None;
                cx.notify();
            });
        })
        .on_mouse_down(MouseButton::Right, move |_, _, cx| {
            model_c2.update(cx, |st, cx| {
                st.tab_context_menu = None;
                cx.notify();
            });
        })
        .child(
            div()
                .id("tab_context_menu_card")
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
                // 1. 顶部当前标签名提示
                .child(
                    div()
                        .px_2()
                        .py_1()
                        .flex()
                        .items_center()
                        .justify_between()
                        .border_b_1()
                        .border_color(pal.card_border)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1p5()
                                .overflow_hidden()
                                .child(
                                    div()
                                        .text_size(px(11.0))
                                        .child(tab_icon),
                                )
                                .child(
                                    div()
                                        .font_weight(FontWeight::BOLD)
                                        .text_size(px(11.5))
                                        .text_color(pal.text_primary)
                                        .overflow_hidden()
                                        .child(tab_title.clone()),
                                ),
                        )
                        .children(is_modified.then(|| {
                            div()
                                .size(px(6.0))
                                .rounded_full()
                                .bg(pal.secondary_accent)
                        })),
                )
                // 2. 关闭 (Close)
                .child(render_menu_item(
                    state.t("tab_menu.close"),
                    Some("Ctrl+W"),
                    None,
                    pal.text_primary,
                    {
                        let tid = t_id.clone();
                        let m = model.clone();
                        move |cx| {
                            let target = tid.clone();
                            m.update(cx, |st, cx| {
                                st.tab_context_menu = None;
                                st.request_close_document_tab(&target);
                                cx.notify();
                            });
                        }
                    },
                    state,
                ))
                // 3. 关闭其他 (Close Others)
                .child(render_menu_item(
                    state.t("tab_menu.close_others"),
                    None,
                    None,
                    pal.text_primary,
                    {
                        let tid = t_id.clone();
                        let m = model.clone();
                        move |cx| {
                            let target = tid.clone();
                            m.update(cx, |st, cx| {
                                st.tab_context_menu = None;
                                st.close_other_tabs(&target);
                                cx.notify();
                            });
                        }
                    },
                    state,
                ))
                // 4. 关闭右侧标签页 (Close to Right)
                .child(render_menu_item(
                    state.t("tab_menu.close_to_right"),
                    None,
                    None,
                    pal.text_primary,
                    {
                        let tid = t_id.clone();
                        let m = model.clone();
                        move |cx| {
                            let target = tid.clone();
                            m.update(cx, |st, cx| {
                                st.tab_context_menu = None;
                                st.close_tabs_to_right(&target);
                                cx.notify();
                            });
                        }
                    },
                    state,
                ))
                // 5. 关闭已保存 (Close Saved)
                .child(render_menu_item(
                    state.t("tab_menu.close_saved"),
                    None,
                    None,
                    pal.text_primary,
                    {
                        let m = model.clone();
                        move |cx| {
                            m.update(cx, |st, cx| {
                                st.tab_context_menu = None;
                                st.close_saved_tabs();
                                cx.notify();
                            });
                        }
                    },
                    state,
                ))
                // 6. 全部关闭 (Close All)
                .child(render_menu_item(
                    state.t("tab_menu.close_all"),
                    None,
                    None,
                    pal.text_primary,
                    {
                        let m = model.clone();
                        move |cx| {
                            m.update(cx, |st, cx| {
                                st.tab_context_menu = None;
                                st.request_close_all_tabs();
                                cx.notify();
                            });
                        }
                    },
                    state,
                ))
                // 分割线
                .child(
                    div()
                        .my_1()
                        .h(px(1.0))
                        .bg(pal.card_border),
                )
                // 7. 保存文件 (Save File)
                .child(render_menu_item(
                    state.t("tab_menu.save"),
                    Some("Ctrl+S"),
                    Some("💾"),
                    pal.primary_accent,
                    {
                        let tid = t_id.clone();
                        let m = model.clone();
                        move |cx| {
                            let target = tid.clone();
                            m.update(cx, |st, cx| {
                                st.tab_context_menu = None;
                                st.save_document_tab(&target);
                                cx.notify();
                            });
                        }
                    },
                    state,
                ))
                // 8. 复制完整路径 (Copy Path)
                .child(render_menu_item(
                    state.t("tab_menu.copy_path"),
                    None,
                    Some("📋"),
                    pal.text_secondary,
                    {
                        let path = tab_path.clone();
                        let m = model.clone();
                        move |cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(path.clone()));
                            m.update(cx, |st, cx| {
                                st.tab_context_menu = None;
                                st.toast = Some(Toast::success(st.t("tab_menu.path_copied")));
                                cx.notify();
                            });
                        }
                    },
                    state,
                ))
                // 9. 复制文件名 (Copy Name)
                .child(render_menu_item(
                    state.t("tab_menu.copy_name"),
                    None,
                    Some("📄"),
                    pal.text_secondary,
                    {
                        let title = tab_title.clone();
                        let m = model.clone();
                        move |cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(title.clone()));
                            m.update(cx, |st, cx| {
                                st.tab_context_menu = None;
                                st.toast = Some(Toast::success(st.t("tab_menu.name_copied")));
                                cx.notify();
                            });
                        }
                    },
                    state,
                )),
        )
        .into_any_element()
}

/// 渲染单行上下文菜单选项
fn render_menu_item(
    label: &str,
    shortcut: Option<&str>,
    icon: Option<&str>,
    text_color: Rgba,
    on_click: impl Fn(&mut App) + 'static,
    state: &AppState,
) -> impl IntoElement {
    let pal = state.theme();
    let label_str = label.to_string();
    let shortcut_str = shortcut.map(|s| s.to_string());
    let icon_str = icon.map(|s| s.to_string());

    div()
        .px_2p5()
        .py_1p5()
        .rounded_md()
        .cursor_pointer()
        .flex()
        .items_center()
        .justify_between()
        .hover(|s| s.bg(pal.card_surface))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            cx.stop_propagation();
            on_click(cx);
        })
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .children(icon_str.map(|ic| {
                    div()
                        .text_size(px(11.0))
                        .child(ic)
                }))
                .child(
                    div()
                        .text_size(px(12.0))
                        .text_color(text_color)
                        .child(label_str),
                ),
        )
        .children(shortcut_str.map(|sc| {
            div()
                .text_size(px(10.0))
                .text_color(pal.text_secondary)
                .child(sc)
        }))
}
