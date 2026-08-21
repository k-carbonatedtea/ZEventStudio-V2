use gpui::*;
use std::time::{Duration, Instant};
use crate::ui::canvas::render_flow_canvas;
use crate::ui::components::render_toast;
use crate::ui::inspector::render_inspector;
use crate::ui::sidebar::render_sidebar;
use crate::ui::state::AppState;
use crate::ui::toolbar::render_toolbar;

pub struct AppView {
    pub model: Entity<AppState>,
    pub frame_count: u32,
    pub last_fps_update: Instant,
}

impl AppView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let model = cx.new(|_| AppState::new());
        Self {
            model,
            frame_count: 0,
            last_fps_update: Instant::now(),
        }
    }
}

impl Render for AppView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_active = window.is_window_active();

        // 实时真实帧率采样检测
        self.frame_count += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_fps_update).as_secs_f32();

        if elapsed >= 0.25 {
            let measured_fps = ((self.frame_count as f32 / elapsed).round()).max(1.0);
            let final_fps = if is_active {
                measured_fps
            } else {
                measured_fps.min(60.0)
            };

            self.frame_count = 0;
            self.last_fps_update = now;
            self.model.update(cx, |state, _| {
                state.fps = final_fps;
            });
        }

        // 动态帧率平滑调度：
        // 配合 smol 微定时器让渡 CPU/GPU 时间片，保障极致流畅响应（~120-144 FPS）的同时消除无节制空转掉帧
        let sleep_ms = if is_active { 7 } else { 33 };
        cx.spawn_in(window, async move |view, cx| {
            smol::Timer::after(Duration::from_millis(sleep_ms)).await;
            let _ = cx.update(|_window, cx| {
                let _ = view.update(cx, |_, cx| {
                    cx.notify();
                });
            });
        })
        .detach();

        let state = self.model.read(cx);
        let toast = state.toast.clone();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(hsla(220.0 / 360.0, 0.35, 0.08, 1.0))
            .font_family(".SystemUIFont")
            .text_color(rgb(0xffffff))
            // Top Toolbar
            .child(render_toolbar(state, &self.model))
            // Workspace Body (Sidebar + Flow Canvas + Inspector)
            .child(
                div()
                    .flex()
                    .flex_1()
                    .size_full()
                    .overflow_hidden()
                    // Left Sidebar
                    .child(render_sidebar(state, &self.model))
                    // Center Canvas
                    .child(render_flow_canvas(state, &self.model))
                    // Right Inspector
                    .child(render_inspector(state, &self.model)),
            )
            // Floating Toast notification
            .children(toast.map(|t| {
                div()
                    .absolute()
                    .bottom_4()
                    .right_4()
                    .child(render_toast(&t))
            }))
    }
}
