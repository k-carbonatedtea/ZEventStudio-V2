use gpui::*;
use crate::model::git::{GitDiffLineType, GitFileDiff};
use crate::ui::state::AppState;

/// Git 差异对比弹窗状态
#[derive(Clone, Debug)]
pub struct GitDiffModalState {
    pub title: String,            // 文件名，例如 "Armor_NPD_Head.sbactorpack"
    pub subtitle: String,         // 提交信息或工作区说明
    pub commit_hash: Option<String>,
    pub file_path: String,        // 相对路径
    pub diff: GitFileDiff,
    pub available_files: Vec<String>, // 该提交包含的所有变更文件，方便快速切换
}

/// 渲染 VS Code / GitHub 风格的代码差异对比模态框 (Diff Viewer Modal)
pub fn render_git_diff_modal(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let modal = match &state.active_diff_modal {
        Some(m) => m.clone(),
        None => return div().into_any_element(),
    };

    let model_close = model.clone();
    let model_file_switch = model.clone();
    let commit_hash_opt = modal.commit_hash.clone();
    let current_file_path = modal.file_path.clone();
    let total_diff_lines = modal.diff.lines.len();

    div()
        .id("git_diff_modal_overlay")
        .absolute()
        .inset_0()
        .occlude()
        .bg(rgba(0x000000aa))
        .flex()
        .items_center()
        .justify_center()
        .on_scroll(|_, _, cx| cx.stop_propagation())
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            cx.stop_propagation();
            model_close.update(cx, |st, cx| {
                st.close_diff_modal();
                cx.notify();
            });
        })
        .child(
            div()
                .id("git_diff_modal_card")
                .w(px(880.0))
                .h(px(600.0))
                .max_w(px(1100.0))
                .max_h(px(780.0))
                .bg(pal.panel_surface)
                .border_1()
                .border_color(pal.panel_border)
                .rounded_lg()
                .shadow_xl()
                .flex()
                .flex_col()
                .overflow_hidden()
                .on_scroll(|_, _, cx| cx.stop_propagation())
                // 阻止卡片内部点击事件冒泡到背景遮罩
                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })

                // 1. 顶部标题栏
                .child({
                    let model_btn_close = model.clone();
                    div()
                        .h(px(46.0))
                        .px_4()
                        .flex()
                        .items_center()
                        .justify_between()
                        .border_b_1()
                        .border_color(pal.panel_border)
                        .bg(pal.card_surface)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .overflow_hidden()
                                .child(
                                    div()
                                        .text_size(px(14.0))
                                        .child("📝"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .overflow_hidden()
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_2()
                                                .child(
                                                    div()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_size(px(12.5))
                                                        .text_color(pal.text_primary)
                                                        .child(modal.title.clone()),
                                                )
                                                .child(
                                                    div()
                                                        .text_size(px(10.0))
                                                        .text_color(pal.text_secondary)
                                                        .child(format!("({})", modal.file_path)),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(10.0))
                                                .text_color(pal.primary_accent)
                                                .child(modal.subtitle.clone()),
                                        ),
                                ),
                        )
                        // 右侧统计与关闭按钮
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                // +新增行数
                                .child(
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_sm()
                                        .bg(rgba(0x2ea04326))
                                        .border_1()
                                        .border_color(rgba(0x2ea04344))
                                        .text_size(px(10.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0x3fb950))
                                        .child(format!("+{}", modal.diff.additions)),
                                )
                                // -删除行数
                                .child(
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_sm()
                                        .bg(rgba(0xf8514926))
                                        .border_1()
                                        .border_color(rgba(0xf8514944))
                                        .text_size(px(10.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0xf85149))
                                        .child(format!("-{}", modal.diff.deletions)),
                                )
                                // 关闭按钮
                                .child(
                                    div()
                                        .px_2()
                                        .py_1()
                                        .rounded_md()
                                        .cursor_pointer()
                                        .text_size(px(12.0))
                                        .text_color(pal.text_secondary)
                                        .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
                                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                            model_btn_close.update(cx, |st, cx| {
                                                st.close_diff_modal();
                                                cx.notify();
                                            });
                                        })
                                        .child("✕"),
                                ),
                        )
                })

                // 2. 多文件切换横条 (如果本次提交修改了多个文件)
                .children((modal.available_files.len() > 1).then(|| {
                    let files = modal.available_files.clone();
                    let active_fp = current_file_path.clone();
                    let hash_clone = commit_hash_opt.clone();
                    let m_switch = model_file_switch.clone();

                    div()
                        .id("diff_modal_files_strip")
                        .h(px(30.0))
                        .px_2()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .overflow_x_scroll()
                        .bg(pal.panel_surface)
                        .border_b_1()
                        .border_color(pal.panel_border)
                        .children(files.into_iter().map(move |fp| {
                            let is_active = fp == active_fp;
                            let fname = std::path::Path::new(&fp)
                                .file_name()
                                .and_then(|s| s.to_str())
                                .unwrap_or(&fp)
                                .to_string();
                            let fp_click = fp.clone();
                            let h_click = hash_clone.clone();
                            let m_sub = m_switch.clone();

                            div()
                                .px_2()
                                .py_0p5()
                                .rounded_sm()
                                .cursor_pointer()
                                .text_size(px(10.0))
                                .border_1()
                                .border_color(if is_active { pal.primary_accent } else { pal.panel_border })
                                .bg(if is_active { pal.card_hover } else { pal.card_surface })
                                .text_color(if is_active { pal.primary_accent } else { pal.text_secondary })
                                .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    let fp_c = fp_click.clone();
                                    let h_c = h_click.clone();
                                    m_sub.update(cx, |st, cx| {
                                        if let Some(ref hash) = h_c {
                                            st.open_commit_file_diff(hash, &fp_c);
                                        } else {
                                            st.open_working_file_diff(&fp_c, false);
                                        }
                                        cx.notify();
                                    });
                                })
                                .child(fname)
                        }))
                }))

                // 3. 差异代码渲染主体
                .child(
                    div()
                        .id("diff_modal_lines_body")
                        .flex_1()
                        .overflow_y_scroll()
                        .bg(pal.panel_surface)
                        .p_2()
                        .children(if modal.diff.lines.is_empty() {
                            vec![
                                div()
                                    .h_full()
                                    .flex()
                                    .flex_col()
                                    .items_center()
                                    .justify_center()
                                    .gap_2()
                                    .text_color(pal.text_secondary)
                                    .child(div().text_size(px(24.0)).child("🔍"))
                                    .child(div().text_size(px(11.5)).child("该文件未发生内容变更，或为二进制格式"))
                                    .into_any_element(),
                            ]
                        } else {
                            modal.diff.lines.into_iter().map(|line| {
                                let (bg_color, text_color, prefix) = match line.line_type {
                                    GitDiffLineType::Addition => (
                                        rgba(0x2ea04320),
                                        rgb(0x3fb950),
                                        "+",
                                    ),
                                    GitDiffLineType::Deletion => (
                                        rgba(0xf8514920),
                                        rgb(0xf85149),
                                        "-",
                                    ),
                                    GitDiffLineType::Header => (
                                        rgba(0x388bfd1a),
                                        rgb(0x58a6ff),
                                        "@",
                                    ),
                                    GitDiffLineType::Context => (
                                        rgba(0x00000000),
                                        pal.text_primary,
                                        " ",
                                    ),
                                };

                                let old_str = line.old_line_num.map(|n| n.to_string()).unwrap_or_default();
                                let new_str = line.new_line_num.map(|n| n.to_string()).unwrap_or_default();

                                div()
                                    .flex()
                                    .items_center()
                                    .min_h(px(18.0))
                                    .bg(bg_color)
                                    .hover(|s| s.bg(pal.card_hover))
                                    // 旧行号
                                    .child(
                                        div()
                                            .w(px(36.0))
                                            .text_size(px(9.5))
                                            .text_color(pal.text_secondary)
                                            .text_align(TextAlign::Right)
                                            .pr_1()
                                            .flex_shrink_0()
                                            .child(old_str),
                                    )
                                    // 新行号
                                    .child(
                                        div()
                                            .w(px(36.0))
                                            .text_size(px(9.5))
                                            .text_color(pal.text_secondary)
                                            .text_align(TextAlign::Right)
                                            .pr_1()
                                            .flex_shrink_0()
                                            .child(new_str),
                                    )
                                    // +/- 标志位
                                    .child(
                                        div()
                                            .w(px(14.0))
                                            .text_size(px(10.0))
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(text_color)
                                            .text_align(TextAlign::Center)
                                            .flex_shrink_0()
                                            .child(prefix),
                                    )
                                    // 代码内容行
                                    .child(
                                        div()
                                            .flex_1()
                                            .text_size(px(11.0))
                                            .text_color(text_color)
                                            .whitespace_nowrap()
                                            .overflow_hidden()
                                            .pl_1()
                                            .child(line.text),
                                    )
                                    .into_any_element()
                            }).collect()
                        })
                )

                // 4. 底部快捷栏与按键提示
                .child(
                    div()
                        .h(px(28.0))
                        .px_4()
                        .flex()
                        .items_center()
                        .justify_between()
                        .border_t_1()
                        .border_color(pal.panel_border)
                        .bg(pal.card_surface)
                        .child(
                            div()
                                .text_size(px(9.5))
                                .text_color(pal.text_secondary)
                                .child("💡 提示：按 Esc 键可快速退出差异对比窗口"),
                        )
                        .child(
                            div()
                                .text_size(px(9.5))
                                .text_color(pal.primary_accent)
                                .child(format!("共 {} 行差异", total_diff_lines)),
                        ),
                ),
        )
        .into_any_element()
}
