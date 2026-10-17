use gpui::*;
use crate::model::DiffViewerTabState;
use crate::ui::state::AppState;
use crate::ui::theme::ThemePalette;

/// 渲染对比模式顶部的面包屑导航与差异统计工具栏 (Breadcrumbs & Diff Stats Header)
/// 严格还原图2中的 VS Code 风格路径面包屑与对比指示
pub fn render_diff_header(
    diff_state: &DiffViewerTabState,
    model: &Entity<AppState>,
    tab_id: &str,
    pal: &ThemePalette,
) -> impl IntoElement {
    let full_path = &diff_state.full_path;
    let curr_diff = diff_state.current_diff();
    let additions = curr_diff.additions;
    let deletions = curr_diff.deletions;

    // 解析主文件面包屑路径分节
    let mut segments: Vec<String> = full_path
        .split(['/', '\\'])
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    // 如果为 SARC 内部子文件，将内部子路径也平滑拼接入面包屑中
    if let Some(sub_path) = diff_state.current_sub_path() {
        for sub_seg in sub_path.split(['/', '\\']).filter(|s| !s.is_empty()) {
            segments.push(sub_seg.to_string());
        }
    }

    let total_segments = segments.len();
    let has_multiple_subs = diff_state.sub_files.len() > 1;

    div()
        .w_full()
        .flex()
        .flex_col()
        .flex_shrink_0()
        // 1. 顶层面包屑与版本统计条
        .child(
            div()
                .h(px(28.0))
                .w_full()
                .px_3()
                .bg(pal.panel_surface)
                .border_b_1()
                .border_color(pal.panel_border)
                .flex()
                .items_center()
                .justify_between()
                .flex_shrink_0()
                // 左侧：面包屑路径 + 格式徽章
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .overflow_hidden()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .overflow_hidden()
                                .children(segments.into_iter().enumerate().map(|(idx, seg)| {
                                    let is_last = idx + 1 == total_segments;
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_1()
                                        .child(
                                            div()
                                                .text_size(px(10.5))
                                                .font_weight(if is_last { FontWeight::BOLD } else { FontWeight::NORMAL })
                                                .text_color(if is_last { pal.text_primary } else { pal.text_secondary })
                                                .child(seg),
                                        )
                                        .children((!is_last).then(|| {
                                            div()
                                                .text_size(px(9.5))
                                                .text_color(rgba(0xffffff33))
                                                .child("›")
                                        }))
                                })),
                        )
                        // 格式徽章 (例如 [AAMP] / [BYML] / [MSBT] / [BFEVFL])
                        .children(diff_state.current_format_name().map(|fmt| {
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_xs()
                                .bg(rgba(0x388bfd1a))
                                .border_1()
                                .border_color(rgba(0x388bfd44))
                                .text_size(px(9.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0x58a6ff))
                                .child(fmt.to_string())
                        })),
                )
                // 右侧：版本对照徽章与差异统计
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .flex_shrink_0()
                        // 版本对照：如 0620348 ↔ 6384de4
                        .child(
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded_xs()
                                .bg(pal.card_surface)
                                .border_1()
                                .border_color(pal.panel_border)
                                .text_size(px(9.5))
                                .text_color(pal.text_secondary)
                                .child(format!("{} ↔ {}", diff_state.old_rev, diff_state.new_rev)),
                        )
                        // 新增行数统计 (+N)
                        .children((additions > 0).then(|| {
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_xs()
                                .bg(rgba(0x2ea04326))
                                .border_1()
                                .border_color(rgba(0x2ea04344))
                                .text_size(px(9.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0x3fb950))
                                .child(format!("+{}", additions))
                        }))
                        // 删除行数统计 (-N)
                        .children((deletions > 0).then(|| {
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_xs()
                                .bg(rgba(0xf8514926))
                                .border_1()
                                .border_color(rgba(0xf8514944))
                                .text_size(px(9.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xf85149))
                                .child(format!("-{}", deletions))
                        })),
                ),
        )
        // 2. 如果 SARC 封包内有多个变更子文件，渲染可切换的子文件选择药丸条 (Sub-files navigation strip)
        .children(has_multiple_subs.then(|| {
            let active_idx = diff_state.active_sub_file_idx;
            let tab_id_str = tab_id.to_string();

            div()
                .h(px(28.0))
                .w_full()
                .px_3()
                .bg(pal.card_surface)
                .border_b_1()
                .border_color(pal.panel_border)
                .flex()
                .items_center()
                .gap_2()
                .overflow_hidden()
                .child(
                    div()
                        .text_size(px(9.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.text_secondary)
                        .flex_shrink_0()
                        .child(format!("📦 封包内变动文件 ({}):", diff_state.sub_files.len())),
                )
                .children(diff_state.sub_files.iter().enumerate().map(|(idx, sub)| {
                    let is_active = idx == active_idx;
                    let m = model.clone();
                    let tid = tab_id_str.clone();

                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_xs()
                        .cursor_pointer()
                        .bg(if is_active { rgba(0x388bfd24) } else { pal.panel_surface })
                        .border_1()
                        .border_color(if is_active { pal.primary_accent } else { pal.panel_border })
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .flex_shrink_0()
                        .hover(|s| s.bg(pal.card_hover))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            m.update(cx, |st, cx| {
                                st.switch_diff_sub_file(&tid, idx);
                                cx.notify();
                            });
                        })
                        .child(
                            div()
                                .text_size(px(9.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(if is_active { pal.primary_accent } else { pal.text_secondary })
                                .child(format!("[{}]", sub.format_name)),
                        )
                        .child(
                            div()
                                .text_size(px(9.5))
                                .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
                                .text_color(if is_active { pal.text_primary } else { pal.text_secondary })
                                .child(sub.sub_path.clone()),
                        )
                        .child(
                            div()
                                .text_size(px(8.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0x3fb950))
                                .child(format!("+{}", sub.diff.additions)),
                        )
                        .child(
                            div()
                                .text_size(px(8.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xf85149))
                                .child(format!("-{}", sub.diff.deletions)),
                        )
                }))
        }))
}
