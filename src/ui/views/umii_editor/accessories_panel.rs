//! UMii 眼镜、头饰、行囊背包与克洛格配件控制面板

use gpui::*;
use crate::model::umii::parts::collect_model_search_dirs;
use crate::ui::state::AppState;
use crate::ui::theme::ThemePalette;

pub const GLASSES_COLORS: [(u32, &str); 6] = [
    (0x2A2A2A, "黑灰"), (0x5E3A1C, "深棕"), (0x991B1B, "酒红"),
    (0x1D4ED8, "宝石蓝"), (0xD97706, "琥珀金"), (0xF3F4F6, "银白"),
];

pub fn render_accessories_panel(
    state: &AppState,
    model: &Entity<AppState>,
    pal: &ThemePalette,
    tab_id: &str,
) -> impl IntoElement {
    let tab = state.open_tabs.iter().find(|t| t.id == tab_id);
    let Some(tab) = tab else { return div().id("accessories_panel_empty"); };
    let Some(umii_state) = &tab.umii_state else { return div().id("accessories_panel_empty"); };
    let data = &umii_state.data;

    let m_glass = model.clone();
    let tid_glass = tab_id.to_string();

    let m_color = model.clone();
    let tid_color = tab_id.to_string();

    let m_hat = model.clone();
    let tid_hat = tab_id.to_string();

    let m_bag = model.clone();
    let tid_bag = tab_id.to_string();

    let m_mask = model.clone();
    let tid_mask = tab_id.to_string();

    div()
        .id("umii_accessories_panel")
        .flex()
        .flex_col()
        .gap_4()
        .p_4()
        // 1. 眼镜款式 (9种: 无 + 8种经典框镜)
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
                        .child(format!("{} (Glasses)", state.t("umii.glass_type"))),
                )
                .child(
                    div()
                        .grid()
                        .grid_cols(3)
                        .gap_1p5()
                        .children([
                            (0, "无眼镜", "🚫"),
                            (1, "椭圆细框", "👓"),
                            (2, "长方形框", "🕶️"),
                            (3, "圆框复古", "🥽"),
                            (4, "半框商务", "👓"),
                            (5, "复古大框", "🕶️"),
                            (6, "猫眼时尚", "👓"),
                            (7, "遮阳墨镜", "🕶️"),
                            (8, "护目风镜", "🥽"),
                        ].into_iter().map(|(val, label, icon)| {
                            let is_sel = data.glass_type == val;
                            let m = m_glass.clone();
                            let tid = tid_glass.clone();
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
                                                u.data.glass_type = val;
                                                t.is_modified = true;
                                                let dirs = collect_model_search_dirs(u.target_platform, st.project_path.as_deref());
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
                                        .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                        .text_color(if is_sel { rgb(0xFFFFFF) } else { pal.text_primary })
                                        .child(label),
                                )
                        })),
                ),
        )
        // 2. 镜架颜色调色盘 (6色)
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
                        .child(state.t("umii.glass_color")),
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
                        .children(GLASSES_COLORS.iter().enumerate().map(|(idx, &(hex, _))| {
                            let is_sel = data.glass_color as usize == idx;
                            let m = m_color.clone();
                            let tid = tid_color.clone();
                            div()
                                .id(ElementId::NamedInteger("glass_color".into(), idx as u64))
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
                                                u.data.glass_color = idx as i32;
                                                t.is_modified = true;
                                            }
                                        }
                                        cx.notify();
                                    });
                                })
                        })),
                ),
        )
        // 3. BotW 专属配件 (帽子与背包)
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
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(pal.text_primary)
                                .child("头饰 / 遮阳帽 (Hat)"),
                        )
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .rounded_md()
                                .bg(if data.common_hat > 0 { rgb(0x2563EB) } else { pal.panel_surface })
                                .border_1()
                                .border_color(if data.common_hat > 0 { rgb(0x60A5FA) } else { pal.card_border })
                                .cursor_pointer()
                                .hover(|s| s.opacity(0.85))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    let tid = tid_hat.clone();
                                    m_hat.update(cx, |st, cx| {
                                        if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                            if let Some(ref mut u) = t.umii_state {
                                                u.data.common_hat = if u.data.common_hat > 0 { 0 } else { 1 };
                                                t.is_modified = true;
                                                let dirs = collect_model_search_dirs(u.target_platform, st.project_path.as_deref());
                                                let _ = u.synthesize_model(&dirs);
                                            }
                                        }
                                        cx.notify();
                                    });
                                })
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(if data.common_hat > 0 { rgb(0xFFFFFF) } else { pal.text_secondary })
                                        .child(if data.common_hat > 0 { "已佩戴帽子 🧢" } else { "无帽子 ❌" }),
                                ),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(pal.text_primary)
                                .child("行囊背包 (Backpack)"),
                        )
                        .child(
                            div()
                                .px_3()
                                .py_1()
                                .rounded_md()
                                .bg(if data.common_backpack > 0 { rgb(0x059669) } else { pal.panel_surface })
                                .border_1()
                                .border_color(if data.common_backpack > 0 { rgb(0x34D399) } else { pal.card_border })
                                .cursor_pointer()
                                .hover(|s| s.opacity(0.85))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    let tid = tid_bag.clone();
                                    m_bag.update(cx, |st, cx| {
                                        if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                            if let Some(ref mut u) = t.umii_state {
                                                u.data.common_backpack = if u.data.common_backpack > 0 { 0 } else { 1 };
                                                t.is_modified = true;
                                                let dirs = collect_model_search_dirs(u.target_platform, st.project_path.as_deref());
                                                let _ = u.synthesize_model(&dirs);
                                            }
                                        }
                                        cx.notify();
                                    });
                                })
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(if data.common_backpack > 0 { rgb(0xFFFFFF) } else { pal.text_secondary })
                                        .child(if data.common_backpack > 0 { "佩戴旅行背包 🎒" } else { "无背包 ❌" }),
                                ),
                        ),
                ),
        )
        // 4. 克洛格专属面具 (当种族为克洛格时展示)
        .children((data.body_race == 1).then(|| {
            div()
                .flex()
                .flex_col()
                .gap_1p5()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0x34D399))
                        .child("克洛格树叶面具 (Korok Leaf Mask)"),
                )
                .child(
                    div()
                        .grid()
                        .grid_cols(4)
                        .gap_1p5()
                        .children((0..4).map(|val| {
                            let is_sel = data.korog_mask == val;
                            let m = m_mask.clone();
                            let tid = tid_mask.clone();
                            div()
                                .py_2()
                                .rounded_md()
                                .bg(if is_sel { rgb(0x065F46) } else { pal.card_surface })
                                .border_1()
                                .border_color(if is_sel { rgb(0x10B981) } else { pal.card_border })
                                .text_center()
                                .cursor_pointer()
                                .hover(|s| s.opacity(0.85))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    m.update(cx, |st, cx| {
                                        if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                            if let Some(ref mut u) = t.umii_state {
                                                u.data.korog_mask = val;
                                                t.is_modified = true;
                                                let dirs = collect_model_search_dirs(u.target_platform, st.project_path.as_deref());
                                                let _ = u.synthesize_model(&dirs);
                                            }
                                        }
                                        cx.notify();
                                    });
                                })
                                .child(div().text_sm().child("🍃"))
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                        .text_color(if is_sel { rgb(0xFFFFFF) } else { pal.text_primary })
                                        .child(format!("面具 #{}", val)),
                                )
                        })),
                )
        }))
}
