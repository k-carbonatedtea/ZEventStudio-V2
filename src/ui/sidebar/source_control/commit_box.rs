use gpui::*;
use crate::ui::components::render_blinking_cursor;
use crate::ui::state::{AppState, GitCommitFocus};
/// 渲染提交信息输入与提交操作区 (Commit Title, Description & Action Button)
pub fn render_commit_box(
    state: &AppState,
    model: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let is_title_focused = state.git_commit_focus == GitCommitFocus::Title;
    let is_desc_focused = state.git_commit_focus == GitCommitFocus::Description;
    let show_desc = state.git_show_desc_field;

    let has_staged = !state.git_staged.is_empty();
    let has_unstaged = !state.git_unstaged.is_empty();
    let has_any_changes = has_staged || has_unstaged;
    let has_title = !state.git_commit_title.trim().is_empty();

    let model_title_click = model.clone();
    let model_title_out = model.clone();
    let model_desc_click = model.clone();
    let model_desc_out = model.clone();
    let model_toggle_desc = model.clone();
    let model_commit = model.clone();

    let commit_btn_label = if has_staged {
        state.t("source_control.commit_btn")
    } else {
        state.t("source_control.stage_and_commit_btn")
    };

    div()
        .p_3()
        .flex()
        .flex_col()
        .gap_2()
        .border_b_1()
        .border_color(pal.panel_border)
        .bg(pal.card_surface)
        .flex_shrink_0()

        // 1. 提交标题输入框 (Commit Title)
        .child(
            div()
                .px_2()
                .py_1p5()
                .rounded_md()
                .bg(pal.panel_surface)
                .border_1()
                .border_color(if is_title_focused {
                    pal.primary_accent
                } else {
                    pal.panel_border
                })
                .cursor_text()
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_title_click.update(cx, |st, cx| {
                        st.git_commit_focus = GitCommitFocus::Title;
                        cx.notify();
                    });
                })
                .on_mouse_down_out(move |_, _, cx| {
                    model_title_out.update(cx, |st, cx| {
                        if st.git_commit_focus == GitCommitFocus::Title {
                            st.git_commit_focus = GitCommitFocus::None;
                            cx.notify();
                        }
                    });
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .overflow_hidden()
                        .children(if state.git_commit_title.is_empty() && !is_title_focused {
                            vec![
                                div()
                                    .text_size(px(11.0))
                                    .text_color(pal.text_secondary)
                                    .child(state.t("source_control.commit_title_placeholder"))
                                    .into_any_element(),
                            ]
                        } else {
                            let mut elems = vec![
                                div()
                                    .text_size(px(11.0))
                                    .text_color(pal.text_primary)
                                    .child(state.git_commit_title.clone())
                                    .into_any_element(),
                            ];
                            if is_title_focused {
                                elems.push(render_blinking_cursor(pal.primary_accent.into()).into_any_element());
                            }
                            elems
                        }),
                ),
        )

        // 2. 展开/收起描述切换链接
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .cursor_pointer()
                        .text_size(px(10.0))
                        .text_color(pal.text_secondary)
                        .hover(|s| s.text_color(pal.primary_accent))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_toggle_desc.update(cx, |st, cx| {
                                st.git_show_desc_field = !st.git_show_desc_field;
                                if st.git_show_desc_field {
                                    st.git_commit_focus = GitCommitFocus::Description;
                                } else if st.git_commit_focus == GitCommitFocus::Description {
                                    st.git_commit_focus = GitCommitFocus::Title;
                                }
                                cx.notify();
                            });
                        })
                        .child(if show_desc {
                            state.t("source_control.hide_desc")
                        } else {
                            state.t("source_control.add_desc")
                        }),
                )
                .child(
                    div()
                        .text_size(px(9.5))
                        .text_color(pal.text_secondary)
                        .child("Ctrl+Enter 提交"),
                ),
        )

        // 3. 详细描述多行输入框 (Commit Description)
        .children(show_desc.then(|| {
            let desc_lines: Vec<String> = if state.git_commit_desc.is_empty() {
                vec![]
            } else {
                state.git_commit_desc.split('\n').map(|s| s.to_string()).collect()
            };

            div()
                .min_h(px(54.0))
                .px_2()
                .py_1p5()
                .rounded_md()
                .bg(pal.panel_surface)
                .border_1()
                .border_color(if is_desc_focused {
                    pal.primary_accent
                } else {
                    pal.panel_border
                })
                .cursor_text()
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_desc_click.update(cx, |st, cx| {
                        st.git_commit_focus = GitCommitFocus::Description;
                        cx.notify();
                    });
                })
                .on_mouse_down_out(move |_, _, cx| {
                    model_desc_out.update(cx, |st, cx| {
                        if st.git_commit_focus == GitCommitFocus::Description {
                            st.git_commit_focus = GitCommitFocus::None;
                            cx.notify();
                        }
                    });
                })
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_0p5()
                        .children(if desc_lines.is_empty() && !is_desc_focused {
                            vec![
                                div()
                                    .text_size(px(10.5))
                                    .text_color(pal.text_secondary)
                                    .child(state.t("source_control.commit_desc_placeholder"))
                                    .into_any_element(),
                            ]
                        } else {
                            let mut line_elems = Vec::new();
                            let last_idx = if desc_lines.is_empty() { 0 } else { desc_lines.len() - 1 };
                            if desc_lines.is_empty() {
                                line_elems.push(
                                    div()
                                        .flex()
                                        .items_center()
                                        .h(px(14.0))
                                        .child(render_blinking_cursor(pal.primary_accent.into()))
                                        .into_any_element(),
                                );
                            } else {
                                for (i, line) in desc_lines.iter().enumerate() {
                                    let is_last = i == last_idx;
                                    line_elems.push(
                                        div()
                                            .flex()
                                            .items_center()
                                            .min_h(px(14.0))
                                            .child(
                                                div()
                                                    .text_size(px(10.5))
                                                    .text_color(pal.text_primary)
                                                    .child(line.clone()),
                                            )
                                            .children((is_last && is_desc_focused).then(|| {
                                                render_blinking_cursor(pal.primary_accent.into()).into_any_element()
                                            }))
                                            .into_any_element(),
                                    );
                                }
                            }
                            line_elems
                        }),
                )
        }))

        // 4. 大号提交动作按钮 (Commit Button)
        .child(
            div()
                .w_full()
                .py_1p5()
                .rounded_md()
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(11.5))
                .font_weight(FontWeight::BOLD)
                .bg(if has_any_changes && has_title {
                    pal.primary_accent
                } else {
                    pal.card_hover
                })
                .text_color(if has_any_changes && has_title {
                    if pal.is_dark { rgb(0x0F1117) } else { rgb(0xFFFFFF) }
                } else {
                    pal.text_secondary
                })
                .cursor_pointer()
                .hover(move |s| {
                    if has_any_changes && has_title {
                        s.opacity(0.9)
                    } else {
                        s
                    }
                })
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    model_commit.update(cx, |st, cx| {
                        st.git_commit();
                        cx.notify();
                    });
                })
                .child(commit_btn_label),
        )
}
