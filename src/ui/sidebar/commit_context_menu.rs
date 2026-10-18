use gpui::*;
use crate::ui::components::Toast;
use crate::ui::state::AppState;

/// 渲染 Git 提交记录右键上下文菜单 (Commit Context Menu)
/// 提供查看修改差异、撤回提交 (Revert)、软/混合重置 (Reset)、检出 (Checkout) 以及复制哈希与说明等常用 Git 功能
pub fn render_commit_context_menu(
    state: &AppState,
    commit_hash: &str,
    screen_pos: (f32, f32),
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let model_close = model.clone();
    let model_c2 = model.clone();

    // 查找当前右键点击的提交记录信息
    let commit_item = state
        .git_commits
        .iter()
        .find(|c| c.hash == commit_hash || c.short_hash == commit_hash)
        .cloned();

    let (short_hash, subject, author, relative_time, refs) = match &commit_item {
        Some(c) => (
            c.short_hash.clone(),
            c.subject.clone(),
            c.author.clone(),
            c.relative_time.clone(),
            c.refs.clone(),
        ),
        None => (
            commit_hash[..7.min(commit_hash.len())].to_string(),
            commit_hash.to_string(),
            String::new(),
            String::new(),
            Vec::new(),
        ),
    };

    let menu_w = 230.0;
    let menu_h = 330.0;
    let pos_x = (screen_pos.0).clamp(10.0, 2400.0 - menu_w);
    let pos_y = (screen_pos.1 + 4.0).clamp(10.0, 1400.0 - menu_h);

    let hash_clone = commit_hash.to_string();

    div()
        .id("commit_context_menu_overlay")
        .absolute()
        .inset_0()
        .bg(hsla(0.0, 0.0, 0.0, 0.15))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            model_close.update(cx, |st, cx| {
                st.git_commit_context_menu = None;
                cx.notify();
            });
        })
        .on_mouse_down(MouseButton::Right, move |_, _, cx| {
            model_c2.update(cx, |st, cx| {
                st.git_commit_context_menu = None;
                cx.notify();
            });
        })
        .child(
            div()
                .id("commit_context_menu_card")
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

                // 1. 顶部提交详情预览卡片 (Header)
                .child(
                    div()
                        .px_2()
                        .py_1p5()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .border_b_1()
                        .border_color(pal.card_border)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .gap_1p5()
                                .child(
                                    div()
                                        .px_1p5()
                                        .py_0p5()
                                        .rounded_xs()
                                        .bg(pal.card_surface)
                                        .border_1()
                                        .border_color(pal.panel_border)
                                        .text_size(px(9.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(pal.primary_accent)
                                        .child(short_hash.clone()),
                                )
                                .children(refs.first().map(|r| {
                                    div()
                                        .px_1p5()
                                        .py_0p5()
                                        .rounded_xs()
                                        .bg(rgba(0x2ea04326))
                                        .border_1()
                                        .border_color(rgba(0x2ea04344))
                                        .text_size(px(8.5))
                                        .text_color(rgb(0x3fb950))
                                        .child(format!("🌿 {}", r))
                                })),
                        )
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(10.5))
                                .text_color(pal.text_primary)
                                .whitespace_nowrap()
                                .overflow_hidden()
                                .child(subject.clone()),
                        )
                        .children((!author.is_empty()).then(|| {
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .text_size(px(8.5))
                                .text_color(pal.text_secondary)
                                .child(format!("👤 {}", author))
                                .child(format!("🕒 {}", relative_time))
                        })),
                )

                // 2. 查看提交变更 / 展开文件列表
                .child({
                    let m = model.clone();
                    let h = hash_clone.clone();
                    render_menu_item("🔍", state.t("commit_menu.view_changes"), &pal, move |cx| {
                        m.update(cx, |st, cx| {
                            st.git_commit_context_menu = None;
                            if st.git_expanded_commit_hash.as_deref() != Some(&h) {
                                st.toggle_expand_commit(&h);
                            }
                            cx.notify();
                        });
                    })
                })

                .child(render_divider(&pal))

                // 3. 撤回提交 (git revert)
                .child({
                    let m = model.clone();
                    let h = hash_clone.clone();
                    render_menu_item("🔄", state.t("commit_menu.revert_commit"), &pal, move |cx| {
                        m.update(cx, |st, cx| {
                            st.revert_commit_by_hash(&h);
                            cx.notify();
                        });
                    })
                })

                // 4. 软重置至此处 (Soft Reset，保留暂存区)
                .child({
                    let m = model.clone();
                    let h = hash_clone.clone();
                    render_menu_item("⏮️", state.t("commit_menu.soft_reset"), &pal, move |cx| {
                        m.update(cx, |st, cx| {
                            st.reset_to_commit_by_hash(&h, "--soft");
                            cx.notify();
                        });
                    })
                })

                // 5. 混合重置至此处 (Mixed Reset，保留修改到工作区)
                .child({
                    let m = model.clone();
                    let h = hash_clone.clone();
                    render_menu_item("↩️", state.t("commit_menu.mixed_reset"), &pal, move |cx| {
                        m.update(cx, |st, cx| {
                            st.reset_to_commit_by_hash(&h, "--mixed");
                            cx.notify();
                        });
                    })
                })

                // 6. 检出此提交 (Checkout)
                .child({
                    let m = model.clone();
                    let h = hash_clone.clone();
                    render_menu_item("🔀", state.t("commit_menu.checkout"), &pal, move |cx| {
                        m.update(cx, |st, cx| {
                            st.checkout_commit_by_hash(&h);
                            cx.notify();
                        });
                    })
                })

                .child(render_divider(&pal))

                // 7. 复制完整哈希
                .child({
                    let m = model.clone();
                    let h = hash_clone.clone();
                    let msg = state.t("toast.copy_hash_success").to_string();
                    render_menu_item("📋", state.t("commit_menu.copy_hash"), &pal, move |cx| {
                        cx.write_to_clipboard(ClipboardItem::new_string(h.clone()));
                        let msg = msg.clone();
                        m.update(cx, |st, cx| {
                            st.git_commit_context_menu = None;
                            st.toast = Some(Toast::success(msg));
                            cx.notify();
                        });
                    })
                })

                // 8. 复制短哈希
                .child({
                    let m = model.clone();
                    let sh = short_hash.clone();
                    let msg = state.t("toast.copy_hash_success").to_string();
                    render_menu_item("📑", state.t("commit_menu.copy_short_hash"), &pal, move |cx| {
                        cx.write_to_clipboard(ClipboardItem::new_string(sh.clone()));
                        let msg = msg.clone();
                        m.update(cx, |st, cx| {
                            st.git_commit_context_menu = None;
                            st.toast = Some(Toast::success(msg));
                            cx.notify();
                        });
                    })
                })

                // 9. 复制提交说明
                .child({
                    let m = model.clone();
                    let s = subject.clone();
                    let msg = state.t("toast.copy_msg_success").to_string();
                    render_menu_item("💬", state.t("commit_menu.copy_message"), &pal, move |cx| {
                        cx.write_to_clipboard(ClipboardItem::new_string(s.clone()));
                        let msg = msg.clone();
                        m.update(cx, |st, cx| {
                            st.git_commit_context_menu = None;
                            st.toast = Some(Toast::success(msg));
                            cx.notify();
                        });
                    })
                }),
        )
}

/// 辅助渲染单项菜单按钮
fn render_menu_item(
    icon: &'static str,
    label: &str,
    pal: &crate::ui::theme::ThemePalette,
    on_click: impl Fn(&mut App) + 'static + Clone,
) -> impl IntoElement {
    let label = label.to_string();
    div()
        .h(px(26.0))
        .w_full()
        .px_2()
        .rounded_md()
        .cursor_pointer()
        .hover(|s| s.bg(pal.card_hover))
        .flex()
        .items_center()
        .justify_between()
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            on_click(cx);
        })
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().text_size(px(11.0)).child(icon))
                .child(
                    div()
                        .text_size(px(10.0))
                        .text_color(pal.text_primary)
                        .child(label),
                ),
        )
}

/// 辅助渲染分割线
fn render_divider(pal: &crate::ui::theme::ThemePalette) -> impl IntoElement {
    div()
        .h(px(1.0))
        .my_0p5()
        .mx_1()
        .bg(pal.card_border)
}
