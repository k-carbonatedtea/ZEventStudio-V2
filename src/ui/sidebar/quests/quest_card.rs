use gpui::*;
use crate::model::quest::QuestDefinition;
use crate::ui::state::AppState;

/// 渲染任务卡片组件 (支持折叠展开阶段详情)
pub fn render_quest_card(
    quest: &QuestDefinition,
    is_selected: bool,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let q_name = quest.name.clone();
    let q_name_click = quest.name.clone();
    let model_click = model.clone();

    let display_title = quest.display_title.as_deref().unwrap_or(&q_name);
    let type_label = state.t(quest.quest_type.label_key());

    let (badge_bg, badge_fg) = match quest.quest_type {
        crate::model::quest::QuestType::Main => (pal.secondary_accent.opacity(0.18), pal.secondary_accent),
        crate::model::quest::QuestType::Sub => (pal.primary_accent.opacity(0.18), pal.primary_accent),
        crate::model::quest::QuestType::Mini => (pal.primary_accent_hover.opacity(0.18), pal.primary_accent_hover),
    };

    div()
        .id(SharedString::from(format!("quest_card_{}", q_name)))
        .flex()
        .flex_col()
        .w_full()
        .mb_1p5()
        .rounded_md()
        .border_1()
        .border_color(if is_selected { pal.primary_accent } else { pal.panel_border })
        .bg(if is_selected { pal.card_hover } else { pal.panel_surface })
        // 头部点击展开/折叠
        .child(
            div()
                .flex()
                .flex_col()
                .p_2()
                .cursor_pointer()
                .hover(|s| s.bg(pal.card_hover))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    let name = q_name_click.clone();
                    model_click.update(cx, |st, cx| {
                        st.toggle_quest_selection(&name);
                        cx.notify();
                    });
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .w_full()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1p5()
                                .child(
                                    div()
                                        .text_size(px(10.0))
                                        .px_1p5()
                                        .py_0p5()
                                        .rounded_sm()
                                        .bg(badge_bg)
                                        .text_color(badge_fg)
                                        .child(type_label),
                                )
                                .children((quest.total_indicators_count() > 0).then(|| {
                                    div()
                                        .text_size(px(10.0))
                                        .px_1p5()
                                        .py_0p5()
                                        .rounded_sm()
                                        .bg(pal.secondary_accent.opacity(0.18))
                                        .text_color(pal.secondary_accent)
                                        .child(format!("{} {}", state.t("quests.badge_indicator"), quest.total_indicators_count()))
                                }))
                                .child(
                                    div()
                                        .text_size(px(12.5))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(pal.text_primary)
                                        .child(display_title.to_string()),
                                ),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(pal.text_secondary)
                                .child(if is_selected { "▼" } else { "▶" }),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .mt_1()
                        .text_size(px(11.0))
                        .text_color(pal.text_secondary)
                        .child(format!("ID: {}", q_name))
                        .children((!quest.orderer.is_empty()).then(|| {
                            div().child(format!("• {}", quest.orderer))
                        }))
                        .children((!quest.location.is_empty()).then(|| {
                            div().child(format!("• {}", quest.location))
                        })),
                ),
        )
        // 展开后的详情视图
        .children(is_selected.then(|| {
            render_quest_details(quest, state, model)
        }))
}

