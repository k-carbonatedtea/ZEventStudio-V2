//! UMii 胡须 (小胡子 Mustache 与 络腮胡 Beard) 样式、色彩与微调罗盘面板

use super::components::render_fine_adjuster;
use super::hair_panel::HAIR_COLORS;
use crate::model::umii::parts::collect_model_search_dirs;
use crate::ui::state::AppState;
use crate::ui::theme::ThemePalette;
use gpui::*;

pub fn render_beard_panel(
    state: &AppState,
    model: &Entity<AppState>,
    pal: &ThemePalette,
    tab_id: &str,
) -> impl IntoElement {
    let tab = state.open_tabs.iter().find(|t| t.id == tab_id);
    let Some(tab) = tab else {
        return div().id("beard_panel_empty");
    };
    let Some(umii_state) = &tab.umii_state else {
        return div().id("beard_panel_empty");
    };
    let data = &umii_state.data;

    let m_mustache = model.clone();
    let tid_mustache = tab_id.to_string();

    let m_beard = model.clone();
    let tid_beard = tab_id.to_string();

    let m_color = model.clone();
    let tid_color = tab_id.to_string();

    let m_s = model.clone();
    let tid_s = tab_id.to_string();

    div()
        .id("umii_beard_panel")
        .flex()
        .flex_col()
        .gap_4()
        .p_4()
        // 1. 小胡子 (Mustache: 4种)
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
                        .child(format!("{} (Mustache)", state.t("umii.mustache"))),
                )
                .child(
                    div().grid().grid_cols(4).gap_1p5().children(
                        [
                            (0, "无", "🚫"),
                            (1, "一字胡", "〰️"),
                            (2, "八字胡", "👨"),
                            (3, "浓密翘胡", "🥸"),
                        ]
                        .into_iter()
                        .map(|(val, label, icon)| {
                            let is_sel = data.beard_mustache == val;
                            let m = m_mustache.clone();
                            let tid = tid_mustache.clone();
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .justify_center()
                                .py_2()
                                .px_1()
                                .rounded_md()
                                .bg(if is_sel {
                                    rgb(0x1E3A8A)
                                } else {
                                    pal.card_surface
                                })
                                .border_1()
                                .border_color(if is_sel {
                                    rgb(0x3B82F6)
                                } else {
                                    pal.card_border
                                })
                                .cursor_pointer()
                                .hover(|s| s.border_color(rgb(0x60A5FA)))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    m.update(cx, |st, cx| {
                                        if let Some(t) =
                                            st.open_tabs.iter_mut().find(|t| t.id == tid)
                                        {
                                            if let Some(ref mut u) = t.umii_state {
                                                u.data.beard_mustache = val;
                                                t.is_modified = true;
                                                let dirs = collect_model_search_dirs(
                                                    u.target_platform,
                                                    st.project_path.as_deref(),
                                                );
                                                let _ = u.synthesize_model(&dirs);
                                            }
                                        }
                                        cx.notify();
                                    });
                                })
                                .child(div().text_sm().child(icon))
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(if is_sel {
                                            FontWeight::BOLD
                                        } else {
                                            FontWeight::NORMAL
                                        })
                                        .text_color(if is_sel {
                                            rgb(0xFFFFFF)
                                        } else {
                                            pal.text_primary
                                        })
                                        .child(label),
                                )
                        }),
                    ),
                ),
        )
        // 2. 络腮大胡须 (Beard: 4种)
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
                        .child(format!("{} (Beard)", state.t("umii.beard_type"))),
                )
                .child(
                    div().grid().grid_cols(4).gap_1p5().children(
                        [
                            (0, "无", "🚫"),
                            (1, "山羊胡", "🐐"),
                            (2, "络腮连鬓胡", "🧔"),
                            (3, "浓密大胡", "🧙"),
                        ]
                        .into_iter()
                        .map(|(val, label, icon)| {
                            let is_sel = data.beard_type == val;
                            let m = m_beard.clone();
                            let tid = tid_beard.clone();
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .justify_center()
                                .py_2()
                                .px_1()
                                .rounded_md()
                                .bg(if is_sel {
                                    rgb(0x4338CA)
                                } else {
                                    pal.card_surface
                                })
                                .border_1()
                                .border_color(if is_sel {
                                    rgb(0x818CF8)
                                } else {
                                    pal.card_border
                                })
                                .cursor_pointer()
                                .hover(|s| s.border_color(rgb(0x6366F1)))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    m.update(cx, |st, cx| {
                                        if let Some(t) =
                                            st.open_tabs.iter_mut().find(|t| t.id == tid)
                                        {
                                            if let Some(ref mut u) = t.umii_state {
                                                u.data.beard_type = val;
                                                t.is_modified = true;
                                                let dirs = collect_model_search_dirs(
                                                    u.target_platform,
                                                    st.project_path.as_deref(),
                                                );
                                                let _ = u.synthesize_model(&dirs);
                                            }
                                        }
                                        cx.notify();
                                    });
                                })
                                .child(div().text_sm().child(icon))
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(if is_sel {
                                            FontWeight::BOLD
                                        } else {
                                            FontWeight::NORMAL
                                        })
                                        .text_color(if is_sel {
                                            rgb(0xFFFFFF)
                                        } else {
                                            pal.text_primary
                                        })
                                        .child(label),
                                )
                        }),
                    ),
                ),
        )
        // 3. 胡须色彩调色盘 (8色)
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
                        .child(state.t("umii.beard_color")),
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
                            let is_sel = data.beard_color as usize == idx;
                            let m = m_color.clone();
                            let tid = tid_color.clone();
                            div()
                                .id(ElementId::NamedInteger("beard_color".into(), idx as u64))
                                .size(px(26.0))
                                .rounded_full()
                                .bg(rgb(hex))
                                .border_2()
                                .border_color(if is_sel {
                                    rgb(0x60A5FA)
                                } else {
                                    rgba((hex << 8) | 0x44)
                                })
                                .cursor_pointer()
                                .hover(|s| s.opacity(0.85))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    m.update(cx, |st, cx| {
                                        if let Some(t) =
                                            st.open_tabs.iter_mut().find(|t| t.id == tid)
                                        {
                                            if let Some(ref mut u) = t.umii_state {
                                                u.data.beard_color = idx as i32;
                                                t.is_modified = true;
                                            }
                                        }
                                        cx.notify();
                                    });
                                })
                        })),
                ),
        )
        // 4. 小胡子尺寸缩放微调罗盘
        .child(div().flex().flex_col().gap_2().child(render_fine_adjuster(
            state.t("umii.scale"),
            format!("{:.1}", data.beard_scale),
            state.t("umii.dpad_smaller"),
            state.t("umii.dpad_larger"),
            {
                let m = m_s.clone();
                let tid = tid_s.clone();
                move |_, _, cx| {
                    m.update(cx, |st, cx| {
                        if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                            if let Some(ref mut u) = t.umii_state {
                                u.data.beard_scale = (u.data.beard_scale - 0.5).max(0.0);
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
                                u.data.beard_scale = (u.data.beard_scale + 0.5).min(8.0);
                                t.is_modified = true;
                            }
                        }
                        cx.notify();
                    });
                }
            },
            pal,
        )))
}
