use gpui::*;
use crate::model::quest::{QuestIndicator, QuestType};
use crate::ui::state::AppState;
use crate::ui::views::quest_map::types::world_to_screen;

/// 任务标点在地图上的屏幕投影实体
#[derive(Debug, Clone)]
pub struct RenderedPinItem {
    pub quest_name: String,
    pub quest_title: String,
    pub quest_type: QuestType,
    pub step_index: usize,
    pub step_name: String,
    pub indicator_index: usize,
    pub indicator: QuestIndicator,
    pub screen_pos: (f32, f32),
    pub is_selected: bool,
    pub is_hovered: bool,
}

/// 提取并计算当前需要渲染的所有标点
pub fn collect_rendered_pins(
    state: &AppState,
    center: (f32, f32),
    pan: (f32, f32),
    zoom: f32,
) -> Vec<RenderedPinItem> {
    let mut list = Vec::new();
    let sel_id = state.quests_selected_id.as_deref();
    let show_all = state.quest_map_show_all_pins;

    for quest in &state.quests_list {
        let is_quest_selected = sel_id == Some(&quest.name);
        if !show_all && !is_quest_selected {
            continue;
        }

        let q_title = quest.display_title.as_deref().unwrap_or(&quest.name).to_string();

        for (s_idx, step) in quest.steps.iter().enumerate() {
            for (i_idx, ind) in step.indicators.iter().enumerate() {
                let (sx, sy) = world_to_screen(ind.position.0, ind.position.2, center, pan, zoom);

                let is_hovered = state.quest_map_hovered_pin.as_ref()
                    .map(|(qn, s, i)| qn == &quest.name && *s == s_idx && *i == i_idx)
                    .unwrap_or(false);

                let is_pin_selected = state.quest_map_selected_pin.as_ref()
                    .map(|(qn, s, i)| qn == &quest.name && *s == s_idx && *i == i_idx)
                    .unwrap_or(false) || is_quest_selected;

                list.push(RenderedPinItem {
                    quest_name: quest.name.clone(),
                    quest_title: q_title.clone(),
                    quest_type: quest.quest_type,
                    step_index: s_idx,
                    step_name: step.name.clone(),
                    indicator_index: i_idx,
                    indicator: ind.clone(),
                    screen_pos: (sx, sy),
                    is_selected: is_pin_selected,
                    is_hovered,
                });
            }
        }
    }

    list
}

/// 任务推进轨迹线段：(起点世界坐标 (X, Z), 终点世界坐标 (X, Z))
pub type TrajectorySegment = ((f32, f32), (f32, f32));

/// 提取任务标点之间的阶段行进轨迹世界坐标线段
pub fn extract_quest_trajectories(state: &AppState) -> Vec<TrajectorySegment> {
    let mut segments = Vec::new();
    let sel_id = state.quests_selected_id.as_deref();
    let show_all = state.quest_map_show_all_pins;

    for quest in &state.quests_list {
        let is_quest_selected = sel_id == Some(&quest.name);
        if !show_all && !is_quest_selected {
            continue;
        }

        let mut prev_pos: Option<(f32, f32)> = None;
        for step in &quest.steps {
            for ind in &step.indicators {
                let curr_pos = (ind.position.0, ind.position.2);
                if let Some(prev) = prev_pos {
                    segments.push((prev, curr_pos));
                }
                prev_pos = Some(curr_pos);
            }
        }
    }

    segments
}

/// 绘制标点之间的任务阶段推进行进虚线轨迹 (Step 1 -> Step 2 -> ...)
pub fn draw_quest_trajectories_coords(
    segments: &[TrajectorySegment],
    center: (f32, f32),
    pan: (f32, f32),
    zoom: f32,
    window: &mut Window,
) {
    if segments.is_empty() {
        return;
    }

    let mut line_builder = PathBuilder::fill();
    for &((x1, z1), (x2, z2)) in segments {
        let (sx1, sy1) = world_to_screen(x1, z1, center, pan, zoom);
        let (sx2, sy2) = world_to_screen(x2, z2, center, pan, zoom);

        // 绘制连线粗带
        let dx = sx2 - sx1;
        let dy = sy2 - sy1;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist > 4.0 {
            let nx = -dy / dist * 1.5;
            let ny = dx / dist * 1.5;

            line_builder.move_to(point(px(sx1 + nx), px(sy1 + ny)));
            line_builder.line_to(point(px(sx2 + nx), px(sy2 + ny)));
            line_builder.line_to(point(px(sx2 - nx), px(sy2 - ny)));
            line_builder.line_to(point(px(sx1 - nx), px(sy1 - ny)));
            line_builder.close();
        }
    }

    if let Ok(p) = line_builder.build() {
        window.paint_path(p, rgba(0x06b6d499)); // 希卡青色高亮轨迹
    }
}

