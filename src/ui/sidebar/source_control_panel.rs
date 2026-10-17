use gpui::*;
use crate::model::git::{GitFileItem, GitFileStatusKind};
use crate::ui::components::render_blinking_cursor;
use crate::ui::state::{AppState, GitCommitFocus};

/// 渲染 VS Code 风格的高性能 Git 源代码管理侧边栏面板 (Source Control Panel)
/// 采用虚拟列表 (uniform_list) 架构，即使有数万个变更文件也保持 60+ FPS 极致流畅！
pub fn render_source_control_panel(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let is_repo = state.git_is_repo;
    let branch_name = state.git_branch.clone();

    // 内存中未保存的已打开文档标签
    let unsaved_tabs: Vec<_> = state
        .open_tabs
        .iter()
        .filter(|t| t.is_modified)
        .cloned()
        .collect();
    let unsaved_count = unsaved_tabs.len();

    // 采用 Arc 快速克隆，零堆内存分配
    let staged_items = state.git_staged.clone();
    let unstaged_items = state.git_unstaged.clone();
    let staged_count = staged_items.len();
    let unstaged_count = unstaged_items.len();
    let total_git_changes = staged_count + unstaged_count;

    let staged_collapsed = state.git_staged_collapsed;
    let unstaged_collapsed = state.git_unstaged_collapsed;

    let model_refresh = model.clone();
    let model_stage_all = model.clone();
    let model_unstage_all = model.clone();
    let model_discard_all = model.clone();
    let model_save_all = model.clone();
    let model_init = model.clone();

    // 计算虚拟列表的条目数量与索引映射
    let staged_header_count = if staged_count > 0 { 1 } else { 0 };
    let staged_items_count = if staged_count > 0 && !staged_collapsed { staged_count } else { 0 };
    let staged_total = staged_header_count + staged_items_count;

    let unstaged_header_count = if unstaged_count > 0 { 1 } else { 0 };
    let unstaged_items_count = if unstaged_count > 0 && !unstaged_collapsed { unstaged_count } else { 0 };
    let unstaged_total = unstaged_header_count + unstaged_items_count;

    let total_virtual_rows = staged_total + unstaged_total;

    div()
        .flex()
        .flex_col()
        .h_full()
        .w_full()
        .bg(pal.panel_surface)
        .overflow_hidden()

        // 1. 顶部标题栏与全局工具栏 (Title & Branch & Actions)
        .child(
            div()
                .h(px(36.0))
                .px_3()
                .flex()
                .items_center()
                .justify_between()
                .border_b_1()
                .border_color(pal.panel_border)
                .bg(pal.panel_surface)
                .flex_shrink_0()
                // 左侧标题与分支药丸
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .overflow_hidden()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(11.0))
                                .text_color(pal.text_primary)
                                .child(state.t("source_control.title")),
                        )
                        .children(is_repo.then(|| {
                            let b_name = branch_name.as_deref().unwrap_or("main");
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_sm()
                                .bg(pal.card_surface)
                                .border_1()
                                .border_color(pal.panel_border)
                                .text_size(px(9.5))
                                .text_color(pal.primary_accent)
                                .font_weight(FontWeight::MEDIUM)
                                .flex()
                                .items_center()
                                .gap_1()
                                .child("🌿")
                                .child(b_name.to_string())
                        })),
                )
                // 右侧全局动作按钮
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        // 批量暂存所有更改 (+)
                        .children((is_repo && unstaged_count > 0).then(|| {
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_md()
                                .cursor_pointer()
                                .text_size(px(12.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(pal.text_secondary)
                                .hover(|s| s.bg(pal.card_hover).text_color(pal.primary_accent))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_stage_all.update(cx, |st, cx| {
                                        st.git_stage_all();
                                        cx.notify();
                                    });
                                })
                                .child("+")
                        }))
                        // 批量取消暂存所有更改 (-)
                        .children((is_repo && staged_count > 0).then(|| {
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_md()
                                .cursor_pointer()
                                .text_size(px(12.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(pal.text_secondary)
                                .hover(|s| s.bg(pal.card_hover).text_color(pal.secondary_accent))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_unstage_all.update(cx, |st, cx| {
                                        st.git_unstage_all();
                                        cx.notify();
                                    });
                                })
                                .child("-")
                        }))
                        // 批量放弃所有未暂存更改 (↩)
                        .children((is_repo && unstaged_count > 0).then(|| {
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_md()
                                .cursor_pointer()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .hover(|s| s.bg(pal.card_hover).text_color(pal.secondary_accent))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_discard_all.update(cx, |st, cx| {
                                        st.git_discard_all();
                                        cx.notify();
                                    });
                                })
                                .child("↩")
                        }))
                        // 刷新 Git 状态与同步 (🔄)
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_md()
                                .cursor_pointer()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_refresh.update(cx, |st, cx| {
                                        st.refresh_git_status();
                                        st.sync_active_tab_modified();
                                        cx.notify();
                                    });
                                })
                                .child("🔄"),
                        ),
                ),
        )

        // 2. 主体内容区
        .child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .overflow_hidden()
                .children(if !is_repo {
                    // ====== 未初始化 Git 仓库时的引导页面 ======
                    vec![render_uninitialized_repo_view(state, &model_init, &pal).into_any_element()]
                } else {
                    let mut views = Vec::new();

                    // 2.1 提交输入与操作区 (Commit Box) - 固定在列表顶部
                    views.push(render_commit_box(state, model, &pal).into_any_element());

                    // 2.2 内存中未保存标签页警告横幅 (Unsaved In-Memory Tabs)
                    if unsaved_count > 0 {
                        views.push(
                            render_unsaved_warning_banner(state, unsaved_count, &model_save_all, &pal)
                                .into_any_element(),
                        );
                    }

                    // 2.3 工作区非常干净 (没有任何变更)
                    if total_git_changes == 0 && unsaved_count == 0 {
                        views.push(render_clean_working_tree_view(state, &pal).into_any_element());
                    } else {
                        // 2.4 虚拟滚动列表 (uniform_list) 渲染暂存与变更文件，支持上万个文件 60FPS 流畅滚动
                        let model_vlist = model.clone();
                        let pal_vlist = pal.clone();
                        let staged_vlist = staged_items.clone();
                        let unstaged_vlist = unstaged_items.clone();

                        views.push(
                            div()
                                .flex_1()
                                .overflow_hidden()
                                .p_1()
                                .child(
                                    uniform_list(
                                        "source_control_virtual_list",
                                        total_virtual_rows,
                                        move |range, _window, _cx| {
                                            let st = staged_vlist.clone();
                                            let ust = unstaged_vlist.clone();
                                            let m = model_vlist.clone();
                                            let p = pal_vlist.clone();

                                            range
                                                .map(|ix| {
                                                    if ix < staged_total {
                                                        if ix == 0 {
                                                            render_staged_header(staged_count, staged_collapsed, &m, &p)
                                                                .into_any_element()
                                                        } else {
                                                            if let Some(item) = st.get(ix - 1) {
                                                                render_file_row(item, true, &m, &p).into_any_element()
                                                            } else {
                                                                div().h(px(30.0)).into_any_element()
                                                            }
                                                        }
                                                    } else {
                                                        let rem_ix = ix - staged_total;
                                                        if rem_ix == 0 {
                                                            render_unstaged_header(unstaged_count, unstaged_collapsed, &m, &p)
                                                                .into_any_element()
                                                        } else {
                                                            if let Some(item) = ust.get(rem_ix - 1) {
                                                                render_file_row(item, false, &m, &p).into_any_element()
                                                            } else {
                                                                div().h(px(30.0)).into_any_element()
                                                            }
                                                        }
                                                    }
                                                })
                                                .collect()
                                        },
                                    )
                                    .track_scroll(&state.source_control_scroll_handle)
                                    .size_full(),
                                )
                                .into_any_element(),
                        );
                    }

                    views
                }),
        )
}

