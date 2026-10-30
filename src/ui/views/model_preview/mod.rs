//! 3D 模型预览主视图容器 (集成视口渲染画布、轨道鼠标手势、工具栏与网格检查侧边栏)

pub mod gizmos;
pub mod mesh_sidebar;
pub mod rasterizer;
pub mod renderer;
pub mod stats_hud;
pub mod toolbar;
pub mod vulkan_renderer;

use crate::ui::state::AppState;
use gpui::*;
use mesh_sidebar::render_mesh_sidebar;
use renderer::draw_model_viewport;
use stats_hud::render_preview_hud;
use toolbar::render_preview_toolbar;

/// 渲染 3D 模型预览主交互视图 (Model Preview View)
pub fn render_model_preview_view(state: &AppState, model: &Entity<AppState>) -> impl IntoElement {
    let pal = state.theme();

    // 检索当前处于激活状态的 ModelPreview 文档标签
    let active_tab = state.active_tab_id.as_deref().and_then(|id| {
        state
            .open_tabs
            .iter()
            .find(|t| t.id == id && t.model_state.is_some())
    });

    let (tab_id, doc_state) = match active_tab {
        Some(t) => (t.id.clone(), t.model_state.clone().unwrap()),
        None => {
            return div()
                .size_full()
                .bg(pal.app_bg)
                .flex()
                .items_center()
                .justify_center()
                .text_color(pal.text_secondary)
                .text_size(px(14.0))
                .child(state.t("model_preview.no_model_selected"))
                .into_any_element();
        }
    };

    let doc_clone = doc_state.clone();
    let m_down = model.clone();
    let m_move = model.clone();
    let m_up = model.clone();
    let m_scroll = model.clone();
    let tid_down = tab_id.clone();
    let tid_move = tab_id.clone();
    let tid_up = tab_id.clone();
    let tid_scroll = tab_id.clone();

    div()
        .size_full()
        .bg(pal.app_bg)
        .flex()
        .flex_col()
        .overflow_hidden()
        // 顶部模式与视角工具栏
        .child(render_preview_toolbar(state, &doc_state, &tab_id, model))
        // 中间主体：3D 画布区域 + 右侧网格目录
        .child(
            div()
                .flex_1()
                .size_full()
                .flex()
                .overflow_hidden()
                // 左侧 3D 交互式视口渲染画布
                .child(
                    div()
                        .flex_1()
                        .size_full()
                        .relative()
                        .overflow_hidden()
                        // 3D 渲染核心 Canvas
                        .child(
                            canvas(
                                |_bounds, _window, _cx| {},
                                move |bounds, _, window, _cx| {
                                    draw_model_viewport(&doc_clone, bounds, window);
                                },
                            )
                            .size_full(),
                        )
                        // 鼠标左键/右键手势事件监听容器
                        .child(
                            div()
                                .absolute()
                                .inset_0()
                                // 鼠标按下：左键环绕旋转，右键/中键平移
                                .on_mouse_down(MouseButton::Left, move |event, _, cx| {
                                    let t = tid_down.clone();
                                    let pos = (event.position.x.into(), event.position.y.into());
                                    m_down.update(cx, |st, cx| {
                                        if let Some(tab) =
                                            st.open_tabs.iter_mut().find(|x| x.id == t)
                                        {
                                            if let Some(ref mut ms) = tab.model_state {
                                                ms.is_orbiting = true;
                                                ms.last_mouse_pos = pos;
                                                cx.notify();
                                            }
                                        }
                                    });
                                })
                                .on_mouse_down(MouseButton::Right, {
                                    let m = model.clone();
                                    let t = tab_id.clone();
                                    move |event, _, cx| {
                                        let t_id = t.clone();
                                        let pos =
                                            (event.position.x.into(), event.position.y.into());
                                        m.update(cx, |st, cx| {
                                            if let Some(tab) =
                                                st.open_tabs.iter_mut().find(|x| x.id == t_id)
                                            {
                                                if let Some(ref mut ms) = tab.model_state {
                                                    ms.is_panning = true;
                                                    ms.last_mouse_pos = pos;
                                                    cx.notify();
                                                }
                                            }
                                        });
                                    }
                                })
                                // 鼠标移动手势
                                .on_mouse_move(move |event, _, cx| {
                                    let t = tid_move.clone();
                                    let current_pos =
                                        (event.position.x.into(), event.position.y.into());
                                    m_move.update(cx, |st, cx| {
                                        if let Some(tab) =
                                            st.open_tabs.iter_mut().find(|x| x.id == t)
                                        {
                                            if let Some(ref mut ms) = tab.model_state {
                                                if ms.is_orbiting {
                                                    let dx = current_pos.0 - ms.last_mouse_pos.0;
                                                    let dy = current_pos.1 - ms.last_mouse_pos.1;
                                                    ms.camera.orbit(dx, dy);
                                                    ms.last_mouse_pos = current_pos;
                                                    cx.notify();
                                                } else if ms.is_panning {
                                                    let dx = current_pos.0 - ms.last_mouse_pos.0;
                                                    let dy = current_pos.1 - ms.last_mouse_pos.1;
                                                    ms.camera.pan(dx, dy, 600.0);
                                                    ms.last_mouse_pos = current_pos;
                                                    cx.notify();
                                                }
                                            }
                                        }
                                    });
                                })
                                // 鼠标抬起释放手势
                                .on_mouse_up(MouseButton::Left, {
                                    let m = m_up.clone();
                                    let t = tid_up.clone();
                                    move |_, _, cx| {
                                        let t_id = t.clone();
                                        m.update(cx, |st, cx| {
                                            if let Some(tab) =
                                                st.open_tabs.iter_mut().find(|x| x.id == t_id)
                                            {
                                                if let Some(ref mut ms) = tab.model_state {
                                                    ms.is_orbiting = false;
                                                    cx.notify();
                                                }
                                            }
                                        });
                                    }
                                })
                                .on_mouse_up(MouseButton::Right, {
                                    let m = model.clone();
                                    let t = tab_id.clone();
                                    move |_, _, cx| {
                                        let t_id = t.clone();
                                        m.update(cx, |st, cx| {
                                            if let Some(tab) =
                                                st.open_tabs.iter_mut().find(|x| x.id == t_id)
                                            {
                                                if let Some(ref mut ms) = tab.model_state {
                                                    ms.is_panning = false;
                                                    cx.notify();
                                                }
                                            }
                                        });
                                    }
                                })
                                .on_mouse_up_out(MouseButton::Left, {
                                    let m = model.clone();
                                    let t = tab_id.clone();
                                    move |_, _, cx| {
                                        let t_id = t.clone();
                                        m.update(cx, |st, cx| {
                                            if let Some(tab) =
                                                st.open_tabs.iter_mut().find(|x| x.id == t_id)
                                            {
                                                if let Some(ref mut ms) = tab.model_state {
                                                    if ms.is_orbiting {
                                                        ms.is_orbiting = false;
                                                        cx.notify();
                                                    }
                                                }
                                            }
                                        });
                                    }
                                })
                                .on_mouse_up_out(MouseButton::Right, {
                                    let m = model.clone();
                                    let t = tab_id.clone();
                                    move |_, _, cx| {
                                        let t_id = t.clone();
                                        m.update(cx, |st, cx| {
                                            if let Some(tab) =
                                                st.open_tabs.iter_mut().find(|x| x.id == t_id)
                                            {
                                                if let Some(ref mut ms) = tab.model_state {
                                                    if ms.is_panning {
                                                        ms.is_panning = false;
                                                        cx.notify();
                                                    }
                                                }
                                            }
                                        });
                                    }
                                })
                                // 滚轮缩放摄像机距离
                                .on_scroll(move |event, _, cx| {
                                    let t = tid_scroll.clone();
                                    let delta_y: f32 = match event.delta {
                                        ScrollDelta::Lines(l) => l.y * 18.0,
                                        ScrollDelta::Pixels(p) => p.y.into(),
                                    };
                                    let zoom_factor = if delta_y > 0.0 { 0.90 } else { 1.10 };

                                    m_scroll.update(cx, |st, cx| {
                                        if let Some(tab) =
                                            st.open_tabs.iter_mut().find(|x| x.id == t)
                                        {
                                            if let Some(ref mut ms) = tab.model_state {
                                                ms.camera.zoom(zoom_factor);
                                                cx.notify();
                                            }
                                        }
                                    });
                                }),
                        )
                        // 浮动模型技术统计与按键操作提示 HUD
                        .children(
                            (doc_state.show_stats).then(|| render_preview_hud(state, &doc_state)),
                        ),
                )
                // 右侧子网格分层侧边栏
                .child(render_mesh_sidebar(state, &doc_state, &tab_id, model)),
        )
        .into_any_element()
}
