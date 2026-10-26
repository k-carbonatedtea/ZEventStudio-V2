use gpui::*;
use crate::model::quest::QuestDefinition;
use crate::ui::state::AppState;
use crate::ui::ThemePalette;

/// 渲染可就地折叠/展开的虚拟任务卡片组件 (结合 gpui::list 仅渲染视口可视范围，未滚动区域零开销)
pub fn render_expandable_quest_card(
    quest: &QuestDefinition,
    is_selected: bool,
    lang: crate::i18n::Language,
    pal: ThemePalette,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let q_name = quest.name.clone();
    let q_name_click = quest.name.clone();
    let model_click = model.clone();

    let display_title = quest.display_title.as_deref().unwrap_or(&q_name);
    let type_label = crate::i18n::t(quest.quest_type.label_key(), lang);
    let indicator_badge = crate::i18n::t("quests.badge_indicator", lang);

    let (badge_bg, badge_fg) = match quest.quest_type {
        crate::model::quest::QuestType::Main => (pal.secondary_accent.opacity(0.18), pal.secondary_accent),
        crate::model::quest::QuestType::Sub => (pal.primary_accent.opacity(0.18), pal.primary_accent),
        crate::model::quest::QuestType::Mini => (pal.primary_accent_hover.opacity(0.18), pal.primary_accent_hover),
    };

    div()
        .id(SharedString::from(format!("quest_row_{}", q_name)))
        .w_full()
        .p_1()
        .child(
            div()
                .size_full()
                .rounded_md()
                .border_1()
                .border_color(if is_selected { pal.primary_accent } else { pal.panel_border })
                .bg(if is_selected { pal.card_hover } else { pal.panel_surface })
                .flex()
                .flex_col()
                // 1. 卡片头部 (点击就地折叠/展开，并联动海拉鲁地图)
                .child(
                    div()
                        .px_2p5()
                        .py_2()
                        .cursor_pointer()
                        .hover(|s| s.bg(pal.card_hover))
                        .flex()
                        .flex_col()
                        .gap_1()
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            let name = q_name_click.clone();
                            model_click.update(cx, |st, cx| {
                                st.current_tab = crate::ui::state_types::AppWorkspaceTab::Quests;
                                st.toggle_quest_selection(&name);
                                st.focus_selected_quest_on_map();
                                cx.notify();
                            });
                        })
                        // 第一行：类型、标点计数、标题与折叠展开指示
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
                                                .child(format!("{} {}", indicator_badge, quest.total_indicators_count()))
                                        }))
                                        .child(
                                            div()
                                                .text_size(px(12.5))
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(if is_selected { pal.primary_accent } else { pal.text_primary })
                                                .child(display_title.to_string()),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_size(px(11.0))
                                        .text_color(if is_selected { pal.primary_accent } else { pal.text_secondary })
                                        .child(if is_selected { "▼" } else { "▶" }),
                                ),
                        )
                        // 第二行：ID、委托人、地点元信息
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .text_size(px(10.5))
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
                // 2. 展开后的详情视图 (就地内联展开，包含冒险笔记、步骤列表与脚手架注入)
                .children(is_selected.then(|| {
                    render_quest_inline_details(quest, lang, pal, model)
                })),
        )
}