/// 渲染未初始化 Git 仓库状态或未打开文件夹视图
fn render_uninitialized_repo_view(
    state: &AppState,
    model: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let model_init = model.clone();
    let model_open_folder = model.clone();
    let has_project_folder = state.get_git_working_dir().is_some();

    if has_project_folder {
        div()
            .p_5()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_3()
            .text_center()
            .child(
                div()
                    .text_size(px(32.0))
                    .child("🌿"),
            )
            .child(
                div()
                    .text_size(px(13.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(pal.text_primary)
                    .child(state.t("source_control.not_a_repo")),
            )
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(pal.text_secondary)
                    .line_height(relative(1.4))
                    .child(state.t("source_control.not_a_repo_hint")),
            )
            .child(
                div()
                    .mt_2()
                    .px_4()
                    .py_1p5()
                    .rounded_md()
                    .bg(pal.primary_accent)
                    .text_color(if pal.is_dark { rgb(0x0F1117) } else { rgb(0xFFFFFF) })
                    .text_size(px(11.5))
                    .font_weight(FontWeight::BOLD)
                    .cursor_pointer()
                    .hover(|s| s.opacity(0.9))
                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                        model_init.update(cx, |st, cx| {
                            st.git_init_repository();
                            cx.notify();
                        });
                    })
                    .child(state.t("source_control.init_repo")),
            )
    } else {
        div()
            .p_5()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_3()
            .text_center()
            .child(
                div()
                    .text_size(px(32.0))
                    .child("📁"),
            )
            .child(
                div()
                    .text_size(px(13.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(pal.text_primary)
                    .child(state.t("source_control.no_folder_open")),
            )
            .child(
                div()
                    .text_size(px(11.0))
                    .text_color(pal.text_secondary)
                    .line_height(relative(1.4))
                    .child(state.t("source_control.no_folder_open_hint")),
            )
            .child(
                div()
                    .mt_2()
                    .px_4()
                    .py_1p5()
                    .rounded_md()
                    .bg(pal.primary_accent)
                    .text_color(if pal.is_dark { rgb(0x0F1117) } else { rgb(0xFFFFFF) })
                    .text_size(px(11.5))
                    .font_weight(FontWeight::BOLD)
                    .cursor_pointer()
                    .hover(|s| s.opacity(0.9))
                    .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                        crate::ui::views::welcome::quick_tab::pick_and_load_mod_folder(&model_open_folder, cx);
                    })
                    .child(state.t("explorer.open_mod")),
            )
    }
}