/// 渲染展开后的任务详情 (描述文本、步骤列表、标志位提示)
fn render_quest_details(
    quest: &QuestDefinition,
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let q_clone = quest.clone();
    let model_scaffold = model.clone();

    div()
        .flex()
        .flex_col()
        .px_2()
        .pb_2()
        .pt_1()
        .border_t_1()
        .border_color(pal.panel_border.opacity(0.5))
        .gap_2()
        // 冒险笔记文本预览
        .children(quest.desc_text.as_ref().map(|desc| {
            div()
                .flex()
                .flex_col()
                .gap_0p5()
                .p_1p5()
                .rounded_sm()
                .bg(pal.panel_surface)
                .border_1()
                .border_color(pal.panel_border.opacity(0.5))
                .child(
                    div()
                        .text_size(px(10.5))
                        .text_color(pal.text_secondary)
                        .child(state.t("quests.journal_desc")),
                )
                .child(
                    div()
                        .text_size(px(11.5))
                        .text_color(pal.text_primary)
                        .child(desc.clone()),
                )
        }))
        // 阶段列表 (Steps)
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .text_size(px(10.5))
                        .text_color(pal.text_secondary)
                        .child(format!("{} ({})", state.t("quests.steps_header"), quest.steps.len())),
                )
                .children(quest.steps.iter().enumerate().map(|(idx, step)| {
                    let has_indicators = !step.indicators.is_empty();
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .p_1()
                        .rounded_sm()
                        .bg(pal.panel_surface)
                        .text_size(px(11.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .w_full()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_1p5()
                                        .child(
                                            div()
                                                .size(px(14.0))
                                                .rounded_full()
                                                .bg(pal.primary_accent.opacity(0.15))
                                                .text_color(pal.primary_accent)
                                                .text_size(px(9.0))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .child(format!("{}", idx + 1)),
                                        )
                                        .child(div().text_color(pal.text_primary).child(step.name.clone()))
                                        .children(has_indicators.then(|| {
                                            div()
                                                .text_size(px(9.5))
                                                .px_1()
                                                .rounded_sm()
                                                .bg(pal.secondary_accent.opacity(0.15))
                                                .text_color(pal.secondary_accent)
                                                .child(format!("📍 {}", step.indicators.len()))
                                        })),
                                )
                                .child(
                                    div()
                                        .text_size(px(10.0))
                                        .text_color(pal.text_secondary)
                                        .child(step.next_flag.clone()),
                                ),
                        )
                        .children(has_indicators.then(|| {
                            render_step_indicators(step, state)
                        }))
                })),
        )
        // 底部动作按钮：在当前流程图中生成脚手架拓扑
        .child(
            div()
                .flex()
                .items_center()
                .justify_end()
                .pt_1()
                .child(
                    div()
                        .id(SharedString::from(format!("scaffold_btn_{}", quest.name)))
                        .flex()
                        .items_center()
                        .gap_1()
                        .px_2()
                        .py_1()
                        .rounded_sm()
                        .bg(pal.primary_accent.opacity(0.12))
                        .text_color(pal.primary_accent)
                        .text_size(px(11.0))
                        .cursor_pointer()
                        .hover(|s| s.bg(pal.primary_accent.opacity(0.25)))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            let q_cfg = crate::model::quest::QuestScaffoldConfig {
                                name: q_clone.name.clone(),
                                quest_type: q_clone.quest_type,
                                orderer: q_clone.orderer.clone(),
                                location: q_clone.location.clone(),
                                display_title: q_clone.display_title.clone().unwrap_or_default(),
                                desc_text: q_clone.desc_text.clone().unwrap_or_default(),
                                finish_text: q_clone.finish_text.clone().unwrap_or_default(),
                                inject_titlebg: false,
                                inject_bootup_flags: false,
                                inject_ql_msbt: false,
                                scaffold_graph_nodes: true,
                                inject_npc_choice: false,
                            };
                            model_scaffold.update(cx, |st, cx| {
                                let count = crate::model::quest::scaffold_quest_graph_nodes(&mut st.graph, &q_cfg).len();
                                st.mark_current_tab_modified();
                                st.fit_view();
                                st.toast = Some(crate::ui::components::Toast::success(
                                    format!("已向当前画布插入 {} 个任务节点拓扑", count)
                                ));
                                cx.notify();
                            });
                        })
                        .child("⚡")
                        .child(state.t("quests.insert_to_canvas")),
                ),
        )
}

/// 渲染阶段包含的地图标点目标列表
fn render_step_indicators(step: &crate::model::quest::QuestStep, state: &AppState) -> impl IntoElement {
    let pal = state.theme();
    div()
        .flex()
        .flex_col()
        .gap_1()
        .pt_1()
        .pl_2()
        .border_t_1()
        .border_color(pal.panel_border.opacity(0.3))
        .children(step.indicators.iter().map(|ind| {
            render_single_indicator(ind, state)
        }))
}

/// 渲染单个地图标点条目 (包含 Actor 标识、三维世界坐标与消隐标志位)
fn render_single_indicator(ind: &crate::model::quest::QuestIndicator, state: &AppState) -> impl IntoElement {
    let pal = state.theme();
    let label = if let Some(ref actor) = ind.actor_name {
        if let Some(ref inst) = ind.instance_name {
            format!("📍 {} [{}]", actor, inst)
        } else {
            format!("📍 {}", actor)
        }
    } else {
        format!("📍 {}", state.t("quests.target_coords"))
    };

    let coords_str = format!("({:.0}, {:.0}, {:.0})", ind.position.0, ind.position.1, ind.position.2);

    div()
        .flex()
        .items_center()
        .justify_between()
        .text_size(px(10.0))
        .p_0p5()
        .rounded_sm()
        .bg(pal.card_hover.opacity(0.5))
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(pal.secondary_accent)
                        .child(label),
                )
                .child(
                    div()
                        .text_color(pal.text_secondary)
                        .child(coords_str),
                ),
        )
        .children(ind.off_flag.as_ref().map(|flag| {
            div()
                .text_size(px(9.0))
                .px_1()
                .rounded_sm()
                .bg(pal.panel_border.opacity(0.3))
                .text_color(pal.text_secondary)
                .child(format!("{}: {}", state.t("quests.off_flag_label"), flag))
        }))
}