/// 渲染就地展开后的任务步骤与流程图脚手架操作区
fn render_quest_inline_details(
    quest: &QuestDefinition,
    lang: crate::i18n::Language,
    pal: ThemePalette,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let q_clone = quest.clone();
    let model_scaffold = model.clone();

    let journal_title = crate::i18n::t("quests.journal_desc", lang);
    let steps_header = crate::i18n::t("quests.steps_header", lang);
    let insert_label = crate::i18n::t("quests.insert_to_canvas", lang);

    div()
        .flex()
        .flex_col()
        .px_2p5()
        .pb_2p5()
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
                .p_2()
                .rounded_sm()
                .bg(pal.panel_surface.opacity(0.8))
                .border_1()
                .border_color(pal.panel_border.opacity(0.5))
                .child(
                    div()
                        .text_size(px(10.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.text_secondary)
                        .child(journal_title),
                )
                .child(
                    div()
                        .text_size(px(11.0))
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
                        .font_weight(FontWeight::BOLD)
                        .text_color(pal.text_secondary)
                        .child(format!("{} ({})", steps_header, quest.steps.len())),
                )
                .children(quest.steps.iter().enumerate().map(|(idx, step)| {
                    let has_indicators = !step.indicators.is_empty();
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .p_1p5()
                        .rounded_sm()
                        .bg(pal.panel_surface)
                        .border_1()
                        .border_color(pal.panel_border.opacity(0.4))
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
                                                .size(px(15.0))
                                                .rounded_full()
                                                .bg(pal.primary_accent.opacity(0.18))
                                                .text_color(pal.primary_accent)
                                                .text_size(px(9.5))
                                                .font_weight(FontWeight::BOLD)
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .child(format!("{}", idx + 1)),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(11.0))
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(pal.text_primary)
                                                .child(step.name.clone()),
                                        )
                                        .children(has_indicators.then(|| {
                                            div()
                                                .text_size(px(9.5))
                                                .px_1()
                                                .rounded_sm()
                                                .bg(pal.secondary_accent.opacity(0.18))
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
                            div()
                                .flex()
                                .flex_col()
                                .gap_0p5()
                                .pt_0p5()
                                .children(step.indicators.iter().map(|ind| {
                                    div()
                                        .text_size(px(9.5))
                                        .text_color(pal.secondary_accent)
                                        .child(format!("🧭 标点: ({:.0}, {:.0}, {:.0})", ind.position.0, ind.position.1, ind.position.2))
                                }))
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
                        .px_2p5()
                        .py_1()
                        .rounded_sm()
                        .bg(pal.primary_accent.opacity(0.15))
                        .text_color(pal.primary_accent)
                        .text_size(px(11.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .cursor_pointer()
                        .hover(|s| s.bg(pal.primary_accent.opacity(0.28)))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            let act_flag = q_clone.steps.iter().find(|s| s.name == "Ready").map(|s| s.next_flag.clone()).unwrap_or_else(|| format!("{}_Activated", q_clone.name));
                            let fin_flag = q_clone.steps.iter().find(|s| s.name == "Finish").map(|s| s.next_flag.clone()).unwrap_or_else(|| format!("{}_Finish", q_clone.name));
                            let extra_flags: Vec<String> = q_clone.steps.iter()
                                .filter(|s| s.name != "Ready" && s.name != "Finish")
                                .map(|s| s.next_flag.clone())
                                .collect();
                            let q_cfg = crate::model::quest::QuestScaffoldConfig {
                                name: q_clone.name.clone(),
                                quest_type: q_clone.quest_type,
                                orderer: q_clone.orderer.clone(),
                                location: q_clone.location.clone(),
                                display_title: q_clone.display_title.clone().unwrap_or_default(),
                                desc_text: q_clone.desc_text.clone().unwrap_or_default(),
                                finish_text: String::new(),
                                ready_flag: format!("{}_Ready", q_clone.name),
                                activated_flag: act_flag,
                                finish_flag: fin_flag,
                                dependency_flag: q_clone.dependency_flag.clone().unwrap_or_default(),
                                extra_flags,
                                enable_indicator: false,
                                indicator_actor: String::new(),
                                indicator_pos: (0.0, 0.0, 0.0),
                                indicator_off_flag: String::new(),
                                indicator_has_far: false,
                                not_post_note: q_clone.not_post_note,
                                inject_titlebg: false,
                                inject_bootup_flags: false,
                                inject_ql_msbt: false,
                                scaffold_graph_nodes: true,
                                inject_npc_choice: false,
                            };
                            model_scaffold.update(cx, |st, cx| {
                                let node_ids = crate::model::quest::scaffold_quest_graph_nodes(&mut st.graph, &q_cfg);
                                st.toast = Some(crate::ui::components::Toast::success(format!("已向当前流程图注入 {} 个节点", node_ids.len())));
                                cx.notify();
                            });
                        })
                        .child("⚡")
                        .child(insert_label),
                ),
        )
}