/// 渲染提交信息输入与提交操作区 (Commit Title, Description & Action Button)
fn render_commit_box(
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

/// 渲染内存中未保存的文档修改警告条目
fn render_unsaved_warning_banner(
    state: &AppState,
    count: usize,
    model_save_all: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let m = model_save_all.clone();

    div()
        .px_3()
        .py_2()
        .bg(if pal.is_dark { rgb(0x282315) } else { rgb(0xFFF9E6) })
        .border_b_1()
        .border_color(pal.secondary_accent)
        .flex()
        .items_center()
        .justify_between()
        .flex_shrink_0()
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .child(
                    div()
                        .text_size(px(12.0))
                        .text_color(pal.secondary_accent)
                        .child("⚠️"),
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(pal.secondary_accent)
                        .child(format!("{} ({} 项)", state.t("source_control.unsaved_tabs_hint"), count)),
                ),
        )
        .child(
            div()
                .px_2()
                .py_0p5()
                .rounded_xs()
                .bg(pal.secondary_accent)
                .text_color(if pal.is_dark { rgb(0x0F1117) } else { rgb(0xFFFFFF) })
                .text_size(px(9.5))
                .font_weight(FontWeight::BOLD)
                .cursor_pointer()
                .hover(|s| s.opacity(0.85))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    m.update(cx, |st, cx| {
                        st.save_all_modified_documents();
                        st.refresh_git_status();
                        cx.notify();
                    });
                })
                .child(state.t("source_control.save_all_tabs")),
        )
}

