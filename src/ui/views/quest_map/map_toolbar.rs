use gpui::*;
use crate::ui::state::AppState;
use crate::ui::views::quest_map::map_asset_indicator::render_asset_source_indicator;
use crate::ui::views::quest_map::map_layer_filters::render_map_layer_filters;

/// 渲染 2D 任务地图顶部文档栏 (展示 Switch/WiiU 平台与 MapTex 来源物理路径，去除流程图加号按钮)
pub fn render_quest_map_top_bar(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    div()
        .id("quest_map_top_bar")
        .h(px(32.0))
        .w_full()
        .bg(pal.panel_surface)
        .border_b_1()
        .border_color(pal.panel_border)
        .flex()
        .items_center()
        .justify_between()
        .px_3()
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(render_asset_source_indicator(state, model)),
        )
}

/// 渲染 2D 地图顶部 HUD 工具栏 (任务快速切换、阶段直达、视口与图层控制)
pub fn render_map_header_hud(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();
    let selected_quest = state.get_selected_quest();
    let model_grid = model.clone();
    let show_grid = state.quest_map_show_grid;
    let is_extracting = state.quest_map_is_extracting_tiles;
    let model_extract = model.clone();

    div()
        .h(px(42.0))
        .w_full()
        .px_3()
        .bg(pal.panel_surface.opacity(0.92))
        .border_b_1()
        .border_color(pal.panel_border)
        .flex()
        .items_center()
        .justify_between()
        // 左侧：选中任务信息与阶段直达 Chips (未选中时保持简洁空白)
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .children(selected_quest.map(|q| {
                    let q_title = q.display_title.as_deref().unwrap_or(&q.name);
                    let (badge_bg, badge_fg) = match q.quest_type {
                        crate::model::quest::QuestType::Main => (pal.secondary_accent.opacity(0.2), pal.secondary_accent),
                        crate::model::quest::QuestType::Sub => (pal.primary_accent.opacity(0.2), pal.primary_accent),
                        crate::model::quest::QuestType::Mini => (pal.primary_accent_hover.opacity(0.2), pal.primary_accent_hover),
                    };

                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1p5()
                                .child(
                                    div()
                                        .text_size(px(10.5))
                                        .px_1p5()
                                        .py_0p5()
                                        .rounded_sm()
                                        .bg(badge_bg)
                                        .text_color(badge_fg)
                                        .child(state.t(q.quest_type.label_key())),
                                )
                                .child(
                                    div()
                                        .text_size(px(13.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(pal.text_primary)
                                        .child(q_title.to_string()),
                                ),
                        )
                        // 阶段标点快速直达
                        .children(q.steps.iter().enumerate().filter(|(_, s)| !s.indicators.is_empty()).map(|(s_idx, s)| {
                            let m_step = model.clone();
                            let pos = s.indicators[0].position;
                            div()
                                .cursor_pointer()
                                .px_2()
                                .py_0p5()
                                .rounded_sm()
                                .bg(pal.card_hover)
                                .text_size(px(10.5))
                                .text_color(pal.text_secondary)
                                .hover(|s| s.bg(pal.primary_accent.opacity(0.25)).text_color(pal.text_primary))
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    m_step.update(cx, |st, cx| {
                                        st.focus_pin_on_map(pos);
                                        cx.notify();
                                    });
                                })
                                .child(format!("📍 Step {}: {}", s_idx + 1, s.name))
                        }))
                }))
        )
        // 右侧：图层开关与实景底图提取控制
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                // 地图标注分类图层过滤器 (神庙/驿站/神兽/塔/城镇)
                .child(render_map_layer_filters(state, model))
                // 底图模式切换按钮：纯代码矢量极速 / 游戏卫星实景瓦片
                .child({
                    let model_mode = model.clone();
                    let is_vector = state.quest_map_use_vector_mode;
                    div()
                        .id("btn_toggle_map_mode")
                        .cursor_pointer()
                        .px_2()
                        .py_1()
                        .rounded_sm()
                        .bg(if is_vector { pal.primary_accent.opacity(0.2) } else { pal.secondary_accent.opacity(0.2) })
                        .border_1()
                        .border_color(if is_vector { pal.primary_accent } else { pal.secondary_accent })
                        .text_size(px(11.0))
                        .text_color(if is_vector { pal.primary_accent } else { pal.secondary_accent })
                        .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_mode.update(cx, |st, cx| {
                                st.toggle_quest_map_vector_mode();
                                cx.notify();
                            });
                        })
                        .child(if is_vector {
                            state.t("quests.map_mode_vector")
                        } else {
                            state.t("quests.map_mode_satellite")
                        })
                })
                // 网格开关
                .child(
                    div()
                        .id("btn_toggle_map_grid")
                        .cursor_pointer()
                        .px_2()
                        .py_1()
                        .rounded_sm()
                        .bg(if show_grid { pal.primary_accent.opacity(0.2) } else { pal.card_hover })
                        .border_1()
                        .border_color(if show_grid { pal.primary_accent } else { pal.panel_border })
                        .text_size(px(11.0))
                        .text_color(if show_grid { pal.primary_accent } else { pal.text_secondary })
                        .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            model_grid.update(cx, |st, cx| {
                                st.quest_map_show_grid = !st.quest_map_show_grid;
                                cx.notify();
                            });
                        })
                        .child(state.t("quests.map_toggle_grid")),
                )
                // 提取瓦片按钮 (优先从打开的项目提取)
                .child(
                    div()
                        .id("btn_extract_map_tiles")
                        .cursor_pointer()
                        .px_2()
                        .py_1()
                        .rounded_sm()
                        .bg(if is_extracting { pal.secondary_accent.opacity(0.3) } else { pal.card_hover })
                        .border_1()
                        .border_color(if is_extracting { pal.secondary_accent } else { pal.panel_border })
                        .text_size(px(11.0))
                        .text_color(if is_extracting { pal.secondary_accent } else { pal.text_primary })
                        .hover(|s| s.bg(pal.primary_accent.opacity(0.25)))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            if !is_extracting {
                                let m = model_extract.clone();
                                model_extract.update(cx, |st, cx| {
                                    st.trigger_extract_map_tiles(&m, cx);
                                });
                            }
                        })
                        .child(if is_extracting {
                            state.t("quests.extracting_tiles")
                        } else {
                            state.t("quests.extract_tiles")
                        }),
                ),
        )
}
