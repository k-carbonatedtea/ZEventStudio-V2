// UMii 脸部轮廓、肤色色板、面部褶皱与妆容面板

use gpui::*;
use super::components::render_fine_adjuster;
use crate::model::umii::parts::collect_model_search_dirs;
use crate::ui::state::AppState;
use crate::ui::theme::ThemePalette;

pub const SKIN_TONES: [(u32, &str); 11] = [
    (0xFFF1E6, "白皙"), (0xFCE3CE, "粉白"), (0xF5D5B8, "象牙"), (0xEEBE94, "自然"),
    (0xE4A877, "小麦"), (0xD8955F, "蜜色"), (0xB87342, "古铜"), (0x96572F, "深褐"),
    (0x733E1D, "深黑"), (0x572F18, "黑巧"), (0x3D1F10, "深沉黝黑"),
];

pub fn render_face_panel(
    state: &AppState,
    model: &Entity<AppState>,
    pal: &ThemePalette,
    tab_id: &str,
) -> impl IntoElement {
    let tab = state.open_tabs.iter().find(|t| t.id == tab_id);
    let Some(tab) = tab else { return div().id("face_panel_empty"); };
    let Some(umii_state) = &tab.umii_state else { return div().id("face_panel_empty"); };
    let data = &umii_state.data;

    let m_jaw = model.clone();
    let tid_jaw = tab_id.to_string();

    let m_skin = model.clone();
    let tid_skin = tab_id.to_string();

    let m_wrinkle = model.clone();
    let tid_wrinkle = tab_id.to_string();

    let m_make = model.clone();
    let tid_make = tab_id.to_string();

    let m_adj_v = model.clone();
    let tid_adj_v = tab_id.to_string();

    let m_adj_s = model.clone();
    let tid_adj_s = tab_id.to_string();

    div()
        .id("umii_face_panel")
        .flex()
        .flex_col()
        .gap_4()
        .p_4()
        // 1. 下巴与脸颊形态 (12种)
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
                        .child(state.t("umii.jaw_shape")),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap_1p5()
                        .children((0..12).into_iter().map(|idx| {
                            let is_sel = data.shape_jaw == idx;
                            let m = m_jaw.clone();
                            let tid = tid_jaw.clone();
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .gap_1()
                                .w(px(70.0))
                                .py_1p5()
                                .rounded_md()
                                .bg(if is_sel { rgb(0x2563EB) } else { pal.card_surface })
                                .border_1()
                                .border_color(if is_sel { rgb(0x60A5FA) } else { pal.card_border })
                                .cursor_pointer()
                                .hover(|s| s.opacity(0.85))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    m.update(cx, |st, cx| {
                                        if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                            if let Some(ref mut u) = t.umii_state {
                                                u.data.shape_jaw = idx;
                                                t.is_modified = true;
                                                let dirs = collect_model_search_dirs(u.target_platform, st.project_path.as_deref());
                                                let _ = u.synthesize_model(&dirs);
                                            }
                                        }
                                        cx.notify();
                                    });
                                })
                                .child(div().text_sm().child(get_face_contour_glyph(idx)))
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                        .text_color(if is_sel { rgb(0xFFFFFF) } else { pal.text_primary })
                                        .child(format!("#{}", idx + 1)),
                                )
                        })),
                ),
        )
        // 2. 经典 11 种自然肤色
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
                        .child(state.t("umii.skin_color")),
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
                        .children(SKIN_TONES.iter().enumerate().map(|(idx, &(hex, _))| {
                            let is_sel = data.shape_skin_color as usize == idx;
                            let m = m_skin.clone();
                            let tid = tid_skin.clone();
                            div()
                                .id(ElementId::NamedInteger("skin_tone".into(), idx as u64))
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
                                                u.data.shape_skin_color = idx as i32;
                                                t.is_modified = true;
                                            }
                                        }
                                        cx.notify();
                                    });
                                })
                        })),
                ),
        )
        // 3. 皱纹与脸部年龄特征 (12种)
        .child(render_feature_grid(
            state.t("umii.wrinkles"),
            data.shape_wrinkle,
            m_wrinkle,
            tid_wrinkle,
            pal,
            |u, val| u.data.shape_wrinkle = val,
        ))
        // 4. 妆容与雀斑特征 (12种)
        .child(render_feature_grid(
            state.t("umii.makeup"),
            data.shape_make,
            m_make,
            tid_make,
            pal,
            |u, val| u.data.shape_make = val,
        ))
        // 5. 脸部特征微调罗盘
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .p_3()
                .bg(pal.card_surface)
                .border_1()
                .border_color(pal.card_border)
                .rounded_lg()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.text_secondary)
                        .child(format!("🎛️ {}", state.t("umii.dpad_title"))),
                )
                .child(render_fine_adjuster(
                    &state.t("umii.vertical_pos"),
                    format!("{:.0}", data.shape_trans_v),
                    "▲",
                    "▼",
                    {
                        let m = m_adj_v.clone();
                        let tid = tid_adj_v.clone();
                        move |_, _, cx| {
                            m.update(cx, |st, cx| {
                                if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                    if let Some(ref mut u) = t.umii_state {
                                        u.data.shape_trans_v = (u.data.shape_trans_v - 1.0).max(0.0);
                                        t.is_modified = true;
                                    }
                                }
                                cx.notify();
                            });
                        }
                    },
                    {
                        let m = m_adj_v;
                        let tid = tid_adj_v;
                        move |_, _, cx| {
                            m.update(cx, |st, cx| {
                                if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                    if let Some(ref mut u) = t.umii_state {
                                        u.data.shape_trans_v = (u.data.shape_trans_v + 1.0).min(18.0);
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
                    &state.t("umii.scale_size"),
                    format!("{:.0}", data.shape_scale),
                    "➖",
                    "➕",
                    {
                        let m = m_adj_s.clone();
                        let tid = tid_adj_s.clone();
                        move |_, _, cx| {
                            m.update(cx, |st, cx| {
                                if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                    if let Some(ref mut u) = t.umii_state {
                                        u.data.shape_scale = (u.data.shape_scale - 1.0).max(0.0);
                                        t.is_modified = true;
                                    }
                                }
                                cx.notify();
                            });
                        }
                    },
                    {
                        let m = m_adj_s;
                        let tid = tid_adj_s;
                        move |_, _, cx| {
                            m.update(cx, |st, cx| {
                                if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                    if let Some(ref mut u) = t.umii_state {
                                        u.data.shape_scale = (u.data.shape_scale + 1.0).min(8.0);
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

fn render_feature_grid<F>(
    title: &str,
    curr_val: i32,
    model: Entity<AppState>,
    tab_id: String,
    pal: &ThemePalette,
    on_set: F,
) -> impl IntoElement
where
    F: Fn(&mut crate::model::umii::UmiiDocumentState, i32) + Copy + 'static,
{
    div()
        .flex()
        .flex_col()
        .gap_1p5()
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(pal.text_secondary)
                .child(title.to_string()),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_1p5()
                .children((0..12).into_iter().map(move |val| {
                    let is_sel = curr_val == val;
                    let m = model.clone();
                    let tid = tab_id.clone();
                    div()
                        .flex_1()
                        .min_w(px(46.0))
                        .py_1p5()
                        .rounded_md()
                        .bg(if is_sel { rgb(0x059669) } else { pal.card_surface })
                        .border_1()
                        .border_color(if is_sel { rgb(0x34D399) } else { pal.card_border })
                        .text_center()
                        .cursor_pointer()
                        .hover(|s| s.opacity(0.85))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            m.update(cx, |st, cx| {
                                if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                    if let Some(ref mut u) = t.umii_state {
                                        on_set(u, val);
                                        t.is_modified = true;
                                    }
                                }
                                cx.notify();
                            });
                        })
                        .child(
                            div()
                                .text_xs()
                                .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                .text_color(if is_sel { rgb(0xFFFFFF) } else { pal.text_primary })
                                .child(if val == 0 { "无".to_string() } else { format!("#{}", val) }),
                        )
                })),
        )
}

fn get_face_contour_glyph(idx: i32) -> &'static str {
    match idx {
        0 => "🥚", // 椭圆
        1 => "🌰", // 尖下巴
        2 => "🟧", // 方脸
        3 => "🔻", // 倒三角
        4 => "⚪", // 圆脸
        5 => "💎", // 菱形
        6 => "🍐", // 梨形
        7 => "🛡️", // 宽下巴
        8 => "📐", // 棱角
        9 => "🍎", // 苹果肌
        10 => "🗿", // 紧致
        11 => "🍉", // 宽阔
        _ => "👤",
    }
}