/// 渲染工作区完全干净视图
fn render_clean_working_tree_view(
    state: &AppState,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    div()
        .p_5()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_2()
        .text_center()
        .child(
            div()
                .text_size(px(26.0))
                .text_color(pal.primary_accent)
                .child("✔"),
        )
        .child(
            div()
                .text_size(px(12.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(pal.text_primary)
                .child(state.t("source_control.clean_tree")),
        )
        .child(
            div()
                .text_size(px(10.5))
                .text_color(pal.text_secondary)
                .child(state.t("source_control.all_saved_hint")),
        )
}

/// 渲染暂存分区的表头行 (高度恒定 30px)
fn render_staged_header(
    count: usize,
    is_collapsed: bool,
    model: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let model_toggle = model.clone();
    let model_unstage_all = model.clone();

    div()
        .h(px(30.0))
        .w_full()
        .px_2()
        .bg(pal.card_surface)
        .border_b_1()
        .border_color(pal.panel_border)
        .flex()
        .items_center()
        .justify_between()
        .cursor_pointer()
        .hover(|s| s.bg(pal.card_hover))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            model_toggle.update(cx, |st, cx| {
                st.git_staged_collapsed = !st.git_staged_collapsed;
                cx.notify();
            });
        })
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .child(
                    div()
                        .text_size(px(9.0))
                        .text_color(pal.text_secondary)
                        .child(if is_collapsed { "▶" } else { "▼" }),
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.text_secondary)
                        .child("暂存的更改"),
                )
                .child(
                    div()
                        .min_w(px(14.0))
                        .h(px(14.0))
                        .px_1()
                        .rounded_full()
                        .bg(pal.primary_accent)
                        .text_color(if pal.is_dark { rgb(0x0F1117) } else { rgb(0xFFFFFF) })
                        .text_size(px(8.5))
                        .font_weight(FontWeight::BOLD)
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(count.to_string()),
                ),
        )
        // 取消暂存全部按钮 (-)
        .child(
            div()
                .px_1p5()
                .py_0p5()
                .rounded_xs()
                .cursor_pointer()
                .text_size(px(12.0))
                .font_weight(FontWeight::BOLD)
                .text_color(pal.text_secondary)
                .hover(|s| s.bg(pal.card_hover).text_color(pal.secondary_accent))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    model_unstage_all.update(cx, |st, cx| {
                        st.git_unstage_all();
                        cx.notify();
                    });
                })
                .child("-"),
        )
}

/// 渲染未暂存更改分区的表头行 (高度恒定 30px)
fn render_unstaged_header(
    count: usize,
    is_collapsed: bool,
    model: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let model_toggle = model.clone();
    let model_stage_all = model.clone();
    let model_discard_all = model.clone();

    div()
        .h(px(30.0))
        .w_full()
        .px_2()
        .bg(pal.card_surface)
        .border_b_1()
        .border_color(pal.panel_border)
        .flex()
        .items_center()
        .justify_between()
        .cursor_pointer()
        .hover(|s| s.bg(pal.card_hover))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            model_toggle.update(cx, |st, cx| {
                st.git_unstaged_collapsed = !st.git_unstaged_collapsed;
                cx.notify();
            });
        })
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .child(
                    div()
                        .text_size(px(9.0))
                        .text_color(pal.text_secondary)
                        .child(if is_collapsed { "▶" } else { "▼" }),
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.text_secondary)
                        .child("更改"),
                )
                .child(
                    div()
                        .min_w(px(14.0))
                        .h(px(14.0))
                        .px_1()
                        .rounded_full()
                        .bg(pal.secondary_accent)
                        .text_color(if pal.is_dark { rgb(0x0F1117) } else { rgb(0xFFFFFF) })
                        .text_size(px(8.5))
                        .font_weight(FontWeight::BOLD)
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(count.to_string()),
                ),
        )
        // 操作按钮组: 全部放弃 (↩) 与 全部暂存 (+)
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .child(
                    div()
                        .px_1p5()
                        .py_0p5()
                        .rounded_xs()
                        .cursor_pointer()
                        .text_size(px(11.0))
                        .text_color(pal.text_secondary)
                        .hover(|s| s.bg(pal.card_hover).text_color(pal.secondary_accent))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            model_discard_all.update(cx, |st, cx| {
                                st.git_discard_all();
                                cx.notify();
                            });
                        })
                        .child("↩"),
                )
                .child(
                    div()
                        .px_1p5()
                        .py_0p5()
                        .rounded_xs()
                        .cursor_pointer()
                        .text_size(px(12.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.text_secondary)
                        .hover(|s| s.bg(pal.card_hover).text_color(pal.primary_accent))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            model_stage_all.update(cx, |st, cx| {
                                st.git_stage_all();
                                cx.notify();
                            });
                        })
                        .child("+"),
                ),
        )
}

