use gpui::*;
use crate::model::binary_diff::SarcSubFileChange;
use crate::model::git::GitFileStatusKind;
use crate::ui::state::AppState;
use crate::ui::theme::ThemePalette;

/// 渲染 SARC 封包展开后的单个内部子文件行
/// 支持高亮悬浮态、格式标签、状态指示器以及一键开启 IDE 差异对比
pub fn render_sarc_subfile_item(
    sub: &SarcSubFileChange,
    is_last: bool,
    pal: &ThemePalette,
    on_click: impl Fn(&mut App) + 'static + Clone,
) -> impl IntoElement {
    let file_name = sub.file_name.clone();
    let dir_name = sub.dir_name.clone();
    let format_name = sub.format_name.clone();
    let icon = sub.icon();

    let (status_label, status_color) = match sub.status {
        GitFileStatusKind::Modified => ("M", pal.secondary_accent),
        GitFileStatusKind::Added => ("A", rgb(0x4EC9B0)),
        GitFileStatusKind::Deleted => ("D", rgb(0xF14C4C)),
        GitFileStatusKind::Renamed => ("R", rgb(0xCE9178)),
        GitFileStatusKind::Untracked => ("U", rgb(0x73C991)),
        GitFileStatusKind::Conflicted => ("!", rgb(0xE51400)),
    };

    let tree_symbol = if is_last { "└─" } else { "├─" };

    div()
        .h(px(24.0))
        .w_full()
        .pl_5()
        .pr_1p5()
        .rounded_xs()
        .hover(|s| s.bg(pal.card_hover))
        .cursor_pointer()
        .flex()
        .items_center()
        .justify_between()
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            // 点击子文件行直接触发差异对比
            on_click(cx);
        })

        // 左侧：树形导引符 + 类型图标 + 子文件名 + 所处目录提示
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .overflow_hidden()
                // 树状层级连线符
                .child(
                    div()
                        .text_size(px(8.5))
                        .text_color(pal.text_secondary)
                        .flex_shrink_0()
                        .child(tree_symbol),
                )
                // 内部文件类型图标
                .child(
                    div()
                        .text_size(px(10.0))
                        .flex_shrink_0()
                        .child(icon),
                )
                // 子文件名
                .child(
                    div()
                        .text_size(px(9.5))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(pal.text_primary)
                        .whitespace_nowrap()
                        .overflow_hidden()
                        .child(file_name),
                )
                // 目录浅色提示 (如 Actor/ActorLink)
                .children((!dir_name.is_empty()).then(|| {
                    div()
                        .text_size(px(8.5))
                        .text_color(pal.text_secondary)
                        .whitespace_nowrap()
                        .overflow_hidden()
                        .child(format!("({})", dir_name))
                })),
        )

        // 右侧：格式药丸徽章 + 状态徽章 (M/A/D) + 对比图标
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .flex_shrink_0()
                // 格式标签 (如 AAMP, BYML)
                .child(
                    div()
                        .px_1()
                        .py_0p5()
                        .rounded_xs()
                        .bg(pal.card_surface)
                        .border_1()
                        .border_color(pal.panel_border)
                        .text_size(px(8.0))
                        .text_color(pal.text_secondary)
                        .child(format_name),
                )
                // 状态标志 (M, A, D)
                .child(
                    div()
                        .px_1()
                        .rounded_xs()
                        .text_size(px(9.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(status_color)
                        .child(status_label),
                )
                // 对比操作符号
                .child(
                    div()
                        .text_size(px(9.5))
                        .text_color(pal.text_secondary)
                        .hover(|s| s.text_color(pal.primary_accent))
                        .child("⇄"),
                ),
        )
}

