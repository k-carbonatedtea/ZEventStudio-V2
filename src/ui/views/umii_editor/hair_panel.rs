//! UMii 发型选择、发色色板与左右翻转控制面板

use gpui::*;
use crate::model::umii::parts::{collect_model_search_dirs, resolve_hair_index};
use crate::ui::state::AppState;
use crate::ui::theme::ThemePalette;

pub const HAIR_COLORS: [(u32, &str); 8] = [
    (0x1B1B1B, "乌黑"), (0x4A2E18, "深褐"), (0x8B3A1C, "赤棕"), (0x9E6B47, "浅棕"),
    (0x8C8C8C, "灰白"), (0xEAEAEA, "雪白"), (0xD4B258, "金发"), (0x6D5D4E, "亚麻"),
];

pub fn render_hair_panel(
    state: &AppState,
    model: &Entity<AppState>,
    pal: &ThemePalette,
    tab_id: &str,
) -> impl IntoElement {
    let tab = state.open_tabs.iter().find(|t| t.id == tab_id);
    let Some(tab) = tab else { return div().id("hair_panel_empty"); };
    let Some(umii_state) = &tab.umii_state else { return div().id("hair_panel_empty"); };
    let data = &umii_state.data;

    let m_hair = model.clone();
    let tid_hair = tab_id.to_string();

    let m_color = model.clone();
    let tid_color = tab_id.to_string();

    let m_flip = model.clone();
    let tid_flip = tab_id.to_string();

    // 132 款发型分页计算 (每页 12 个，共 11 页)
    let curr_hair = data.hair_type.clamp(0, 131) as usize;
    let page = curr_hair / 12;
    let page_start = page * 12;
    let page_end = (page_start + 12).min(132);

    let m_prev = model.clone();
    let tid_prev = tab_id.to_string();
    let m_next = model.clone();
    let tid_next = tab_id.to_string();

    div()
        .id("umii_hair_panel")
        .flex()
        .flex_col()
        .gap_4()
        .p_4()
        // 1. 发型网格 (带分页控件与官方回退指示)
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1p5()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(pal.text_secondary)
                                .child(format!("{} (132 款发型)", state.t("umii.hair_type"))),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1p5()
                                .child(
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_md()
                                        .bg(pal.card_surface)
                                        .border_1()
                                        .border_color(pal.card_border)
                                        .text_xs()
                                        .cursor_pointer()
                                        .hover(|s| s.bg(pal.card_hover))
                                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                            if page > 0 {
                                                let new_hair = ((page - 1) * 12) as i32;
                                                let tid = tid_prev.clone();
                                                m_prev.update(cx, |st, cx| {
                                                    if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                                        if let Some(ref mut u) = t.umii_state {
                                                            u.data.hair_type = new_hair;
                                                            t.is_modified = true;
                                                            let dirs = collect_model_search_dirs(u.target_platform, st.project_path.as_deref());
                                                            let _ = u.synthesize_model(&dirs);
                                                        }
                                                    }
                                                    cx.notify();
                                                });
                                            }
                                        })
                                        .child("◀"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(pal.text_secondary)
                                        .child(format!("{}/11", page + 1)),
                                )
                                .child(
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_md()
                                        .bg(pal.card_surface)
                                        .border_1()
                                        .border_color(pal.card_border)
                                        .text_xs()
                                        .cursor_pointer()
                                        .hover(|s| s.bg(pal.card_hover))
                                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                            if page < 10 {
                                                let new_hair = ((page + 1) * 12) as i32;
                                                let tid = tid_next.clone();
                                                m_next.update(cx, |st, cx| {
                                                    if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                                        if let Some(ref mut u) = t.umii_state {
                                                            u.data.hair_type = new_hair;
                                                            t.is_modified = true;
                                                            let dirs = collect_model_search_dirs(u.target_platform, st.project_path.as_deref());
                                                            let _ = u.synthesize_model(&dirs);
                                                        }
                                                    }
                                                    cx.notify();
                                                });
                                            }
                                        })
                                        .child("▶"),
                                ),
                        ),
                )
                .child(
                    div()
                        .grid()
                        .grid_cols(4)
                        .gap_1p5()
                        .children((page_start..page_end).map(|idx| {
                            let val = idx as i32;
                            let is_sel = data.hair_type == val;
                            let target_idx = resolve_hair_index(val);
                            let has_fb = target_idx != val;
                            let m = m_hair.clone();
                            let tid = tid_hair.clone();

                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .justify_center()
                                .py_2()
                                .px_1()
                                .rounded_md()
                                .bg(if is_sel { rgb(0x1E3A8A) } else { pal.card_surface })
                                .border_1()
                                .border_color(if is_sel { rgb(0x3B82F6) } else { pal.card_border })
                                .cursor_pointer()
                                .hover(|s| s.border_color(rgb(0x60A5FA)))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    m.update(cx, |st, cx| {
                                        if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                            if let Some(ref mut u) = t.umii_state {
                                                u.data.hair_type = val;
                                                t.is_modified = true;
                                                let dirs = collect_model_search_dirs(u.target_platform, st.project_path.as_deref());
                                                let _ = u.synthesize_model(&dirs);
                                            }
                                        }
                                        cx.notify();
                                    });
                                })
                                .child(div().text_sm().child("💇"))
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                        .text_color(if is_sel { rgb(0xFFFFFF) } else { pal.text_primary })
                                        .child(format!("#{:03}", val)),
                                )
                                .children(has_fb.then(|| {
                                    div()
                                        .text_size(px(9.0))
                                        .text_color(rgb(0xF59E0B))
                                        .child(format!("➔ #{:03}", target_idx))
                                }))
                        }))
                ),
        )
        // 2. 发色调色盘 (8色)
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1p5()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.text_secondary)
                        .child(state.t("umii.hair_color")),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .flex_wrap()
                        .gap_2()
                        .p_2()
                        .bg(pal.card_surface)
                        .border_1()
                        .border_color(pal.card_border)
                        .rounded_lg()
                        .children(HAIR_COLORS.iter().enumerate().map(|(idx, &(hex, _))| {
                            let is_sel = data.hair_color as usize == idx;
                            let m = m_color.clone();
                            let tid = tid_color.clone();
                            div()
                                .id(ElementId::NamedInteger("hair_color".into(), idx as u64))
                                .size(px(26.0))
                                .rounded_full()
                                .bg(rgb(hex))
                                .border_2()
                                .border_color(if is_sel { rgb(0x60A5FA) } else { rgba((hex << 8) | 0x44) })
                                .cursor_pointer()
                                .hover(|s| s.opacity(0.85))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    m.update(cx, |st, cx| {
                                        if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                            if let Some(ref mut u) = t.umii_state {
                                                u.data.hair_color = idx as i32;
                                                u.data.eyebrow_color = idx as i32;
                                                u.data.beard_color = idx as i32;
                                                t.is_modified = true;
                                            }
                                        }
                                        cx.notify();
                                    });
                                })
                        }))
                ),
        )
        // 3. 发型左右翻折 (Flip)
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .p_3()
                .bg(pal.card_surface)
                .border_1()
                .border_color(pal.card_border)
                .rounded_lg()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(pal.text_primary)
                        .child(state.t("umii.hair_flip")),
                )
                .child(
                    div()
                        .px_3()
                        .py_1()
                        .rounded_md()
                        .bg(if data.hair_flip { rgb(0x2563EB) } else { pal.panel_surface })
                        .border_1()
                        .border_color(if data.hair_flip { rgb(0x60A5FA) } else { pal.card_border })
                        .cursor_pointer()
                        .hover(|s| s.opacity(0.85))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            let tid = tid_flip.clone();
                            m_flip.update(cx, |st, cx| {
                                if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                    if let Some(ref mut u) = t.umii_state {
                                        u.data.hair_flip = !u.data.hair_flip;
                                        t.is_modified = true;
                                    }
                                }
                                cx.notify();
                            });
                        })
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(if data.hair_flip { rgb(0xFFFFFF) } else { pal.text_secondary })
                                .child(if data.hair_flip { "已水平翻转 ⇄" } else { "标准未翻转 ➔" }),
                        ),
                ),
        )
}
