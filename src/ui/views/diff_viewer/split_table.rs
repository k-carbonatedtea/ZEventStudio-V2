use gpui::*;
use crate::model::git_diff_align::{DiffCellKind, DiffSideCell, SideBySideRow};
use crate::model::DiffViewerTabState;
use crate::ui::theme::ThemePalette;
use super::hatch_pattern::render_diagonal_hatch_placeholder;

/// 渲染 Side-by-Side 双栏差异对比表格主体
pub fn render_diff_split_table(
    state: &crate::ui::state::AppState,
    diff_state: &DiffViewerTabState,
    pal: &ThemePalette,
) -> impl IntoElement {
    let rows = diff_state.current_rows();
    let file_display = diff_state.current_sub_path().unwrap_or(&diff_state.file_path);
    let old_rev_label = format!("{} ({})", file_display, diff_state.old_rev);
    let new_rev_label = format!("{} ({})", file_display, diff_state.new_rev);

    div()
        .flex_1()
        .flex()
        .flex_col()
        .size_full()
        .bg(pal.panel_surface)
        .overflow_hidden()

        // 1. 左右双栏子表头 (Old Revision Header & New Revision Header)
        .child(
            div()
                .h(px(26.0))
                .w_full()
                .flex()
                .items_center()
                .bg(pal.card_surface)
                .border_b_1()
                .border_color(pal.panel_border)
                .flex_shrink_0()
                // 左侧表头
                .child(
                    div()
                        .flex_1()
                        .h_full()
                        .px_3()
                        .border_r_1()
                        .border_color(pal.panel_border)
                        .flex()
                        .items_center()
                        .child(
                            div()
                                .text_size(px(10.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(pal.text_secondary)
                                .whitespace_nowrap()
                                .overflow_hidden()
                                .child(old_rev_label),
                        ),
                )
                // 右侧表头
                .child(
                    div()
                        .flex_1()
                        .h_full()
                        .px_3()
                        .flex()
                        .items_center()
                        .child(
                            div()
                                .text_size(px(10.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(pal.text_primary)
                                .whitespace_nowrap()
                                .overflow_hidden()
                                .child(new_rev_label),
                        ),
                ),
        )

        // 2. 双栏代码对比数据主体
        .child(
            div()
                .id("diff_split_table_scroll")
                .flex_1()
                .size_full()
                .overflow_y_scroll()
                .overflow_x_scroll()
                .children(if rows.is_empty() {
                    vec![
                        div()
                            .h_full()
                            .w_full()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .gap_2()
                            .py_12()
                            .text_color(pal.text_secondary)
                            .child(div().text_size(px(32.0)).child("🔍"))
                            .child(
                                div()
                                    .text_size(px(13.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(pal.text_primary)
                                    .child(state.t("diff.binary_or_no_text")),
                            )
                            .child(
                                div()
                                    .text_size(px(10.5))
                                    .child(format!("{}: {}", state.t("diff.file_path_prefix"), file_display)),
                            )
                            .into_any_element(),
                    ]
                } else {
                    rows.iter()
                        .map(|row| match row {
                            SideBySideRow::HunkHeader {
                                header_text,
                                context_hint,
                                ..
                            } => render_hunk_divider(header_text, context_hint, pal).into_any_element(),
                            SideBySideRow::LinePair { left, right } => {
                                render_line_pair(left, right, pal).into_any_element()
                            }
                        })
                        .collect()
                }),
        )
}

/// 渲染 Hunk 分隔条（横跨左右双栏）
fn render_hunk_divider(
    header_text: &str,
    context_hint: &str,
    pal: &ThemePalette,
) -> impl IntoElement {
    div()
        .w_full()
        .h(px(22.0))
        .bg(rgba(0x388bfd14))
        .border_y_1()
        .border_color(rgba(0x388bfd33))
        .px_3()
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .text_size(px(9.5))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0x58a6ff))
                .child(header_text.to_string()),
        )
        .children((!context_hint.is_empty()).then(|| {
            div()
                .text_size(px(9.5))
                .text_color(pal.text_secondary)
                .child(context_hint.to_string())
        }))
}

/// 渲染双栏对齐行对（左右两半绝对同步）
fn render_line_pair(
    left: &DiffSideCell,
    right: &DiffSideCell,
    pal: &ThemePalette,
) -> impl IntoElement {
    div()
        .w_full()
        .min_h(px(20.0))
        .flex()
        .items_stretch()
        .hover(|s| s.bg(pal.card_hover))
        // 左栏单元格
        .child(
            div()
                .flex_1()
                .min_h(px(20.0))
                .border_r_1()
                .border_color(pal.panel_border)
                .child(render_side_cell(left, false, pal)),
        )
        // 右栏单元格
        .child(
            div()
                .flex_1()
                .min_h(px(20.0))
                .child(render_side_cell(right, true, pal)),
        )
}

/// 渲染单侧单元格（行号 + 占位阴影/代码高亮）
fn render_side_cell(
    cell: &DiffSideCell,
    _is_right: bool,
    pal: &ThemePalette,
) -> impl IntoElement {
    let (row_bg, text_color, num_color) = match cell.kind {
        DiffCellKind::Context => (
            rgba(0x00000000),
            pal.text_primary,
            pal.text_secondary,
        ),
        DiffCellKind::Addition => (
            rgba(0x2ea04326), // 绿色半透明高亮 (图2效果)
            pal.text_primary,
            rgb(0x3fb950),   // 绿色行号
        ),
        DiffCellKind::Deletion => (
            rgba(0xf8514926), // 红色半透明高亮
            pal.text_primary,
            rgb(0xf85149),   // 红色行号
        ),
        DiffCellKind::Placeholder => (
            rgba(0x00000000),
            pal.text_secondary,
            pal.text_secondary,
        ),
    };

    let gutter_bg = if cell.kind == DiffCellKind::Placeholder {
        rgba(0x00000028)
    } else {
        pal.card_surface
    };

    div()
        .size_full()
        .min_h(px(20.0))
        .flex()
        .items_stretch()
        .bg(row_bg)

        // 1. 行号槽 (Gutter)
        .child(
            div()
                .w(px(46.0))
                .min_h(px(20.0))
                .px_1p5()
                .flex_shrink_0()
                .bg(gutter_bg)
                .border_r_1()
                .border_color(pal.panel_border)
                .text_align(TextAlign::Right)
                .flex()
                .items_center()
                .justify_end()
                .child(
                    div()
                        .text_size(px(10.0))
                        .text_color(num_color)
                        .child(cell.line_label.clone()),
                ),
        )

        // 2. 代码内容行或斜纹占位区
        .child(if cell.kind == DiffCellKind::Placeholder {
            render_diagonal_hatch_placeholder(px(20.0), pal).into_any_element()
        } else {
            let prefix_symbol = if cell.kind == DiffCellKind::Addition {
                "+"
            } else if cell.kind == DiffCellKind::Deletion {
                "-"
            } else {
                " "
            };

            div()
                .flex_1()
                .min_h(px(20.0))
                .flex()
                .items_center()
                .px_2()
                .child(
                    div()
                        .w(px(12.0))
                        .flex_shrink_0()
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(if cell.kind == DiffCellKind::Addition {
                            rgb(0x3fb950)
                        } else if cell.kind == DiffCellKind::Deletion {
                            rgb(0xf85149)
                        } else {
                            rgba(0x00000000)
                        })
                        .child(prefix_symbol),
                )
                .child(
                    div()
                        .flex_1()
                        .text_size(px(11.0))
                        .text_color(text_color)
                        .whitespace_nowrap()
                        .child(cell.text.clone()),
                )
                .into_any_element()
        })
}
