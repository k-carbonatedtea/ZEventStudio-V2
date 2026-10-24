use std::f32::consts::TAU;
use std::time::{SystemTime, UNIX_EPOCH};

use gpui::*;
use gpui_component::StyledExt;
use crate::ui::state::AppState;

/// 渲染 2D 海拉鲁地图加载与标点实时提取遮罩层 (深色玻璃拟态风格)
pub fn render_map_loading_overlay(state: &AppState) -> impl IntoElement {
    let pal = state.theme();
    let current_file_label = if state.quest_map_extracting_file_label.is_empty() {
        state.t("quests.loading_init_status").to_string()
    } else {
        state.quest_map_extracting_file_label.clone()
    };
    let title_text = state.t("quests.loading_markers_title");
    let extracting_label = state.t("quests.loading_extracting_label");

    // 计算当前时间戳对应的旋转与脉冲相位
    let now_millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let spin_angle = (now_millis % 1200) as f32 / 1200.0 * TAU;
    let pulse_alpha = 0.6 + 0.4 * ((now_millis % 800) as f32 / 800.0 * TAU).sin().abs();

    div()
        .id("map_loading_overlay_root")
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        // 柔和暗色微遮罩，隐约透出下层地图瓦片与地貌
        .bg(gpui::rgba(0x090d14b3))
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_4()
                .px_8()
                .py_6()
                .w(px(520.0))
                .rounded_2xl()
                .bg(gpui::rgba(0x13171fe6))
                .border_1()
                .border_color(gpui::rgba(0x388bfd59))
                .shadow_2xl()
                // 1. 希卡科技风格流光加载环指示器
                .child(
                    div()
                        .size(px(56.0))
                        .child(
                            canvas(
                                |_bounds, _window, _cx| {},
                                move |bounds, _, window, _cx| {
                                    let orig_x: f32 = bounds.origin.x.into();
                                    let orig_y: f32 = bounds.origin.y.into();
                                    let center_x = orig_x + 28.0;
                                    let center_y = orig_y + 28.0;

                                    // 绘制背景暗圈
                                    let dot_count = 12;
                                    for i in 0..dot_count {
                                        let angle = spin_angle + (i as f32 / dot_count as f32) * TAU;
                                        let radius = 20.0;
                                        let pos_x = center_x + radius * angle.cos();
                                        let pos_y = center_y + radius * angle.sin();
                                        let alpha_factor = (i as f32 + 1.0) / dot_count as f32;
                                        let dot_alpha = (alpha_factor * 0.95).clamp(0.1, 1.0);
                                        let dot_size = 2.0 + alpha_factor * 2.5;

                                        let dot_bounds = Bounds {
                                            origin: Point {
                                                x: px(pos_x - dot_size * 0.5),
                                                y: px(pos_y - dot_size * 0.5),
                                            },
                                            size: Size {
                                                width: px(dot_size),
                                                height: px(dot_size),
                                            },
                                        };
                                        let color = gpui::rgba(
                                            0x388bfd00 | ((dot_alpha * 255.0) as u32),
                                        );
                                        window.paint_quad(fill(dot_bounds, color));
                                    }

                                    // 中心发光脉冲点
                                    let core_size = 6.0 * pulse_alpha;
                                    let core_bounds = Bounds {
                                        origin: Point {
                                            x: px(center_x - core_size * 0.5),
                                            y: px(center_y - core_size * 0.5),
                                        },
                                        size: Size {
                                            width: px(core_size),
                                            height: px(core_size),
                                        },
                                    };
                                    window.paint_quad(fill(
                                        core_bounds,
                                        gpui::rgba(0x58a6ffff),
                                    ));
                                },
                            )
                            .size_full(),
                        ),
                )
                // 2. 主标题
                .child(
                    div()
                        .text_base()
                        .font_semibold()
                        .text_color(pal.text_primary)
                        .child(title_text),
                )
                // 3. 实时正在提取的文件徽标胶囊 (Pill Badge)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_4()
                        .py_2()
                        .max_w(px(480.0))
                        .rounded_full()
                        .bg(gpui::rgba(0x0d1117cc))
                        .border_1()
                        .border_color(gpui::rgba(0x30363d80))
                        // 脉冲绿点指示活动状态
                        .child(
                            div()
                                .size(px(7.0))
                                .rounded_full()
                                .bg(gpui::rgba(0x2ea043ff)),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_medium()
                                .text_color(pal.text_secondary)
                                .child(format!("{}:", extracting_label)),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_semibold()
                                .text_color(gpui::rgb(0x58a6ff))
                                .overflow_hidden()
                                .child(current_file_label),
                        ),
                ),
        )
}
