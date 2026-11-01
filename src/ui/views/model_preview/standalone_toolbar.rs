//! 独立模型预览窗口顶部工具栏组件

use gpui::*;
use crate::model::model_preview::{PreviewPresetView, RenderMode};
use crate::ui::theme::ThemePalette;
use super::standalone_window::StandaloneModelPreviewView;

/// 渲染顶部渲染模式与相机视角工具栏
pub fn render_standalone_toolbar(
    view_state: &StandaloneModelPreviewView,
    pal: &ThemePalette,
    view: &Entity<StandaloneModelPreviewView>,
) -> impl IntoElement {
    let current_mode = view_state.model_state.render_mode;
    let show_grid = view_state.model_state.show_grid;
    let show_axes = view_state.model_state.show_axes;
    let show_bounds = view_state.model_state.show_bounds;
    let show_stats = view_state.model_state.show_stats;
    let show_sidebar = view_state.show_sidebar;

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
        // 左侧：渲染模式切换药丸组 (着色 / 线框 / 线框叠加 / 点云)
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
                    let v = view.clone();
                    let label = view_state.t(mode.i18n_key());
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
                        .hover(move |s| {
                            if !is_active {
                                s.bg(pal.card_hover).text_color(pal.text_primary)
                            } else {
                                s
                            }
                        })
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            v.update(cx, |this, cx| {
                                this.model_state.render_mode = mode;
                                cx.notify();
                            });
                        })
                        .child(icon)
                        .child(label)
                })),
        )
        // 中间：视角预设与辅助图层开关
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                // 适应居中对焦按钮
                .child(render_action_btn("🎯", view_state.t("model_preview.fit_view"), pal, {
                    let v = view.clone();
                    move |cx| {
                        v.update(cx, |this, cx| {
                            this.model_state.fit_camera();
                            cx.notify();
                        });
                    }
                }))
                // 正面视角
                .child(render_action_btn("👁️", view_state.t("model_preview.view_front"), pal, {
                    let v = view.clone();
                    move |cx| {
                        v.update(cx, |this, cx| {
                            this.model_state.camera.set_preset(PreviewPresetView::Front);
                            cx.notify();
                        });
                    }
                }))
                // 顶视
                .child(render_action_btn("🔝", view_state.t("model_preview.view_top"), pal, {
                    let v = view.clone();
                    move |cx| {
                        v.update(cx, |this, cx| {
                            this.model_state.camera.set_preset(PreviewPresetView::Top);
                            cx.notify();
                        });
                    }
                }))
                // 空间网格开关
                .child(render_toggle_btn("🌐", view_state.t("model_preview.toggle_grid"), show_grid, pal, {
                    let v = view.clone();
                    move |cx| {
                        v.update(cx, |this, cx| {
                            this.model_state.show_grid = !this.model_state.show_grid;
                            cx.notify();
                        });
                    }
                }))
                // 世界坐标轴开关
                .child(render_toggle_btn("📐", view_state.t("model_preview.toggle_axes"), show_axes, pal, {
                    let v = view.clone();
                    move |cx| {
                        v.update(cx, |this, cx| {
                            this.model_state.show_axes = !this.model_state.show_axes;
                            cx.notify();
                        });
                    }
                }))
                // 包围盒开关
                .child(render_toggle_btn("📦", view_state.t("model_preview.toggle_bounds"), show_bounds, pal, {
                    let v = view.clone();
                    move |cx| {
                        v.update(cx, |this, cx| {
                            this.model_state.show_bounds = !this.model_state.show_bounds;
                            cx.notify();
                        });
                    }
                }))
                // 技术统计 HUD 开关
                .child(render_toggle_btn("📊", view_state.t("model_preview.toggle_stats"), show_stats, pal, {
                    let v = view.clone();
                    move |cx| {
                        v.update(cx, |this, cx| {
                            this.model_state.show_stats = !this.model_state.show_stats;
                            cx.notify();
                        });
                    }
                })),
        )
        // 右侧：子网格侧边栏折叠按钮
        .child(
            render_toggle_btn("🧩", view_state.t("model_preview.toggle_sidebar"), show_sidebar, pal, {
                let v = view.clone();
                move |cx| {
                    v.update(cx, |this, cx| {
                        this.show_sidebar = !this.show_sidebar;
                        cx.notify();
                    });
                }
            }),
        )
}

/// 渲染动作按钮
fn render_action_btn<F>(icon: &'static str, label: &'static str, pal: &ThemePalette, action: F) -> impl IntoElement
where
    F: Fn(&mut App) + 'static,
{
    div()
        .h(px(26.0))
        .px_2()
        .rounded_xs()
        .cursor_pointer()
        .flex()
        .items_center()
        .gap_1()
        .text_size(px(11.0))
        .bg(pal.card_surface)
        .border_1()
        .border_color(pal.panel_border)
        .text_color(pal.text_primary)
        .hover(|s| s.bg(pal.card_hover))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| action(cx))
        .child(icon)
        .child(label)
}

/// 渲染开关按钮
fn render_toggle_btn<F>(icon: &'static str, label: &'static str, is_active: bool, pal: &ThemePalette, action: F) -> impl IntoElement
where
    F: Fn(&mut App) + 'static,
{
    div()
        .h(px(26.0))
        .px_2()
        .rounded_xs()
        .cursor_pointer()
        .flex()
        .items_center()
        .gap_1()
        .text_size(px(11.0))
        .bg(if is_active { pal.card_hover } else { pal.card_surface })
        .border_1()
        .border_color(if is_active { pal.primary_accent } else { pal.panel_border })
        .text_color(if is_active { pal.primary_accent } else { pal.text_secondary })
        .hover(|s| s.opacity(0.85))
        .on_mouse_down(MouseButton::Left, move |_, _, cx| action(cx))
        .child(icon)
        .child(label)
}
