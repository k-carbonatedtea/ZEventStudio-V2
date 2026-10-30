//! 3D 模型预览顶部控制工具栏

use gpui::*;
use crate::model::model_preview::{ModelDocumentState, PreviewPresetView, RenderMode};
use crate::ui::state::AppState;

/// 渲染 3D 模型预览视图顶部玻璃拟态工具栏
pub fn render_preview_toolbar(
    state: &AppState,
    doc_state: &ModelDocumentState,
    tab_id: &str,
    model: &Entity<AppState>,
) -> impl IntoElement {
    let pal = state.theme();
    let current_mode = doc_state.render_mode;
    let tid = tab_id.to_string();

    div()
        .h(px(38.0))
        .w_full()
        .bg(pal.panel_surface)
        .border_b_1()
        .border_color(pal.panel_border)
        .flex()
        .items_center()
        .justify_between()
        .px_3()
        .gap_2()

        // 左侧：渲染模式切换药丸胶囊
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .bg(pal.card_surface)
                .p_0p5()
                .rounded_md()
                .border_1()
                .border_color(pal.panel_border)
                .children([
                    RenderMode::Shaded,
                    RenderMode::Wireframe,
                    RenderMode::ShadedWireframe,
                    RenderMode::Points,
                ].into_iter().map(|mode| {
                    let is_active = mode == current_mode;
                    let m_clone = model.clone();
                    let t_clone = tid.clone();
                    let label = state.t(mode.i18n_key());
                    let icon = mode.icon();

                    div()
                        .h(px(26.0))
                        .px_2()
                        .rounded_xs()
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .text_size(px(11.0))
                        .bg(if is_active { pal.primary_accent } else { rgba(0x00000000) })
                        .text_color(if is_active { pal.app_bg } else { pal.text_secondary })
                        .hover(|s| {
                            if !is_active {
                                s.bg(pal.card_hover).text_color(pal.text_primary)
                            } else {
                                s
                            }
                        })
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            let t = t_clone.clone();
                            m_clone.update(cx, |st, cx| {
                                if let Some(tab) = st.open_tabs.iter_mut().find(|x| x.id == t) {
                                    if let Some(ref mut ms) = tab.model_state {
                                        ms.render_mode = mode;
                                        cx.notify();
                                    }
                                }
                            });
                        })
                        .child(icon)
                        .child(label)
                }))
        )

        // 中间：图层开关辅助切换 (地平面网格、坐标轴、外接包围盒、数据 HUD)
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .child(render_toggle_button(
                    state.t("model_preview.toggle_grid"),
                    "🌐",
                    doc_state.show_grid,
                    &tid,
                    model,
                    &pal,
                    |ms| ms.show_grid = !ms.show_grid,
                ))
                .child(render_toggle_button(
                    state.t("model_preview.toggle_axes"),
                    "📐",
                    doc_state.show_axes,
                    &tid,
                    model,
                    &pal,
                    |ms| ms.show_axes = !ms.show_axes,
                ))
                .child(render_toggle_button(
                    state.t("model_preview.toggle_bounds"),
                    "📦",
                    doc_state.show_bounds,
                    &tid,
                    model,
                    &pal,
                    |ms| ms.show_bounds = !ms.show_bounds,
                ))
                .child(render_toggle_button(
                    state.t("model_preview.toggle_stats"),
                    "📊",
                    doc_state.show_stats,
                    &tid,
                    model,
                    &pal,
                    |ms| ms.show_stats = !ms.show_stats,
                ))
        )

        // 右侧：摄像机快捷预设视角与重置对齐按键
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .child(render_preset_button(state.t("model_preview.view_front"), PreviewPresetView::Front, &tid, model, &pal))
                .child(render_preset_button(state.t("model_preview.view_side"), PreviewPresetView::Right, &tid, model, &pal))
                .child(render_preset_button(state.t("model_preview.view_top"), PreviewPresetView::Top, &tid, model, &pal))
                .child(render_preset_button(state.t("model_preview.view_iso"), PreviewPresetView::Isometric, &tid, model, &pal))
                // 居中适配模型视角
                .child({
                    let m_fit = model.clone();
                    let t_fit = tid.clone();
                    div()
                        .h(px(26.0))
                        .px_2p5()
                        .ml_1()
                        .rounded_xs()
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .gap_1()
                        .text_size(px(11.0))
                        .font_weight(FontWeight::MEDIUM)
                        .bg(pal.card_surface)
                        .border_1()
                        .border_color(pal.primary_accent)
                        .text_color(pal.primary_accent)
                        .hover(|s| s.bg(pal.card_hover))
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            let t = t_fit.clone();
                            m_fit.update(cx, |st, cx| {
                                if let Some(tab) = st.open_tabs.iter_mut().find(|x| x.id == t) {
                                    if let Some(ref mut ms) = tab.model_state {
                                        ms.fit_camera();
                                        cx.notify();
                                    }
                                }
                            });
                        })
                        .child("🎯")
                        .child(state.t("model_preview.fit_view"))
                })
        )
}

/// 渲染开关图标胶囊按钮
fn render_toggle_button(
    title: &'static str,
    icon: &'static str,
    is_active: bool,
    tab_id: &str,
    model: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
    action: fn(&mut ModelDocumentState),
) -> impl IntoElement {
    let m = model.clone();
    let tid = tab_id.to_string();

    div()
        .h(px(26.0))
        .px_2()
        .rounded_xs()
        .cursor_pointer()
        .flex()
        .items_center()
        .gap_1()
        .text_size(px(11.0))
        .bg(if is_active { pal.card_surface } else { rgba(0x00000000) })
        .border_1()
        .border_color(if is_active { pal.primary_accent } else { pal.panel_border })
        .text_color(if is_active { pal.text_primary } else { pal.text_secondary })
        .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            let t = tid.clone();
            m.update(cx, |st, cx| {
                if let Some(tab) = st.open_tabs.iter_mut().find(|x| x.id == t) {
                    if let Some(ref mut ms) = tab.model_state {
                        action(ms);
                        cx.notify();
                    }
                }
            });
        })
        .child(icon)
        .child(title)
}

/// 渲染视角预设按钮
fn render_preset_button(
    title: &'static str,
    preset: PreviewPresetView,
    tab_id: &str,
    model: &Entity<AppState>,
    pal: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let m = model.clone();
    let tid = tab_id.to_string();

    div()
        .h(px(26.0))
        .px_2()
        .rounded_xs()
        .cursor_pointer()
        .flex()
        .items_center()
        .text_size(px(11.0))
        .bg(pal.card_surface)
        .border_1()
        .border_color(pal.panel_border)
        .text_color(pal.text_secondary)
        .hover(|s| s.bg(pal.card_hover).text_color(pal.text_primary))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            let t = tid.clone();
            m.update(cx, |st, cx| {
                if let Some(tab) = st.open_tabs.iter_mut().find(|x| x.id == t) {
                    if let Some(ref mut ms) = tab.model_state {
                        ms.camera.set_preset(preset);
                        cx.notify();
                    }
                }
            });
        })
        .child(title)
}
