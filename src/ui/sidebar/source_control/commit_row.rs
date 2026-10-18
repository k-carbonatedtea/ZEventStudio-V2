use gpui::*;
use crate::model::git::GitFileStatusKind;
use crate::ui::state::AppState;
pub fn render_commit_row(
    item: crate::model::git::GitCommitItem,
    ix: usize,
    total_count: usize,
    is_expanded: bool,
    expanded_files: &[crate::model::git::GitCommitFileChange],
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let hash = item.hash.clone();
    let hash_click = hash.clone();
    let hash_right = hash.clone();
    let model_click = model.clone();
    let model_right = model.clone();

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
                .on_mouse_down(MouseButton::Right, move |event, _, cx| {
                    cx.stop_propagation();
                    let h = hash_right.clone();
                    let pos = (event.position.x.into(), event.position.y.into());
                    model_right.update(cx, |st, cx| {
                        st.git_commit_context_menu = Some((h, pos));
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

                            let is_sarc = crate::model::file_tree::is_sarc_extension(&file_name);
                            let sarc_key = format!("commit:{}:{}", f_hash, f_path);
                            let is_sarc_expanded = is_sarc && state.git_expanded_sarc_keys.contains(&sarc_key);

                            let icon = if file_name.ends_with(".bfevfl") {
                                "⚡"
                            } else if file_name.ends_with(".msbt") {
                                "💬"
                            } else if is_sarc {
                                "🗃️"
                            } else {
                                "📄"
                            };

                            let f_hash_chevron = f_hash.clone();
                            let f_path_chevron = f_path.clone();
                            let m_chevron = model.clone();

                            div()
                                .w_full()
                                .flex()
                                .flex_col()
                                .gap_0p5()
                                .child(
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
                                                if is_sarc && !st.git_expanded_sarc_keys.contains(&format!("commit:{}:{}", h, fp)) {
                                                    st.toggle_expand_commit_sarc(&h, &fp);
                                                }
                                                st.open_commit_file_diff_tab(&h, &fp);
                                                cx.notify();
                                            });
                                        })
                                        // 文件展开折叠箭头(若是压缩包) + 图标与名称 + 变动子文件数提示
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_1p5()
                                                .overflow_hidden()
                                                .children(is_sarc.then(|| {
                                                    div()
                                                        .px_1()
                                                        .py_0p5()
                                                        .cursor_pointer()
                                                        .text_size(px(8.5))
                                                        .text_color(if is_sarc_expanded {
                                                            pal.primary_accent
                                                        } else {
                                                            pal.text_secondary
                                                        })
                                                        .hover(|s| s.text_color(pal.text_primary))
                                                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                                            cx.stop_propagation();
                                                            let h = f_hash_chevron.clone();
                                                            let fp = f_path_chevron.clone();
                                                            m_chevron.update(cx, |st, cx| {
                                                                st.toggle_expand_commit_sarc(&h, &fp);
                                                                cx.notify();
                                                            });
                                                        })
                                                        .child(if is_sarc_expanded { "▼" } else { "▶" })
                                                }))
                                                .child(div().text_size(px(10.0)).child(icon))
                                                .child(
                                                    div()
                                                        .text_size(px(10.0))
                                                        .text_color(pal.text_primary)
                                                        .whitespace_nowrap()
                                                        .overflow_hidden()
                                                        .child(file_name),
                                                )
                                                .children(if is_sarc {
                                                    state.git_sarc_subfiles_cache.get(&sarc_key).map(|subs| {
                                                        div()
                                                            .px_1()
                                                            .rounded_xs()
                                                            .bg(pal.card_surface)
                                                            .border_1()
                                                            .border_color(pal.panel_border)
                                                            .text_size(px(8.0))
                                                            .text_color(pal.text_secondary)
                                                            .child(format!("{} 变更", subs.len()))
                                                    })
                                                } else {
                                                    None
                                                }),
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
                                                        .hover(|s| s.text_color(pal.primary_accent))
                                                        .child("⇄"),
                                                ),
                                        ),
                                )
                                // 如果已展开，渲染 SARC 封包内部子文件列表
                                .children(is_sarc_expanded.then(|| {
                                    crate::ui::sidebar::sarc_subfile_view::render_commit_sarc_subfiles(
                                        &commit_hash,
                                        &f.path,
                                        state,
                                        model,
                                        &pal,
                                    )
                                    .into_any_element()
                                }))
                                .into_any_element()
                        })
                        .collect()
                })
        }))
}