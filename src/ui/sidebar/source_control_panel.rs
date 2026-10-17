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
                        views.push(render_compact_clean_tree_banner(state, &pal).into_any_element());
                    } else {
                        // 2.4 虚拟滚动列表 (uniform_list) 渲染暂存与变更文件，支持上万个文件 60FPS 流畅滚动
                        let model_vlist = model.clone();
                        let pal_vlist = pal;
                        let staged_vlist = staged_items.clone();
                        let unstaged_vlist = unstaged_items.clone();

                        let changes_height = if total_virtual_rows <= 5 {
                            px((total_virtual_rows as f32 * 30.0 + 8.0).max(40.0))
                        } else {
                            px(210.0)
                        };

                        views.push(
                            div()
                                .h(changes_height)
                                .min_h(px(32.0))
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
                                            let p = pal_vlist;

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

                    // 2.5 提交历史与分支图谱 (Commit History & Git Graph)
                    views.push(render_commit_history_section(state, model, &pal).into_any_element());

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
#[allow(dead_code)]
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
    let model_diff = model.clone();

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
    } else if crate::model::file_tree::is_sarc_extension(&item.file_name) {
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
        // 点击在主工作区加载打开文件或对比（非SARC直接进入IDE对比模式，SARC封包定位并展开）
        .on_mouse_down(MouseButton::Left, {
            let rp = rel_path.clone();
            move |_, _, cx| {
                let rp = rp.clone();
                model_click.update(cx, |st, cx| {
                    st.open_working_file_diff_tab(&rp, is_staged);
                    cx.notify();
                });
            }
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
                                .flex_shrink_0()
                                .whitespace_nowrap()
                                .child(item.file_name.clone()),
                        )
                        .children((!item.dir_name.is_empty()).then(|| {
                            div()
                                .text_size(px(9.0))
                                .text_color(pal.text_secondary)
                                .flex_1()
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
                // 对比修改差异按钮 (Diff 标签页模式)
                .child({
                    let rp = rel_path.clone();
                    div()
                        .px_1()
                        .py_0p5()
                        .rounded_xs()
                        .cursor_pointer()
                        .text_size(px(10.0))
                        .text_color(pal.text_secondary)
                        .hover(|s| s.bg(pal.card_surface).text_color(pal.primary_accent))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            let p = rp.clone();
                            model_diff.update(cx, |st, cx| {
                                st.open_working_file_diff_tab(&p, is_staged);
                                cx.notify();
                            });
                        })
                        .child("⇄")
                })
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

/// 渲染紧凑的工作区干净横条
fn render_compact_clean_tree_banner(
    state: &AppState,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    div()
        .h(px(32.0))
        .px_3()
        .flex()
        .items_center()
        .gap_2()
        .bg(pal.card_surface)
        .border_b_1()
        .border_color(pal.panel_border)
        .flex_shrink_0()
        .child(
            div()
                .text_size(px(12.0))
                .text_color(pal.primary_accent)
                .child("✔"),
        )
        .child(
            div()
                .text_size(px(10.5))
                .text_color(pal.text_secondary)
                .child(state.t("source_control.clean_tree")),
        )
}

/// 渲染 Git 提交历史与分支图谱视图 (Git Graph & Commit History Section)
fn render_commit_history_section(
    state: &AppState,
    model: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let is_collapsed = state.git_history_collapsed;
    let commits = state.git_commits.clone();
    let total_count = commits.len();
    let model_toggle = model.clone();
    let model_refresh = model.clone();
    let model_search_toggle = model.clone();
    let is_search_focused = state.git_history_search_focused;
    let search_query = state.git_history_search_query.clone();

    div()
        .flex_1()
        .flex()
        .flex_col()
        .overflow_hidden()
        .border_t_1()
        .border_color(pal.panel_border)
        .bg(pal.panel_surface)

        // 1. 提交历史折叠分组标题栏 (Header)
        .child(
            div()
                .h(px(28.0))
                .px_2()
                .flex()
                .items_center()
                .justify_between()
                .bg(pal.card_surface)
                .border_b_1()
                .border_color(pal.panel_border)
                .flex_shrink_0()
                // 左侧折叠箭头 + 标题 + 数量徽章
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .cursor_pointer()
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_toggle.update(cx, |st, cx| {
                                st.git_history_collapsed = !st.git_history_collapsed;
                                cx.notify();
                            });
                        })
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
                                .text_color(pal.text_primary)
                                .child(state.t("source_control.history")),
                        )
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_full()
                                .bg(pal.card_hover)
                                .text_size(px(9.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(pal.primary_accent)
                                .child(total_count.to_string()),
                        ),
                )
                // 右侧操作按钮 (搜索切换、刷新)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        // 搜索按钮
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_sm()
                                .cursor_pointer()
                                .text_size(px(10.0))
                                .text_color(if is_search_focused || !search_query.is_empty() {
                                    pal.primary_accent
                                } else {
                                    pal.text_secondary
                                })
                                .hover(|s| s.bg(pal.card_hover).text_color(pal.primary_accent))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_search_toggle.update(cx, |st, cx| {
                                        st.git_history_search_focused = !st.git_history_search_focused;
                                        cx.notify();
                                    });
                                })
                                .child("🔍"),
                        )
                        // 刷新历史
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_sm()
                                .cursor_pointer()
                                .text_size(px(10.0))
                                .text_color(pal.text_secondary)
                                .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    model_refresh.update(cx, |st, cx| {
                                        st.refresh_git_history();
                                        cx.notify();
                                    });
                                })
                                .child("🔄"),
                        ),
                ),
        )

        // 2. 提交历史主体内容 (如果未折叠)
        .children((!is_collapsed).then(|| {
            let model_content = model.clone();
            let pal_c = *pal;
            let commits_list = commits.clone();
            let q = state.git_history_search_query.trim().to_lowercase();
            let filtered: Vec<_> = commits_list
                .iter()
                .filter(|c| {
                    if q.is_empty() {
                        true
                    } else {
                        c.subject.to_lowercase().contains(&q)
                            || c.author.to_lowercase().contains(&q)
                            || c.short_hash.to_lowercase().contains(&q)
                            || c.refs.iter().any(|r| r.to_lowercase().contains(&q))
                    }
                })
                .cloned()
                .collect();

            let model_search = model.clone();
            let model_search_out = model.clone();
            let model_clear_search = model.clone();
            let model_load_more = model.clone();
            let expanded_hash = state.git_expanded_commit_hash.clone();
            let expanded_files = state.git_expanded_commit_files.clone();
            let has_more = state.git_commits.len() >= state.git_history_limit;

            div()
                .flex_1()
                .flex()
                .flex_col()
                .overflow_hidden()

                // 2.1 搜索输入栏
                .child(
                    div()
                        .h(px(26.0))
                        .px_2()
                        .mx_1p5()
                        .my_1()
                        .rounded_sm()
                        .bg(pal.card_surface)
                        .border_1()
                        .border_color(if is_search_focused {
                            pal.primary_accent
                        } else {
                            pal.panel_border
                        })
                        .flex()
                        .items_center()
                        .justify_between()
                        .cursor_text()
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_search.update(cx, |st, cx| {
                                st.clear_active_text_inputs();
                                st.git_history_search_focused = true;
                                cx.notify();
                            });
                        })
                        .on_mouse_down_out(move |_, _, cx| {
                            model_search_out.update(cx, |st, cx| {
                                if st.git_history_search_focused {
                                    st.git_history_search_focused = false;
                                    cx.notify();
                                }
                            });
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1p5()
                                .overflow_hidden()
                                .child(
                                    div()
                                        .text_size(px(10.0))
                                        .text_color(pal.text_secondary)
                                        .child("🔍"),
                                )
                                .child(
                                    if search_query.is_empty() && !is_search_focused {
                                        div()
                                            .text_size(px(9.5))
                                            .text_color(pal.text_secondary)
                                            .child(state.t("source_control.search_commits_placeholder"))
                                            .into_any_element()
                                    } else {
                                        div()
                                            .flex()
                                            .items_center()
                                            .text_size(px(9.5))
                                            .text_color(pal.text_primary)
                                            .child(search_query.clone())
                                            .children(is_search_focused.then(|| {
                                                render_blinking_cursor(pal_c.primary_accent.into()).into_any_element()
                                            }))
                                            .into_any_element()
                                    },
                                ),
                        )
                        .children((!search_query.is_empty()).then(|| {
                            div()
                                .px_1()
                                .cursor_pointer()
                                .text_size(px(9.5))
                                .text_color(pal.text_secondary)
                                .hover(|s| s.text_color(pal.text_primary))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    cx.stop_propagation();
                                    model_clear_search.update(cx, |st, cx| {
                                        st.git_history_search_query.clear();
                                        cx.notify();
                                    });
                                })
                                .child("✕")
                        })),
                )

                // 2.2 提交列表渲染区
                .child(
                    div()
                        .id("git_history_scroll_list")
                        .flex_1()
                        .overflow_y_scroll()
                        .px_1()
                        .children(if filtered.is_empty() {
                            vec![
                                div()
                                    .p_4()
                                    .flex()
                                    .flex_col()
                                    .items_center()
                                    .justify_center()
                                    .gap_1()
                                    .text_color(pal.text_secondary)
                                    .child(div().text_size(px(16.0)).child("🌿"))
                                    .child(
                                        div()
                                            .text_size(px(10.0))
                                            .child(if !q.is_empty() {
                                                state.t("source_control.no_matching_commits")
                                            } else {
                                                state.t("source_control.no_commits")
                                            }),
                                    )
                                    .into_any_element(),
                            ]
                        } else {
                            let total_f = filtered.len();
                            let mut rows = Vec::new();

                            for (ix, item) in filtered.into_iter().enumerate() {
                                let is_exp = expanded_hash.as_deref() == Some(&item.hash);
                                rows.push(
                                    render_commit_row(
                                        item,
                                        ix,
                                        total_f,
                                        is_exp,
                                        &expanded_files,
                                        &model_content,
                                        &pal_c,
                                    )
                                    .into_any_element(),
                                );
                            }

                            // 2.3 加载更多按钮
                            if has_more {
                                rows.push(
                                    div()
                                        .w_full()
                                        .py_1p5()
                                        .my_1()
                                        .rounded_sm()
                                        .bg(pal_c.card_surface)
                                        .border_1()
                                        .border_color(pal_c.panel_border)
                                        .cursor_pointer()
                                        .hover(|s| s.bg(pal_c.card_hover).border_color(pal_c.primary_accent))
                                        .text_align(TextAlign::Center)
                                        .text_size(px(10.0))
                                        .text_color(pal_c.primary_accent)
                                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                            model_load_more.update(cx, |st, cx| {
                                                st.load_more_git_history();
                                                cx.notify();
                                            });
                                        })
                                        .child(state.t("source_control.load_more"))
                                        .into_any_element(),
                                );
                            }

                            rows
                        }),
                )
        }))
}

