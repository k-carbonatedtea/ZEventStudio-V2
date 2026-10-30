//! 全局文件与项目外部拖拽视觉反馈与动态波纹动画蒙层组件 (Animated Drag & Drop Overlay)

use crate::ui::state::AppState;
use gpui::*;
use std::time::{SystemTime, UNIX_EPOCH};

/// 渲染全局外部文件拖拽悬浮提示层（包含呼吸光晕、悬浮浮动与粒子波纹动画）
pub fn render_drag_drop_overlay(state: &AppState) -> impl IntoElement {
    let pal = state.theme();

    // 采样当前时间戳计算平滑动画相位
    let now_millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as f32)
        .unwrap_or(0.0);

    let time_sec = now_millis * 0.001;
    // 呼吸律动 (周期约 1.8 秒，取值范围 0.0 ~ 1.0)
    let breathe = (time_sec * 3.5).sin() * 0.5 + 0.5;
    // 图标上下悬浮动画 (-5px 到 +5px)
    let float_offset = (time_sec * 4.2).sin() * 5.0;
    // 双层能量光环扩散相位 (0.0 -> 1.0)
    let wave_phase_first = (now_millis % 1800.0) / 1800.0;
    let wave_phase_second = ((now_millis + 900.0) % 1800.0) / 1800.0;

    // 动态呼吸边框色彩 (基于遗迹青蓝 Primary Accent 计算呼吸透明度)
    let border_alpha = 0.50 + 0.45 * breathe;
    let border_color = hsla(191.0 / 360.0, 1.0, 0.50, border_alpha);
    let halo_alpha = 0.15 + 0.25 * breathe;
    let halo_border = hsla(191.0 / 360.0, 1.0, 0.50, halo_alpha);

    div()
        .id("app_global_drop_overlay")
        .absolute()
        .inset_0()
        .invisible()
        .group_drag_over::<ExternalPaths>("app_root_drop_target", |style| style.visible())
        .flex()
        .items_center()
        .justify_center()
        .bg(rgba(0x060c18d4)) // 蓝黑深色玻璃拟态底色
        .child(
            div()
                .relative()
                .flex()
                .flex_col()
                .items_center()
                .gap_4()
                .px_10()
                .py_8()
                .w(px(560.0))
                .rounded_2xl()
                .bg(rgba(0x0f1c2efa))
                .border_2()
                .border_color(border_color)
                .shadow_2xl()
                // 外层呼吸能量扩散光环 1
                .child(render_ripple_ring(wave_phase_first))
                // 外层呼吸能量扩散光环 2
                .child(render_ripple_ring(wave_phase_second))
                // 顶部浮动主图标容器
                .child(
                    div()
                        .relative()
                        .top(px(float_offset))
                        .w(px(76.0))
                        .h(px(76.0))
                        .rounded_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(rgba(0x00d2ff1f))
                        .border_2()
                        .border_color(halo_border)
                        .child(div().text_size(px(40.0)).child("📥")),
                )
                // 文本标题与副标题
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_1()
                        .child(
                            div()
                                .text_size(px(20.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(pal.text_primary)
                                .child(state.t("drop.overlay_title")),
                        )
                        .child(
                            div()
                                .text_size(px(12.5))
                                .text_color(pal.text_secondary)
                                .child(state.t("drop.overlay_hint")),
                        ),
                )
                // 格式徽章跑马灯流光渲染
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .mt_2()
                        .child(render_animated_badge(
                            "BFEVFL",
                            0,
                            time_sec,
                            rgba(0x00d2ff33),
                            rgba(0x00d2ffcc),
                        ))
                        .child(render_animated_badge(
                            "BFEVTM",
                            1,
                            time_sec,
                            rgba(0xffa50033),
                            rgba(0xffa500cc),
                        ))
                        .child(render_animated_badge(
                            "BFRES",
                            2,
                            time_sec,
                            rgba(0x9d4edd33),
                            rgba(0x9d4eddcc),
                        ))
                        .child(render_animated_badge(
                            "PACK / SARC",
                            3,
                            time_sec,
                            rgba(0x2ec4b633),
                            rgba(0x2ec4b6cc),
                        ))
                        .child(render_animated_badge(
                            "MOD DIR",
                            4,
                            time_sec,
                            rgba(0x3a86ff33),
                            rgba(0x3a86ffcc),
                        )),
                ),
        )
}

/// 渲染向外扩散的呼吸能量环
fn render_ripple_ring(phase: f32) -> impl IntoElement {
    let spread_offset = -6.0 - phase * 14.0;
    let ring_alpha = (1.0 - phase) * 0.35;
    let ring_color = hsla(191.0 / 360.0, 1.0, 0.50, ring_alpha);

    div()
        .absolute()
        .top(px(spread_offset))
        .bottom(px(spread_offset))
        .left(px(spread_offset))
        .right(px(spread_offset))
        .rounded(px(20.0 + phase * 10.0))
        .border_1()
        .border_color(ring_color)
}

/// 渲染带按序微光渐变动画的格式徽章
fn render_animated_badge(
    label: &'static str,
    index: usize,
    time_sec: f32,
    base_bg: impl Into<Hsla>,
    base_text: impl Into<Hsla>,
) -> impl IntoElement {
    let shimmer = ((time_sec * 3.8) + (index as f32) * 0.85).sin() * 0.5 + 0.5;
    let base_text_hsla: Hsla = base_text.into();
    let dynamic_text_hsla = Hsla {
        l: (base_text_hsla.l + shimmer * 0.16).clamp(0.0, 1.0),
        a: (0.75 + shimmer * 0.25).clamp(0.0, 1.0),
        ..base_text_hsla
    };

    let base_bg_hsla: Hsla = base_bg.into();
    let dynamic_bg_hsla = Hsla {
        a: (0.20 + shimmer * 0.18).clamp(0.0, 1.0),
        ..base_bg_hsla
    };

    div()
        .px_2p5()
        .py_1()
        .rounded_md()
        .bg(dynamic_bg_hsla)
        .border_1()
        .border_color(dynamic_text_hsla)
        .text_size(px(11.0))
        .font_weight(FontWeight::BOLD)
        .text_color(dynamic_text_hsla)
        .child(label)
}