/// 渲染单行文件变更项 (高度恒定 30px，满足 uniform_list 约束)
fn render_file_row(
    item: &GitFileItem,
    is_staged: bool,
    model: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let model_click = model.clone();
    let model_action = model.clone();
    let model_discard = model.clone();

    let full_path = item.full_path.clone();
    let rel_path = item.rel_path.clone();
    let is_untracked = item.status == GitFileStatusKind::Untracked;

    // 状态字符与对应颜色
    let (status_label, status_color) = match item.status {
        GitFileStatusKind::Modified => ("M", pal.secondary_accent),
        GitFileStatusKind::Added => ("A", rgb(0x4EC9B0)),
        GitFileStatusKind::Deleted => ("D", rgb(0xF14C4C)),
        GitFileStatusKind::Renamed => ("R", rgb(0xCE9178)),
        GitFileStatusKind::Untracked => ("U", rgb(0x73C991)),
        GitFileStatusKind::Conflicted => ("!", rgb(0xE51400)),
    };

    // 文件类型图标
    let icon = if item.file_name.ends_with(".bfevfl") {
        "⚡"
    } else if item.file_name.ends_with(".msbt") {
        "💬"
    } else if item.file_name.ends_with(".sbeventpack") || item.file_name.ends_with(".pack") {
        "🗃️"
    } else {
        "📄"
    };

    div()
        .h(px(30.0))
        .w_full()
        .px_2()
        .rounded_sm()
        .cursor_pointer()
        .hover(|s| s.bg(pal.card_hover))
        .flex()
        .items_center()
        .justify_between()
        // 点击在主工作区加载打开文件
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            let fp = full_path.clone();
            model_click.update(cx, |st, cx| {
                st.load_file_by_path(&fp);
                cx.notify();
            });
        })
        // 左侧图标与文件名/目录
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .overflow_hidden()
                .child(
                    div()
                        .text_size(px(11.0))
                        .flex_shrink_0()
                        .child(icon),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .overflow_hidden()
                        .child(
                            div()
                                .text_size(px(10.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(status_color)
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .child(item.file_name.clone()),
                        )
                        .children((!item.dir_name.is_empty()).then(|| {
                            div()
                                .text_size(px(9.0))
                                .text_color(pal.text_secondary)
                                .overflow_hidden()
                                .whitespace_nowrap()
                                .child(item.dir_name.clone())
                        })),
                ),
        )
        // 右侧操作按钮组与状态字母徽章
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .flex_shrink_0()
                // 放弃更改按钮 (仅未暂存可放弃)
                .children((!is_staged).then(|| {
                    let rp = rel_path.clone();
                    div()
                        .px_1()
                        .py_0p5()
                        .rounded_xs()
                        .cursor_pointer()
                        .text_size(px(10.0))
                        .text_color(pal.text_secondary)
                        .hover(|s| s.bg(pal.card_surface).text_color(pal.secondary_accent))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            let p = rp.clone();
                            model_discard.update(cx, |st, cx| {
                                st.git_discard_file(&p, is_untracked);
                                cx.notify();
                            });
                        })
                        .child("↩")
                }))
                // 暂存 (+) 或 取消暂存 (-) 按钮
                .child({
                    let rp = rel_path.clone();
                    div()
                        .px_1()
                        .py_0p5()
                        .rounded_xs()
                        .cursor_pointer()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.text_secondary)
                        .hover(move |s| {
                            if is_staged {
                                s.bg(pal.card_surface).text_color(pal.secondary_accent)
                            } else {
                                s.bg(pal.card_surface).text_color(pal.primary_accent)
                            }
                        })
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            let p = rp.clone();
                            model_action.update(cx, |st, cx| {
                                if is_staged {
                                    st.git_unstage_file(&p);
                                } else {
                                    st.git_stage_file(&p);
                                }
                                cx.notify();
                            });
                        })
                        .child(if is_staged { "-" } else { "+" })
                })
                // 状态标志字母 (M, A, D, U, R, !)
                .child(
                    div()
                        .w(px(16.0))
                        .h(px(16.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(status_color)
                        .child(status_label),
                ),
        )
}
