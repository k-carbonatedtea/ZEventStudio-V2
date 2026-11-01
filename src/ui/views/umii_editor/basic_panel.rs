// UMii 基础信息、性别年龄、喜好色与身材体型面板

use gpui::*;
use crate::model::umii::parts::collect_model_search_dirs;
use crate::ui::state::AppState;
use crate::ui::theme::ThemePalette;
use super::components::render_stepper;

pub const FAVORITE_COLORS: [(u32, &str); 12] = [
    (0xE52521, "红"), (0xF37021, "橙"), (0xFFD800, "黄"), (0x88C222, "黄绿"),
    (0x009543, "绿"), (0x009FE3, "水蓝"), (0x00479D, "深蓝"), (0xE84393, "粉红"),
    (0x92278F, "紫"), (0x7C4E29, "棕"), (0xFFFFFF, "白"), (0x2B2B2B, "黑"),
];

pub fn render_basic_panel(
    state: &AppState,
    model: &Entity<AppState>,
    pal: &ThemePalette,
    tab_id: &str,
) -> impl IntoElement {
    let tab = state.open_tabs.iter().find(|t| t.id == tab_id);
    let Some(tab) = tab else { return div().id("basic_panel_empty"); };
    let Some(umii_state) = &tab.umii_state else { return div().id("basic_panel_empty"); };
    let data = &umii_state.data;

    let m_gender = model.clone();
    let tid_gender = tab_id.to_string();

    let m_color = model.clone();
    let tid_color = tab_id.to_string();

    let m_race = model.clone();
    let tid_race = tab_id.to_string();

    let m_type = model.clone();
    let tid_type = tab_id.to_string();

    let m_height = model.clone();
    let tid_height = tab_id.to_string();

    let m_weight = model.clone();
    let tid_weight = tab_id.to_string();

    div()
        .id("umii_basic_panel")
        .flex()
        .flex_col()
        .gap_4()
        .p_4()
        // 1. 性别与年龄层
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
                        .child(state.t("umii.sex_age")),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap_1p5()
                        .children([
                            (0, state.t("umii.sex_age_boy")),
                            (1, state.t("umii.sex_age_man")),
                            (2, state.t("umii.sex_age_old_man")),
                            (3, state.t("umii.sex_age_girl")),
                            (4, state.t("umii.sex_age_woman")),
                            (5, state.t("umii.sex_age_old_woman")),
                        ].into_iter().map(|(val, label)| {
                            let is_sel = data.personal_sex_age == val;
                            let m = m_gender.clone();
                            let tid = tid_gender.clone();
                            div()
                                .flex_1()
                                .min_w(px(95.0))
                                .py_1p5()
                                .px_2()
                                .rounded_md()
                                .bg(if is_sel { rgb(0x2563EB) } else { pal.card_surface })
                                .border_1()
                                .border_color(if is_sel { rgb(0x60A5FA) } else { pal.card_border })
                                .text_center()
                                .cursor_pointer()
                                .hover(|s| s.opacity(0.85))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    m.update(cx, |st, cx| {
                                        if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                            if let Some(ref mut u) = t.umii_state {
                                                u.data.personal_sex_age = val;
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
                                        .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                        .text_color(if is_sel { rgb(0xFFFFFF) } else { pal.text_primary })
                                        .child(label),
                                )
                        })),
                ),
        )
        // 2. 经典 12 喜好色
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
                        .child(state.t("umii.fav_color")),
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
                        .children(FAVORITE_COLORS.iter().enumerate().map(|(idx, &(hex, _))| {
                            let is_sel = data.personal_fav_color as usize == idx;
                            let m = m_color.clone();
                            let tid = tid_color.clone();
                            div()
                                .id(ElementId::NamedInteger("fav_color".into(), idx as u64))
                                .size(px(24.0))
                                .rounded_full()
                                .bg(rgb(hex))
                                .border_2()
                                .border_color(if is_sel { rgb(0x60A5FA) } else { rgba((hex << 8) | 0x33) })
                                .cursor_pointer()
                                .hover(|s| s.opacity(0.85))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    m.update(cx, |st, cx| {
                                        if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                            if let Some(ref mut u) = t.umii_state {
                                                u.data.personal_fav_color = idx as i32;
                                                t.is_modified = true;
                                            }
                                        }
                                        cx.notify();
                                    });
                                })
                        })),
                ),
        )
        // 3. 身材比例 (身高与体型步进器)
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
                .child(render_stepper(
                    state.t("umii.height_slider"),
                    data.body_height,
                    pal,
                    {
                        let m = m_height.clone();
                        let tid = tid_height.clone();
                        move |_, _, cx| {
                            m.update(cx, |st, cx| {
                                if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                    if let Some(ref mut u) = t.umii_state {
                                        u.data.body_height = (u.data.body_height - 8).clamp(0, 127);
                                        t.is_modified = true;
                                    }
                                }
                                cx.notify();
                            });
                        }
                    },
                    {
                        let m = m_height;
                        let tid = tid_height;
                        move |_, _, cx| {
                            m.update(cx, |st, cx| {
                                if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                    if let Some(ref mut u) = t.umii_state {
                                        u.data.body_height = (u.data.body_height + 8).clamp(0, 127);
                                        t.is_modified = true;
                                    }
                                }
                                cx.notify();
                            });
                        }
                    },
                ))
                .child(render_stepper(
                    state.t("umii.weight_slider"),
                    data.body_weight,
                    pal,
                    {
                        let m = m_weight.clone();
                        let tid = tid_weight.clone();
                        move |_, _, cx| {
                            m.update(cx, |st, cx| {
                                if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                    if let Some(ref mut u) = t.umii_state {
                                        u.data.body_weight = (u.data.body_weight - 8).clamp(0, 127);
                                        t.is_modified = true;
                                    }
                                }
                                cx.notify();
                            });
                        }
                    },
                    {
                        let m = m_weight;
                        let tid = tid_weight;
                        move |_, _, cx| {
                            m.update(cx, |st, cx| {
                                if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                    if let Some(ref mut u) = t.umii_state {
                                        u.data.body_weight = (u.data.body_weight + 8).clamp(0, 127);
                                        t.is_modified = true;
                                    }
                                }
                                cx.notify();
                            });
                        }
                    },
                )),
        )
        // 4. BotW 种族设定
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
                        .child(state.t("umii.race")),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap_1p5()
                        .children([
                            (0, state.t("umii.race_hylian")),
                            (1, state.t("umii.race_korok")),
                            (2, state.t("umii.race_goron")),
                            (4, state.t("umii.race_gerudo")),
                            (5, state.t("umii.race_zora")),
                            (6, state.t("umii.race_rito")),
                        ].into_iter().map(|(val, label)| {
                            let is_sel = data.body_race == val;
                            let m = m_race.clone();
                            let tid = tid_race.clone();
                            div()
                                .flex_1()
                                .min_w(px(90.0))
                                .py_1p5()
                                .px_2()
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
                                                u.data.body_race = val;
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
                                        .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                        .text_color(if is_sel { rgb(0xFFFFFF) } else { pal.text_primary })
                                        .child(label),
                                )
                        })),
                ),
        )
        // 5. 身躯形态/职业分类
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
                        .child(state.t("umii.body_type")),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap_1p5()
                        .children([
                            (0, state.t("umii.body_type_normal")),
                            (1, state.t("umii.body_type_traveler")),
                            (2, state.t("umii.body_type_warrior")),
                            (3, state.t("umii.body_type_villager")),
                        ].into_iter().map(|(val, label)| {
                            let is_sel = data.body_type == val;
                            let m = m_type.clone();
                            let tid = tid_type.clone();
                            div()
                                .flex_1()
                                .min_w(px(70.0))
                                .py_1p5()
                                .px_2()
                                .rounded_md()
                                .bg(if is_sel { rgb(0x7C3AED) } else { pal.card_surface })
                                .border_1()
                                .border_color(if is_sel { rgb(0xA78BFA) } else { pal.card_border })
                                .text_center()
                                .cursor_pointer()
                                .hover(|s| s.opacity(0.85))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    m.update(cx, |st, cx| {
                                        if let Some(t) = st.open_tabs.iter_mut().find(|t| t.id == tid) {
                                            if let Some(ref mut u) = t.umii_state {
                                                u.data.body_type = val;
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
                                        .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::NORMAL })
                                        .text_color(if is_sel { rgb(0xFFFFFF) } else { pal.text_primary })
                                        .child(label),
                                )
                        })),
                ),
        )
}