/// 渲染提交历史中展开的 SARC 封包子文件列表
pub fn render_commit_sarc_subfiles(
    commit_hash: &str,
    sarc_rel_path: &str,
    state: &AppState,
    model: &Entity<AppState>,
    pal: &ThemePalette,
) -> impl IntoElement {
    let key = format!("commit:{}:{}", commit_hash, sarc_rel_path);
    let subfiles_opt = state.git_sarc_subfiles_cache.get(&key);

    let hash_clone = commit_hash.to_string();
    let sarc_path_clone = sarc_rel_path.to_string();
    let model_clone = model.clone();

    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_0p5()
        .children(match subfiles_opt {
            None => {
                vec![
                    div()
                        .h(px(22.0))
                        .pl_7()
                        .flex()
                        .items_center()
                        .text_size(px(9.0))
                        .text_color(pal.text_secondary)
                        .child(state.t("source_control.sarc_subfiles_loading"))
                        .into_any_element(),
                ]
            }
            Some(subs) if subs.is_empty() => {
                vec![
                    div()
                        .h(px(22.0))
                        .pl_7()
                        .flex()
                        .items_center()
                        .text_size(px(9.0))
                        .text_color(pal.text_secondary)
                        .child(state.t("source_control.sarc_no_subfiles_changed"))
                        .into_any_element(),
                ]
            }
            Some(subs) => {
                let total = subs.len();
                subs.iter()
                    .enumerate()
                    .map(|(ix, sub)| {
                        let is_last = ix == total - 1;
                        let h = hash_clone.clone();
                        let sp = sarc_path_clone.clone();
                        let sub_p = sub.sub_path.clone();
                        let m = model_clone.clone();

                        render_sarc_subfile_item(
                            sub,
                            is_last,
                            pal,
                            move |cx| {
                                let h = h.clone();
                                let sp = sp.clone();
                                let sub_p = sub_p.clone();
                                m.update(cx, |st, cx| {
                                    st.open_commit_sarc_subfile_diff(&h, &sp, &sub_p);
                                    cx.notify();
                                });
                            },
                        )
                        .into_any_element()
                    })
                    .collect()
            }
        })
}

/// 渲染工作区/暂存区展开的 SARC 封包子文件列表
#[allow(dead_code)]
pub fn render_working_sarc_subfiles(
    sarc_rel_path: &str,
    is_staged: bool,
    state: &AppState,
    model: &Entity<AppState>,
    pal: &ThemePalette,
) -> impl IntoElement {
    let key = if is_staged {
        format!("staged:{}", sarc_rel_path)
    } else {
        format!("working:{}", sarc_rel_path)
    };
    let subfiles_opt = state.git_sarc_subfiles_cache.get(&key);

    let sarc_path_clone = sarc_rel_path.to_string();
    let model_clone = model.clone();

    div()
        .w_full()
        .flex()
        .flex_col()
        .gap_0p5()
        .children(match subfiles_opt {
            None => {
                vec![
                    div()
                        .h(px(22.0))
                        .pl_7()
                        .flex()
                        .items_center()
                        .text_size(px(9.0))
                        .text_color(pal.text_secondary)
                        .child(state.t("source_control.sarc_subfiles_loading"))
                        .into_any_element(),
                ]
            }
            Some(subs) if subs.is_empty() => {
                vec![
                    div()
                        .h(px(22.0))
                        .pl_7()
                        .flex()
                        .items_center()
                        .text_size(px(9.0))
                        .text_color(pal.text_secondary)
                        .child(state.t("source_control.sarc_no_subfiles_changed"))
                        .into_any_element(),
                ]
            }
            Some(subs) => {
                let total = subs.len();
                subs.iter()
                    .enumerate()
                    .map(|(ix, sub)| {
                        let is_last = ix == total - 1;
                        let sp = sarc_path_clone.clone();
                        let sub_p = sub.sub_path.clone();
                        let m = model_clone.clone();

                        render_sarc_subfile_item(
                            sub,
                            is_last,
                            pal,
                            move |cx| {
                                let sp = sp.clone();
                                let sub_p = sub_p.clone();
                                m.update(cx, |st, cx| {
                                    st.open_working_sarc_subfile_diff(&sp, is_staged, &sub_p);
                                    cx.notify();
                                });
                            },
                        )
                        .into_any_element()
                    })
                    .collect()
            }
        })
}
