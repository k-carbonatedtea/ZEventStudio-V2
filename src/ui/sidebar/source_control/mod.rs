pub mod commit_box;
pub mod commit_history;
pub mod commit_row;
pub mod empty_view;
pub mod file_list;
pub mod file_row;

use gpui::*;
use crate::ui::state::AppState;
use self::commit_box::render_commit_box;
use self::commit_history::render_commit_history_section;
use self::empty_view::render_uninitialized_repo_view;
use self::file_list::{
    render_compact_clean_tree_banner, render_staged_header,
    render_unsaved_warning_banner, render_unstaged_header,
};
use self::file_row::render_file_row;

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
        .filter(|t| t.is_modified && t.tab_kind != crate::model::DocumentTabKind::DiffViewer)
        .cloned()
        .collect();
    let unsaved_count = unsaved_tabs.len();

    // 采用 Arc 快速克隆，合并内存中未保存的已打开文档至未暂存列表
    let staged_items = state.git_staged.clone();
    let mut unstaged_items: Vec<crate::model::git::GitFileItem> = (*state.git_unstaged).clone();

    let project_dir = state.get_git_working_dir();
    for tab in &unsaved_tabs {
        let norm_tab_path = tab.file_path.replace('\\', "/");
        let rel_path = if let Some(ref root) = project_dir {
            let norm_root = root.replace('\\', "/").trim_end_matches('/').to_string();
            if let Some(stripped) = norm_tab_path.strip_prefix(&norm_root) {
                stripped.trim_start_matches('/').to_string()
            } else {
                norm_tab_path.clone()
            }
        } else {
            norm_tab_path.clone()
        };

        let in_staged = staged_items.iter().any(|item| item.rel_path.replace('\\', "/") == rel_path);
        let in_unstaged = unstaged_items.iter().any(|item| item.rel_path.replace('\\', "/") == rel_path);

        if !in_staged && !in_unstaged {
            let path_obj = std::path::Path::new(&rel_path);
            let file_name = path_obj
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(&tab.title)
                .to_string();
            let dir_name = path_obj
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or("")
                .replace('\\', "/");
            unstaged_items.push(crate::model::git::GitFileItem {
                rel_path,
                file_name,
                dir_name,
                full_path: tab.file_path.clone(),
                status: crate::model::git::GitFileStatusKind::Modified,
                is_staged: false,
            });
        }
    }

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
                        }))
                        .children((state.git_is_operating || !state.git_operating_files.is_empty()).then(|| {
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_sm()
                                .bg(pal.card_surface)
                                .border_1()
                                .border_color(pal.secondary_accent)
                                .flex()
                                .items_center()
                                .gap_1()
                                .child(
                                    div()
                                        .text_size(px(10.0))
                                        .child("⏳")
                                        .with_animation(
                                            "sc_global_spinner",
                                            Animation::new(std::time::Duration::from_millis(800)).repeat(),
                                            |el, delta| {
                                                let a = 0.3 + 0.7 * (delta * std::f32::consts::PI * 2.0).sin().abs();
                                                el.opacity(a)
                                            },
                                        ),
                                )
                                .child(
                                    div()
                                        .text_size(px(9.5))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(pal.secondary_accent)
                                        .child(state.t("source_control.processing")),
                                )
                        })),
                )
                // 右侧全局动作按钮
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        // 批量暂存所有更改 (+)
                        .children((is_repo && (unstaged_count > 0 || unsaved_count > 0)).then(|| {
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
                        .children((is_repo && (unstaged_count > 0 || unsaved_count > 0)).then(|| {
                            let m_discard_all = model_discard_all.clone();
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_md()
                                .cursor_pointer()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .hover(|s| s.bg(pal.card_hover).text_color(pal.secondary_accent))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    crate::ui::state_git_sync::git_discard_all_async(&m_discard_all, cx);
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
                                        st.refresh_git_history();
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
                        let unstaged_vlist = std::sync::Arc::new(unstaged_items);
                        let operating_vlist = state.git_operating_files.clone();
                        let is_global_operating = state.git_is_operating;

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
                                            let op = operating_vlist.clone();
                                            let glob_op = is_global_operating;

                                            range
                                                .map(|ix| {
                                                    if ix < staged_total {
                                                        if ix == 0 {
                                                            render_staged_header(staged_count, staged_collapsed, &m, &p)
                                                                .into_any_element()
                                                        } else {
                                                            if let Some(item) = st.get(ix - 1) {
                                                                let busy = glob_op || op.contains(&item.rel_path);
                                                                render_file_row(item, true, busy, &m, &p).into_any_element()
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
                                                                let busy = glob_op || op.contains(&item.rel_path);
                                                                render_file_row(item, false, busy, &m, &p).into_any_element()
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