/// 渲染浮动在地图上的任务指示标点 (Pins)
pub fn render_pins_elements(
    pins: &[RenderedPinItem],
    model: &Entity<AppState>,
) -> impl IntoElement {
    let model_click = model.clone();
    let model_hover = model.clone();

    div()
        .absolute()
        .inset_0()
        .children(pins.iter().map(|pin| {
            let (sx, sy) = pin.screen_pos;
            let q_name = pin.quest_name.clone();
            let s_idx = pin.step_index;
            let i_idx = pin.indicator_index;

            let m_c = model_click.clone();
            let m_h = model_hover.clone();
            let qn_click = q_name.clone();
            let qn_hover = q_name.clone();

            let (pin_bg, pin_border, pin_fg) = match pin.quest_type {
                QuestType::Main => (rgba(0xf59e0bcc), rgb(0xfbbf24), rgb(0x000000)),
                QuestType::Sub => (rgba(0x06b6d4cc), rgb(0x38bdf8), rgb(0x000000)),
                QuestType::Mini => (rgba(0x10b981cc), rgb(0x34d399), rgb(0x000000)),
            };

            let final_border = if pin.is_selected {
                rgb(0xffffff)
            } else if pin.is_hovered {
                rgb(0xe0f2fe)
            } else {
                pin_border
            };

            let pin_size = if pin.is_selected { px(30.0) } else { px(26.0) };

            let label_text = if let Some(ref actor) = pin.indicator.actor_name {
                actor.clone()
            } else {
                format!("Step {}", pin.step_index + 1)
            };

            let anchor_w = 200.0;

            div()
                .id(SharedString::from(format!("map_pin_{}_{}_{}", q_name, s_idx, i_idx)))
                .absolute()
                .left(px(sx - anchor_w / 2.0))
                .top(px(sy - 28.0))
                .w(px(anchor_w))
                .cursor_pointer()
                .flex()
                .flex_col()
                .items_center()
                .hover(|s| s.opacity(0.85))
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    let qn = qn_click.clone();
                    m_c.update(cx, |st, cx| {
                        st.quests_selected_id = Some(qn.clone());
                        st.quest_map_selected_pin = Some((qn, s_idx, i_idx));
                        cx.notify();
                    });
                })
                .on_hover(move |hovered, _, cx| {
                    let qn = qn_hover.clone();
                    m_h.update(cx, |st, cx| {
                        st.quest_map_hovered_pin = if *hovered {
                            Some((qn, s_idx, i_idx))
                        } else {
                            None
                        };
                        cx.notify();
                    });
                })
                // 标点图标胶囊
                .child(
                    div()
                        .size(pin_size)
                        .rounded_full()
                        .bg(pin_bg)
                        .border_2()
                        .border_color(final_border)
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(if pin.is_selected { px(12.0) } else { px(11.0) })
                        .font_weight(FontWeight::BOLD)
                        .text_color(pin_fg)
                        .shadow_md()
                        .child(format!("{}", pin.step_index + 1)),
                )
                // 标点指针小脚标
                .child(
                    div()
                        .size(px(6.0))
                        .rounded_full()
                        .bg(final_border)
                        .mt_0p5(),
                )
                // 标点 Actor/阶段名微标签
                .child(
                    div()
                        .mt_0p5()
                        .px_1p5()
                        .py_0p5()
                        .rounded_sm()
                        .bg(rgba(0x0f172aee))
                        .border_1()
                        .border_color(pin_border.opacity(0.6))
                        .text_size(px(9.5))
                        .text_color(rgb(0xffffff))
                        .whitespace_nowrap()
                        .child(label_text),
                )
        }))
}
