//! UMii 编辑器通用视觉组件库 (微调罗盘控制台与分页网格卡片)

use crate::ui::state::AppState;
use crate::ui::theme::ThemePalette;
use gpui::*;

/// 渲染微调罗盘控制台 (上下/间距/大小/旋转等调节)
pub fn render_fine_adjuster<FDec, FInc>(
    title: &str,
    val_display: String,
    dec_label: &'static str,
    inc_label: &'static str,
    on_dec: FDec,
    on_inc: FInc,
    pal: &ThemePalette,
) -> impl IntoElement
where
    FDec: Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    FInc: Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
{
    div()
        .flex()
        .items_center()
        .justify_between()
        .px_3()
        .py_1p5()
        .bg(pal.card_surface)
        .border_1()
        .border_color(pal.card_border)
        .rounded_md()
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(pal.text_primary)
                .child(title.to_string()),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .bg(pal.panel_surface)
                        .border_1()
                        .border_color(pal.card_border)
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.primary_accent)
                        .child(val_display),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded_md()
                                .bg(pal.card_hover)
                                .border_1()
                                .border_color(pal.card_border)
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(pal.text_primary)
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0x3B82F6)).text_color(rgb(0xFFFFFF)))
                                .on_mouse_down(MouseButton::Left, on_dec)
                                .child(dec_label),
                        )
                        .child(
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded_md()
                                .bg(pal.card_hover)
                                .border_1()
                                .border_color(pal.card_border)
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(pal.text_primary)
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0x3B82F6)).text_color(rgb(0xFFFFFF)))
                                .on_mouse_down(MouseButton::Left, on_inc)
                                .child(inc_label),
                        ),
                ),
        )
}

/// 渲染分页网格顶部栏
#[allow(dead_code)]
pub fn render_paging_header<FPrev, FNext>(
    title: &str,
    curr_page: usize,
    total_pages: usize,
    on_prev: FPrev,
    on_next: FNext,
    pal: &ThemePalette,
    _state: &AppState,
) -> impl IntoElement
where
    FPrev: Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    FNext: Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
{
    div()
        .flex()
        .items_center()
        .justify_between()
        .pb_1()
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(pal.text_secondary)
                .child(title.to_string()),
        )
        .children((total_pages > 1).then(|| {
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .child(
                    div()
                        .px_1p5()
                        .py_0p5()
                        .rounded_sm()
                        .bg(pal.card_surface)
                        .border_1()
                        .border_color(pal.card_border)
                        .text_xs()
                        .cursor_pointer()
                        .hover(|s| s.bg(pal.card_hover))
                        .on_mouse_down(MouseButton::Left, on_prev)
                        .child("◀"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(pal.text_secondary)
                        .child(format!("{}/{}", curr_page + 1, total_pages)),
                )
                .child(
                    div()
                        .px_1p5()
                        .py_0p5()
                        .rounded_sm()
                        .bg(pal.card_surface)
                        .border_1()
                        .border_color(pal.card_border)
                        .text_xs()
                        .cursor_pointer()
                        .hover(|s| s.bg(pal.card_hover))
                        .on_mouse_down(MouseButton::Left, on_next)
                        .child("▶"),
                )
        }))
}

/// 渲染数值步进器控制行 (➖ [数值] ➕)
pub fn render_stepper<FDec, FInc>(
    label: &str,
    val: i32,
    pal: &ThemePalette,
    on_dec: FDec,
    on_inc: FInc,
) -> impl IntoElement
where
    FDec: Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    FInc: Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
{
    div()
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(pal.text_primary)
                .child(label.to_string()),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .bg(pal.panel_surface)
                        .border_1()
                        .border_color(pal.card_border)
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.primary_accent)
                        .child(format!("{}", val)),
                )
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .bg(pal.card_hover)
                        .cursor_pointer()
                        .hover(|s| s.bg(rgb(0x3B82F6)))
                        .on_mouse_down(MouseButton::Left, on_dec)
                        .child("➖"),
                )
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .bg(pal.card_hover)
                        .cursor_pointer()
                        .hover(|s| s.bg(rgb(0x3B82F6)))
                        .on_mouse_down(MouseButton::Left, on_inc)
                        .child("➕"),
                ),
        )
}
