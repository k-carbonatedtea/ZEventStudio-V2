use gpui::*;
use crate::ui::state::AppState;
use crate::ui::views::quest_map::types::world_to_screen;

/// 渲染地图上用户鼠标左键单点放置的单一自定义交互标点 (支持点击自身重置清空)
pub fn render_user_pin_layer(
    state: &AppState,
    model: &Entity<AppState>,
    pan: (f32, f32),
    zoom: f32,
) -> impl IntoElement {
    let user_pin = state.quest_map_user_pin;
    let model_click = model.clone();
    let model_close = model.clone();

    div()
        .id("user_interactive_pin_layer")
        .absolute()
        .inset_0()
        .children(user_pin.map(|(wx, wz)| {
            let (sx, sy) = world_to_screen(wx, wz, (0.0, 0.0), pan, zoom);
            let m_c = model_click.clone();
            let m_x = model_close.clone();

            let anchor_w = 160.0;
            let anchor_h = 70.0;

            div()
                .id("map_user_single_pin")
                .absolute()
                .left(px(sx - anchor_w / 2.0))
                .top(px(sy - anchor_h + 10.0)) // 尖端精准对齐 (sx, sy)
                .w(px(anchor_w))
                .h(px(anchor_h))
                .cursor_pointer()
                .flex()
                .flex_col()
                .items_center()
                .justify_end()
                .hover(|s| s.opacity(0.92))
                // 1. 点击标点自身重置清空
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    cx.stop_propagation();
                    m_c.update(cx, |st, cx| {
                        st.quest_map_user_pin = None;
                        cx.notify();
                    });
                })
                .on_mouse_up(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                // 顶部浮动坐标药丸卡片
                .child(
                    div()
                        .mb_1()
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .bg(rgba(0x081220ee))
                        .border_1()
                        .border_color(rgb(0x38bdf8))
                        .shadow_lg()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .child(
                            div()
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0x38bdf8))
                                .child("✦")
                        )
                        .child(
                            div()
                                .text_size(px(10.0))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xffffff))
                                .child(format!("X: {:.0}, Z: {:.0}", wx, wz))
                        )
                        .child(
                            div()
                                .px_1()
                                .rounded_xs()
                                .text_size(px(8.5))
                                .text_color(rgb(0x94a3b8))
                                .hover(|s| s.text_color(rgb(0xf43f5e)))
                                .child("✕")
                                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                    cx.stop_propagation();
                                    m_x.update(cx, |st, cx| {
                                        st.quest_map_user_pin = None;
                                        cx.notify();
                                    });
                                })
                                .on_mouse_up(MouseButton::Left, |_, _, cx| {
                                    cx.stop_propagation();
                                })
                        )
                )
                // 中部希卡立柱发光图钉主体
                .child(
                    div()
                        .relative()
                        .size(px(26.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        // 外层呼吸辉光
                        .child(
                            div()
                                .absolute()
                                .inset_0()
                                .rounded_full()
                                .bg(rgba(0x06b6d444))
                                .border_1()
                                .border_color(rgb(0x38bdf8).opacity(0.8))
                        )
                        // 内部发光核心
                        .child(
                            div()
                                .size(px(20.0))
                                .rounded_full()
                                .bg(rgb(0x0284c7))
                                .border_2()
                                .border_color(rgb(0xffffff))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_size(px(10.0))
                                .text_color(rgb(0xffffff))
                                .child("📍")
                        )
                )
                // 底部地面光波扩散光圈 (贴紧点击坐标中心)
                .child(
                    div()
                        .size(px(20.0))
                        .rounded_full()
                        .border_2()
                        .border_color(rgb(0x38bdf8).opacity(0.75))
                        .bg(rgba(0x06b6d433))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .size(px(4.0))
                                .rounded_full()
                                .bg(rgb(0xffffff))
                        )
                )
        }))
}