/// 渲染单个提交历史节点与分支图谱轨道
fn render_commit_row(
    item: crate::model::git::GitCommitItem,
    ix: usize,
    total_count: usize,
    is_expanded: bool,
    expanded_files: &[crate::model::git::GitCommitFileChange],
    model: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let hash = item.hash.clone();
    let hash_click = hash.clone();
    let model_click = model.clone();

    // 分支轨道配色规范
    let dot_color = match ix % 4 {
        0 => pal.primary_accent,     // Relic Cyan
        1 => rgb(0x4EC9B0),          // Emerald
        2 => rgb(0xC586C0),          // Purple
        _ => rgb(0xDCDCAA),          // Amber
    };
    let line_color = rgba(0xffffff22);

    div()
        .w_full()
        .my_0p5()
        .rounded_sm()
        .bg(if is_expanded { pal.card_surface } else { rgba(0x00000000) })
        .hover(|s| s.bg(pal.card_hover))
        .border_1()
        .border_color(if is_expanded { pal.panel_border } else { rgba(0x00000000) })
        .flex()
        .flex_col()
        .child(
            div()
                .w_full()
                .p_1p5()
                .cursor_pointer()
                .flex()
                .items_start()
                .gap_1p5()
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    let h = hash_click.clone();
                    model_click.update(cx, |st, cx| {
                        st.toggle_expand_commit(&h);
                        cx.notify();
                    });
                })

                // 1. 左侧矢量分支图谱轨道 (Vector Git Graph Lane)
                .child(
                    div()
                        .w(px(14.0))
                        .min_h(px(32.0))
                        .flex()
                        .flex_col()
                        .items_center()
                        .flex_shrink_0()
                        // 顶部连线 (如果不是首个提交)
                        .child(
                            div()
                                .w(px(2.0))
                                .h(px(6.0))
                                .bg(if ix > 0 { line_color } else { rgba(0x00000000) }),
                        )
                        // 提交圆点
                        .child(
                            div()
                                .size(px(7.0))
                                .rounded_full()
                                .bg(dot_color)
                                .shadow_sm(),
                        )
                        // 底部连线 (如果不是最后一个提交，或当前已展开显示变更文件)
                        .child(
                            div()
                                .w(px(2.0))
                                .flex_1()
                                .min_h(px(14.0))
                                .bg(if ix < total_count - 1 || is_expanded {
                                    line_color
                                } else {
                                    rgba(0x00000000)
                                }),
                        ),
                )

                // 2. 中间提交详情 (Commit Info)
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .gap_0p5()
                        .overflow_hidden()
                        // 第一行：短哈希 + 分支标签 + 提交标题
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1p5()
                                .overflow_hidden()
                                // 短哈希药丸
                                .child(
                                    div()
                                        .px_1()
                                        .py_0p5()
                                        .rounded_xs()
                                        .bg(pal.card_surface)
                                        .border_1()
                                        .border_color(pal.panel_border)
                                        .text_size(px(8.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(dot_color)
                                        .flex_shrink_0()
                                        .child(item.short_hash.clone()),
                                )
                                // 分支引用标签 (如 HEAD -> main)
                                .children(item.refs.first().map(|r| {
                                    div()
                                        .px_1p5()
                                        .py_0p5()
                                        .rounded_xs()
                                        .bg(rgba(0x2ea04326))
                                        .border_1()
                                        .border_color(rgba(0x2ea04344))
                                        .text_size(px(8.5))
                                        .text_color(rgb(0x3fb950))
                                        .font_weight(FontWeight::MEDIUM)
                                        .flex_shrink_0()
                                        .child(format!("🌿 {}", r))
                                }))
                                // 提交标题
                                .child(
                                    div()
                                        .flex_1()
                                        .text_size(px(10.5))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(pal.text_primary)
                                        .whitespace_nowrap()
                                        .overflow_hidden()
                                        .child(item.subject.clone()),
                                ),
                        )
                        // 第二行：作者 + 相对时间
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .text_size(px(9.0))
                                .text_color(pal.text_secondary)
                                .child(format!("👤 {}", item.author))
                                .child(format!("🕒 {}", item.relative_time)),
                        ),
                )

                // 3. 右侧展开折叠箭头
                .child(
                    div()
                        .text_size(px(9.0))
                        .text_color(pal.text_secondary)
                        .flex_shrink_0()
                        .child(if is_expanded { "▼" } else { "▶" }),
                ),
        )

        // 4. 展开的修改文件列表 (Expanded File Changes)
        .children(is_expanded.then(|| {
            let commit_hash = hash.clone();
            div()
                .w_full()
                .pl_4()
                .pr_1p5()
                .pb_2()
                .flex()
                .flex_col()
                .gap_0p5()
                .children(if expanded_files.is_empty() {
                    vec![
                        div()
                            .py_1()
                            .text_size(px(9.5))
                            .text_color(pal.text_secondary)
                            .child("正在加载变更文件列表...")
                            .into_any_element(),
                    ]
                } else {
                    expanded_files
                        .iter()
                        .map(|f| {
                            let f_path = f.path.clone();
                            let f_hash = commit_hash.clone();
                            let m_diff = model.clone();
                            let file_name = std::path::Path::new(&f.path)
                                .file_name()
                                .and_then(|s| s.to_str())
                                .unwrap_or(&f.path)
                                .to_string();

                            let (status_label, status_color) = match f.status {
                                GitFileStatusKind::Modified => ("M", pal.secondary_accent),
                                GitFileStatusKind::Added => ("A", rgb(0x4EC9B0)),
                                GitFileStatusKind::Deleted => ("D", rgb(0xF14C4C)),
                                GitFileStatusKind::Renamed => ("R", rgb(0xCE9178)),
                                GitFileStatusKind::Untracked => ("U", rgb(0x73C991)),
                                GitFileStatusKind::Conflicted => ("!", rgb(0xE51400)),
                            };

                            let icon = if file_name.ends_with(".bfevfl") {
                                "⚡"
                            } else if file_name.ends_with(".msbt") {
                                "💬"
                            } else if crate::model::file_tree::is_sarc_extension(&file_name) {
                                "🗃️"
                            } else {
                                "📄"
                            };

                            div()
                                .h(px(24.0))
                                .px_1p5()
                                .rounded_xs()
                                .bg(rgba(0x00000022))
                                .hover(|s| s.bg(pal.card_hover))
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .justify_between()
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    let fp = f_path.clone();
                                    let h = f_hash.clone();
                                    m_diff.update(cx, |st, cx| {
                                        st.open_commit_file_diff_tab(&h, &fp);
                                        cx.notify();
                                    });
                                })
                                // 文件图标与名称
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_1p5()
                                        .overflow_hidden()
                                        .child(div().text_size(px(10.0)).child(icon))
                                        .child(
                                            div()
                                                .text_size(px(10.0))
                                                .text_color(pal.text_primary)
                                                .whitespace_nowrap()
                                                .overflow_hidden()
                                                .child(file_name),
                                        ),
                                )
                                // 右侧状态与差异对比图标
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_1()
                                        .flex_shrink_0()
                                        .child(
                                            div()
                                                .px_1()
                                                .rounded_xs()
                                                .text_size(px(9.0))
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(status_color)
                                                .child(status_label),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(10.0))
                                                .text_color(pal.text_secondary)
                                                .child("⇄"),
                                        ),
                                )
                                .into_any_element()
                        })
                        .collect()
                })
        }))
}
