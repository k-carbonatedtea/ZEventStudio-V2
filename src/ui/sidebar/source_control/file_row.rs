use gpui::*;
use crate::model::git::{GitFileItem, GitFileStatusKind};
use crate::ui::state::AppState;

/// 渲染单行文件变更项 (高度恒定 30px，满足 uniform_list 约束)
pub fn render_file_row(
    item: &GitFileItem,
    is_staged: bool,
    is_busy: bool,
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
        .hover(move |s| if !is_busy { s.bg(pal.card_hover) } else { s })
        // 点击在主工作区加载打开文件或对比（若存在已打开文档标签页优先直接切换，否则进入对比模式）
        .on_mouse_down(MouseButton::Left, {
            let rp = rel_path.clone();
            move |_, _, cx| {
                let rp = rp.clone();
                model_click.update(cx, |st, cx| {
                    let matched_tab_id = st.open_tabs.iter().find(|t| {
                        t.tab_kind != crate::model::DocumentTabKind::DiffViewer && {
                            let norm = t.file_path.replace('\\', "/");
                            norm.ends_with(&rp) || norm == rp
                        }
                    }).map(|t| t.id.clone());

                    if let Some(tid) = matched_tab_id {
                        st.switch_document_tab(&tid);
                    } else {
                        st.open_working_file_diff_tab(&rp, is_staged);
                    }
                    cx.notify();
                });
            }
        })
        .flex()
        .items_center()
        .justify_between()
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
                                .text_color(if is_busy { pal.text_secondary } else { status_color })
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
                // 放弃更改按钮 (仅未暂存可放弃；正在处理中则展示呼吸动画)
                .children((!is_staged).then(|| {
                    if is_busy {
                        div()
                            .px_1()
                            .py_0p5()
                            .rounded_xs()
                            .text_size(px(10.0))
                            .text_color(pal.secondary_accent)
                            .child("⏳")
                            .with_animation(
                                SharedString::from(format!("git_file_busy_{}", rel_path)),
                                Animation::new(std::time::Duration::from_millis(800)).repeat(),
                                |el, delta| {
                                    let a = 0.3 + 0.7 * (delta * std::f32::consts::PI * 2.0).sin().abs();
                                    el.opacity(a)
                                },
                            )
                            .into_any_element()
                    } else {
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
                                crate::ui::state_git_sync::git_discard_file_async(
                                    &model_discard,
                                    rp.clone(),
                                    is_untracked,
                                    cx,
                                );
                            })
                            .child("↩")
                            .into_any_element()
                    }
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
