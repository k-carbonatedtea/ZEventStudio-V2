use gpui::*;
use crate::ui::state::AppState;

/// 过滤按钮高亮主题色
#[derive(Clone, Copy)]
struct FilterColor {
    bg: Rgba,
    fg: Rgba,
}

/// 渲染 2D 地图顶部 HUD 地标图层分类过滤胶囊按钮组
pub fn render_map_layer_filters(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();

    let show_shrines = state.quest_map_show_shrines;
    let show_stables = state.quest_map_show_stables;
    let show_beasts = state.quest_map_show_beasts;
    let show_towers = state.quest_map_show_towers;
    let show_settlements = state.quest_map_show_settlements;

    let m_shrine = model.clone();
    let m_stable = model.clone();
    let m_beast = model.clone();
    let m_tower = model.clone();
    let m_town = model.clone();

    div()
        .flex()
        .items_center()
        .gap_1()
        .p_0p5()
        .rounded_md()
        .bg(pal.card_hover.opacity(0.6))
        .border_1()
        .border_color(pal.panel_border.opacity(0.8))
        // 1. 神庙开关 (136座)
        .child(render_filter_chip(
            "btn_filter_shrines",
            state.t("quests.map_filter_shrines"),
            show_shrines,
            FilterColor { bg: rgba(0x0284c733), fg: rgb(0x38bdf8) },
            pal.text_secondary,
            pal.card_hover,
            move |_, cx| {
                m_shrine.update(cx, |st, cx| {
                    st.quest_map_show_shrines = !st.quest_map_show_shrines;
                    cx.notify();
                });
            },
        ))
        // 2. 驿站开关 (15座)
        .child(render_filter_chip(
            "btn_filter_stables",
            state.t("quests.map_filter_stables"),
            show_stables,
            FilterColor { bg: rgba(0xd9770633), fg: rgb(0xfbbf24) },
            pal.text_secondary,
            pal.card_hover,
            move |_, cx| {
                m_stable.update(cx, |st, cx| {
                    st.quest_map_show_stables = !st.quest_map_show_stables;
                    cx.notify();
                });
            },
        ))
        // 3. 四神兽开关 (4座)
        .child(render_filter_chip(
            "btn_filter_beasts",
            state.t("quests.map_filter_beasts"),
            show_beasts,
            FilterColor { bg: rgba(0x0284c744), fg: rgb(0x67e8f9) },
            pal.text_secondary,
            pal.card_hover,
            move |_, cx| {
                m_beast.update(cx, |st, cx| {
                    st.quest_map_show_beasts = !st.quest_map_show_beasts;
                    cx.notify();
                });
            },
        ))
        // 4. 希卡塔开关 (15座)
        .child(render_filter_chip(
            "btn_filter_towers",
            state.t("quests.map_filter_towers"),
            show_towers,
            FilterColor { bg: rgba(0x0e749033), fg: rgb(0x22d3ee) },
            pal.text_secondary,
            pal.card_hover,
            move |_, cx| {
                m_tower.update(cx, |st, cx| {
                    st.quest_map_show_towers = !st.quest_map_show_towers;
                    cx.notify();
                });
            },
        ))
        // 5. 城镇村落与要地开关
        .child(render_filter_chip(
            "btn_filter_settlements",
            state.t("quests.map_filter_settlements"),
            show_settlements,
            FilterColor { bg: rgba(0x0f766e33), fg: rgb(0x2dd4bf) },
            pal.text_secondary,
            pal.card_hover,
            move |_, cx| {
                m_town.update(cx, |st, cx| {
                    st.quest_map_show_settlements = !st.quest_map_show_settlements;
                    cx.notify();
                });
            },
        ))
}

/// 渲染单个紧凑小胶囊过滤芯片 (参数合并以符合代码质量规范)
fn render_filter_chip(
    id_str: &'static str,
    label: &str,
    is_active: bool,
    color: FilterColor,
    inactive_fg: Rgba,
    hover_bg: Rgba,
    on_click: impl Fn(&MouseDownEvent, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id_str)
        .cursor_pointer()
        .px_1p5()
        .py_0p5()
        .rounded_sm()
        .bg(if is_active { color.bg } else { rgba(0x00000000) })
        .border_1()
        .border_color(if is_active { color.fg.opacity(0.8) } else { rgba(0x00000000) })
        .text_size(px(10.5))
        .text_color(if is_active { color.fg } else { inactive_fg })
        .hover(move |s| s.bg(hover_bg).text_color(color.fg))
        .on_mouse_down(MouseButton::Left, move |e, _, cx| {
            cx.stop_propagation();
            on_click(e, cx);
        })
        .child(label.to_string())
}
