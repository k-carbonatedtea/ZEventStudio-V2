use gpui::*;
use crate::presets::get_available_presets;
use crate::ui::state::AppState;

/// 渲染官方 EVFL 预设流案例列表
pub fn render_preset_list(
    state: &AppState,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let presets = get_available_presets();
    let current_id = state.active_preset_id.clone();

    div()
        .flex()
        .flex_col()
        .gap_2p5()
        .p_3()
        .children(presets.into_iter().map(|preset| {
            let is_active = preset.id == current_id;
            let p_id = preset.id.to_string();
            let model_load = model.clone();

            div()
                .p_3()
                .rounded_lg()
                .bg(if is_active {
                    hsla(215.0 / 360.0, 0.45, 0.20, 0.95)
                } else {
                    hsla(220.0 / 360.0, 0.30, 0.14, 0.8)
                })
                .border_1()
                .border_color(if is_active {
                    hsla(145.0 / 360.0, 0.7, 0.50, 1.0)
                } else {
                    hsla(215.0 / 360.0, 0.25, 0.25, 0.8)
                })
                .hover(|s| {
                    if !is_active {
                        s.bg(hsla(215.0 / 360.0, 0.35, 0.20, 1.0))
                    } else {
                        s
                    }
                })
                .cursor_pointer()
                .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                    let pid = p_id.clone();
                    model_load.update(cx, |st, cx| {
                        st.load_preset(&pid);
                        cx.notify();
                    });
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_size(px(12.5))
                                .text_color(rgb(0xffffff))
                                .child(preset.name),
                        )
                        .children(if is_active {
                            Some(
                                div()
                                    .px_1p5()
                                    .py_0p5()
                                    .rounded_sm()
                                    .bg(hsla(145.0 / 360.0, 0.6, 0.35, 1.0))
                                    .text_size(px(9.5))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xffffff))
                                    .child(state.t("sidebar.active_badge")),
                            )
                        } else {
                            None
                        }),
                )
                .child(
                    div()
                        .mt_1()
                        .text_size(px(10.5))
                        .text_color(hsla(215.0 / 360.0, 0.25, 0.70, 1.0))
                        .child(preset.description),
                )
        }))
}

