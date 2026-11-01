//! 独立弹出式 3D 模型预览窗口视图 (集成视口渲染、轨道摄像机操作、模式切换与网格分层控制)

use gpui::*;
use gpui_component::TitleBar;
use crate::i18n::Language;
use crate::model::model_preview::{ActorModelTarget, ModelDocumentState};
use crate::ui::theme::{AppTheme, ThemePalette};
use super::renderer::draw_model_viewport;
use super::standalone_sidebar::render_standalone_sidebar;
use super::standalone_toolbar::render_standalone_toolbar;

/// 独立窗口 3D 模型预览视图实体
pub struct StandaloneModelPreviewView {
    /// 3D 渲染与相机状态
    pub model_state: ModelDocumentState,
    /// Actor 与模型配置元数据
    pub target: ActorModelTarget,
    /// 当前界面多语言设置
    pub lang: Language,
    /// 是否展开右侧子网格分层管理侧边栏
    pub show_sidebar: bool,
    /// 统一界面设计系统调色板
    palette: ThemePalette,
}

impl StandaloneModelPreviewView {
    /// 构建独立模型预览视图
    pub fn new(
        target: ActorModelTarget,
        model_state: ModelDocumentState,
        lang: Language,
    ) -> Self {
        Self {
            model_state,
            target,
            lang,
            show_sidebar: true,
            palette: AppTheme::default().palette(),
        }
    }

    /// 本地化快捷翻译辅助方法
    pub fn t<'a>(&self, key: &'a str) -> &'a str {
        crate::i18n::t(key, self.lang)
    }

    /// 切换所有子网格的可见性
    pub fn set_all_submeshes_visibility(&mut self, visible: bool) {
        self.model_state.set_all_visibility(visible);
    }

    /// 切换指定索引子网格的可见性
    pub fn toggle_submesh_visibility(&mut self, index: usize) {
        self.model_state.toggle_mesh_visibility(index);
    }
}

/// 弹出独立窗口以渲染指定 Actor 的 3D 模型
pub fn open_standalone_model_preview_window(
    target: ActorModelTarget,
    model_state: ModelDocumentState,
    lang: Language,
    cx: &mut App,
) -> Result<(), String> {
    let mut options = TitleBar::window_options();
    options.window_bounds = Some(WindowBounds::Windowed(Bounds::centered(
        None,
        size(px(1120.0), px(740.0)),
        cx,
    )));

    cx.open_window(options, |_window, cx| {
        cx.new(|_cx| StandaloneModelPreviewView::new(target, model_state, lang))
    })
    .map_err(|e| format!("{:?}", e))?;

    Ok(())
}

impl Render for StandaloneModelPreviewView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pal = self.palette;
        let view_entity = cx.entity().clone();
        let doc_state = self.model_state.clone();

        div()
            .size_full()
            .bg(pal.app_bg)
            .flex()
            .flex_col()
            .overflow_hidden()
            // 1. 顶部无边框玻璃拟态标题栏 (集成系统窗口拖拽与关闭按钮)
            .child(self.render_window_title_bar(&pal))
            // 2. 模式与控制工具栏
            .child(render_standalone_toolbar(self, &pal, &view_entity))
            // 3. 视口主体 (3D 交互画布 + 浮动 HUD + 可选右侧网格侧边栏)
            .child(
                div()
                    .flex_1()
                    .size_full()
                    .flex()
                    .overflow_hidden()
                    .child(self.render_viewport_canvas(&doc_state, &pal, &view_entity))
                    .children(self.show_sidebar.then(|| render_standalone_sidebar(self, &pal, &view_entity))),
            )
    }
}

