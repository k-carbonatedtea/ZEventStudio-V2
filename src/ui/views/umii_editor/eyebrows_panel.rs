//! UMii 眉毛样式、色彩与微调罗盘面板

use gpui::*;
use crate::ui::state::AppState;
use crate::ui::theme::ThemePalette;
use super::components::render_fine_adjuster;
use super::hair_panel::HAIR_COLORS;

pub fn render_eyebrows_panel(
    state: &AppState,
    model: &Entity<AppState>,
    pal: &ThemePalette,
    tab_id: &str,
) -> impl IntoElement {
    let tab = state.open_tabs.iter().find(|t| t.id == tab_id);
    let Some(tab) = tab else { return div().id("eyebrows_panel_empty"); };
    let Some(umii_state) = &tab.umii_state else { return div().id("eyebrows_panel_empty"); };
    let data = &umii_state.data;

    let m_type = model.clone();
    let tid_type = tab_id.to_string();

    let m_color = model.clone();
    let tid_color = tab_id.to_string();

    let m_v = model.clone();
    let tid_v = tab_id.to_string();

    let m_u = model.clone();
    let tid_u = tab_id.to_string();

    let m_s = model.clone();
    let tid_s = tab_id.to_string();

    let m_r = model.clone();
    let tid_r = tab_id.to_string();

    let curr_brow = data.eyebrow_type.clamp(0, 23) as usize;
    let page = curr_brow / 12;
    let p_start = page * 12;
    let p_end = (p_start + 12).min(24);

    let m_prev = model.clone();
    let tid_prev = tab_id.to_string();
    let m_next = model.clone();
    let tid_next = tab_id.to_string();

    div()
        .id("umii_eyebrows_panel")
        .flex()
        .flex_col()
        .gap_4()
        .p_4()
        // 1. 眉毛形态选择网格 (24种，2页)
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
                                .child(format!("{} (24 种形态)", state.t("umii.eyebrow_type"))),
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
                                                let tid = tid_prev.clone();
                                                m_prev.update(cx, |st, cx| {
                                                    if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                                        if let Some(ref mut u) = t.umii_state {
                                                            u.data.eyebrow_type = 0;
                                                            t.is_modified = true;
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
                                        .child(format!("{}/2", page + 1)),
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
                                            if page == 0 {
                                                let tid = tid_next.clone();
                                                m_next.update(cx, |st, cx| {
                                                    if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                                        if let Some(ref mut u) = t.umii_state {
                                                            u.data.eyebrow_type = 12;
                                                            t.is_modified = true;
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
                        .children((p_start..p_end).map(|idx| {
                            let val = idx as i32;
                            let is_sel = data.eyebrow_type == val;
                            let m = m_type.clone();
                            let tid = tid_type.clone();
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
                                                u.data.eyebrow_type = val;
                                                t.is_modified = true;
                                            }
                                        }
                                        cx.notify();
                                    });
                                })
                                .child(div().text_sm().child("🤨"))
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                        .text_color(if is_sel { rgb(0xFFFFFF) } else { pal.text_primary })
                                        .child(format!("#{:02}", val + 1)),
                                )
                        })),
                ),
        )
        // 2. 眉色色板
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
                        .child(state.t("umii.eyebrow_color")),
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
                            let is_sel = data.eyebrow_color as usize == idx;
                            let m = m_color.clone();
                            let tid = tid_color.clone();
                            div()
                                .id(ElementId::NamedInteger("brow_color".into(), idx as u64))
                                .size(px(24.0))
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
                                                u.data.eyebrow_color = idx as i32;
                                                t.is_modified = true;
                                            }
                                        }
                                        cx.notify();
                                    });
                                })
                        })),
                ),
        )
        // 3. 微调罗盘控制台 (高度、间距、尺寸、旋转)
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(render_fine_adjuster(
                    state.t("umii.vertical_pos"),
                    format!("{:.1}", data.eyebrow_trans_v),
                    state.t("umii.dpad_down"),
                    state.t("umii.dpad_up"),
                    {
                        let m = m_v.clone();
                        let tid = tid_v.clone();
                        move |_, _, cx| {
                            m.update(cx, |st, cx| {
                                if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                    if let Some(ref mut u) = t.umii_state {
                                        u.data.eyebrow_trans_v = (u.data.eyebrow_trans_v - 0.5).max(0.0);
                                        t.is_modified = true;
                                    }
                                }
                                cx.notify();
                            });
                        }
                    },
                    {
                        let m = m_v;
                        let tid = tid_v;
                        move |_, _, cx| {
                            m.update(cx, |st, cx| {
                                if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                    if let Some(ref mut u) = t.umii_state {
                                        u.data.eyebrow_trans_v = (u.data.eyebrow_trans_v + 0.5).min(18.0);
                                        t.is_modified = true;
                                    }
                                }
                                cx.notify();
                            });
                        }
                    },
                    pal,
                ))
                .child(render_fine_adjuster(
                    state.t("umii.spacing"),
                    format!("{:.1}", data.eyebrow_trans_u),
                    state.t("umii.dpad_in"),
                    state.t("umii.dpad_out"),
                    {
                        let m = m_u.clone();
                        let tid = tid_u.clone();
                        move |_, _, cx| {
                            m.update(cx, |st, cx| {
                                if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                    if let Some(ref mut u) = t.umii_state {
                                        u.data.eyebrow_trans_u = (u.data.eyebrow_trans_u - 0.5).max(0.0);
                                        t.is_modified = true;
                                    }
                                }
                                cx.notify();
                            });
                        }
                    },
                    {
                        let m = m_u;
                        let tid = tid_u;
                        move |_, _, cx| {
                            m.update(cx, |st, cx| {
                                if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                    if let Some(ref mut u) = t.umii_state {
                                        u.data.eyebrow_trans_u = (u.data.eyebrow_trans_u + 0.5).min(16.0);
                                        t.is_modified = true;
                                    }
                                }
                                cx.notify();
                            });
                        }
                    },
                    pal,
                ))
                .child(render_fine_adjuster(
                    state.t("umii.scale"),
                    format!("{:.1}", data.eyebrow_scale),
                    state.t("umii.dpad_smaller"),
                    state.t("umii.dpad_larger"),
                    {
                        let m = m_s.clone();
                        let tid = tid_s.clone();
                        move |_, _, cx| {
                            m.update(cx, |st, cx| {
                                if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                    if let Some(ref mut u) = t.umii_state {
                                        u.data.eyebrow_scale = (u.data.eyebrow_scale - 0.5).max(0.0);
                                        t.is_modified = true;
                                    }
                                }
                                cx.notify();
                            });
                        }
                    },
                    {
                        let m = m_s;
                        let tid = tid_s;
                        move |_, _, cx| {
                            m.update(cx, |st, cx| {
                                if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                    if let Some(ref mut u) = t.umii_state {
                                        u.data.eyebrow_scale = (u.data.eyebrow_scale + 0.5).min(8.0);
                                        t.is_modified = true;
                                    }
                                }
                                cx.notify();
                            });
                        }
                    },
                    pal,
                ))
                .child(render_fine_adjuster(
                    state.t("umii.rotation"),
                    format!("{:.1}", data.eyebrow_rotate),
                    state.t("umii.dpad_rot_left"),
                    state.t("umii.dpad_rot_right"),
                    {
                        let m = m_r.clone();
                        let tid = tid_r.clone();
                        move |_, _, cx| {
                            m.update(cx, |st, cx| {
                                if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                    if let Some(ref mut u) = t.umii_state {
                                        u.data.eyebrow_rotate = (u.data.eyebrow_rotate - 0.5).max(-6.0);
                                        t.is_modified = true;
                                    }
                                }
                                cx.notify();
                            });
                        }
                    },
                    {
                        let m = m_r;
                        let tid = tid_r;
                        move |_, _, cx| {
                            m.update(cx, |st, cx| {
                                if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                    if let Some(ref mut u) = t.umii_state {
                                        u.data.eyebrow_rotate = (u.data.eyebrow_rotate + 0.5).min(6.0);
                                        t.is_modified = true;
                                    }
                                }
                                cx.notify();
                            });
                        }
                    },
                    pal,
                )),
        )
}