impl StandaloneModelPreviewView {
    /// 渲染独立窗口顶部标题栏
    fn render_window_title_bar(&self, pal: &ThemePalette) -> impl IntoElement {
        let title_text = format!(
            "{} - {} ({})",
            self.t("model_preview.window_title"),
            self.target.actor_name,
            self.target.folder
        );

        TitleBar::new()
            .bg(pal.panel_surface)
            .border_color(pal.panel_border)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .child(div().text_size(px(13.0)).child("🧊"))
                    .child(
                        div()
                            .text_size(px(12.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(pal.text_primary)
                            .child(title_text),
                    ),
            )
    }

    /// 渲染 3D 交互视口画布及鼠标手势
    fn render_viewport_canvas(
        &self,
        doc_clone: &ModelDocumentState,
        pal: &ThemePalette,
        view: &Entity<Self>,
    ) -> impl IntoElement {
        let v_down = view.clone();
        let v_move = view.clone();
        let v_up = view.clone();
        let v_scroll = view.clone();
        let doc_paint = doc_clone.clone();

        div()
            .flex_1()
            .size_full()
            .relative()
            .overflow_hidden()
            // 核心 Canvas
            .child(
                canvas(
                    |_bounds, _window, _cx| {},
                    move |bounds, _, window, _cx| {
                        draw_model_viewport(&doc_paint, bounds, window);
                    },
                )
                .size_full(),
            )
            // 鼠标交互捕获层
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .on_mouse_down(MouseButton::Left, move |event, _, cx| {
                        let pos = (event.position.x.into(), event.position.y.into());
                        v_down.update(cx, |this, cx| {
                            this.model_state.is_orbiting = true;
                            this.model_state.last_mouse_pos = pos;
                            cx.notify();
                        });
                    })
                    .on_mouse_down(MouseButton::Right, {
                        let v = view.clone();
                        move |event, _, cx| {
                            let pos = (event.position.x.into(), event.position.y.into());
                            v.update(cx, |this, cx| {
                                this.model_state.is_panning = true;
                                this.model_state.last_mouse_pos = pos;
                                cx.notify();
                            });
                        }
                    })
                    .on_mouse_move(move |event, _, cx| {
                        let current_pos = (event.position.x.into(), event.position.y.into());
                        v_move.update(cx, |this, cx| {
                            if this.model_state.is_orbiting {
                                let dx = current_pos.0 - this.model_state.last_mouse_pos.0;
                                let dy = current_pos.1 - this.model_state.last_mouse_pos.1;
                                this.model_state.camera.orbit(dx, dy);
                                this.model_state.last_mouse_pos = current_pos;
                                cx.notify();
                            } else if this.model_state.is_panning {
                                let dx = current_pos.0 - this.model_state.last_mouse_pos.0;
                                let dy = current_pos.1 - this.model_state.last_mouse_pos.1;
                                this.model_state.camera.pan(dx, dy, 600.0);
                                this.model_state.last_mouse_pos = current_pos;
                                cx.notify();
                            }
                        });
                    })
                    .on_mouse_up(MouseButton::Left, {
                        let v = v_up.clone();
                        move |_, _, cx| {
                            v.update(cx, |this, cx| {
                                this.model_state.is_orbiting = false;
                                cx.notify();
                            });
                        }
                    })
                    .on_mouse_up(MouseButton::Right, {
                        let v = view.clone();
                        move |_, _, cx| {
                            v.update(cx, |this, cx| {
                                this.model_state.is_panning = false;
                                cx.notify();
                            });
                        }
                    })
                    .on_mouse_up_out(MouseButton::Left, {
                        let v = view.clone();
                        move |_, _, cx| {
                            v.update(cx, |this, cx| {
                                this.model_state.is_orbiting = false;
                                cx.notify();
                            });
                        }
                    })
                    .on_mouse_up_out(MouseButton::Right, {
                        let v = view.clone();
                        move |_, _, cx| {
                            v.update(cx, |this, cx| {
                                this.model_state.is_panning = false;
                                cx.notify();
                            });
                        }
                    })
                    .on_scroll(move |event, _, cx| {
                        let delta_y: f32 = match event.delta {
                            ScrollDelta::Lines(l) => l.y * 18.0,
                            ScrollDelta::Pixels(p) => p.y.into(),
                        };
                        let zoom_factor = if delta_y > 0.0 { 0.90 } else { 1.10 };
                        v_scroll.update(cx, |this, cx| {
                            this.model_state.camera.zoom(zoom_factor);
                            cx.notify();
                        });
                    }),
            )
            // 浮动技术 HUD
            .children(self.model_state.show_stats.then(|| self.render_hud(pal)))
    }

    /// 渲染浮动技术指标面板
    fn render_hud(&self, pal: &ThemePalette) -> impl IntoElement {
        let dims = self.model_state.dimensions();
        let dist = self.model_state.camera.distance;

        div()
            .absolute()
            .top_3()
            .left_3()
            .p_3()
            .w(px(230.0))
            .bg(pal.card_surface)
            .border_1()
            .border_color(pal.panel_border)
            .rounded_lg()
            .shadow_md()
            .flex()
            .flex_col()
            .gap_1p5()
            .child(
                div()
                    .text_size(px(11.5))
                    .font_weight(FontWeight::BOLD)
                    .text_color(pal.primary_accent)
                    .child(format!("📦 {}", self.target.actor_name)),
            )
            .child(self.render_hud_line(self.t("model_preview.stat_meshes"), format!("{}/{}", self.model_state.visible_meshes_count(), self.model_state.mesh_items.len()), pal))
            .child(self.render_hud_line(self.t("model_preview.stat_vertices"), format!("{}", self.model_state.visible_vertices_count()), pal))
            .child(self.render_hud_line(self.t("model_preview.stat_triangles"), format!("{}", self.model_state.visible_triangles_count()), pal))
            .child(self.render_hud_line(self.t("model_preview.stat_dimensions"), format!("{:.2} × {:.2} × {:.2}", dims[0], dims[1], dims[2]), pal))
            .child(self.render_hud_line(self.t("model_preview.stat_distance"), format!("{:.2} m", dist), pal))
            .child(
                div()
                    .mt_1()
                    .pt_1p5()
                    .border_t_1()
                    .border_color(pal.panel_border)
                    .text_size(px(10.0))
                    .text_color(pal.text_secondary)
                    .flex()
                    .flex_col()
                    .gap_0p5()
                    .child(self.t("model_preview.hint_orbit"))
                    .child(self.t("model_preview.hint_pan"))
                    .child(self.t("model_preview.hint_zoom")),
            )
    }

    /// HUD 单行键值对展示
    fn render_hud_line(&self, label: &'static str, value: String, pal: &ThemePalette) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .text_size(px(10.5))
            .child(div().text_color(pal.text_secondary).child(label))
            .child(div().font_weight(FontWeight::SEMIBOLD).text_color(pal.text_primary).child(value))
    }
}
