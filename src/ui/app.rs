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
    pub fps: f32,
    pub frame_time_ms: f32,
}

impl AppView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let model = cx.new(|_| AppState::new());
        Self {
            model,
            frame_count: 0,
            last_fps_update: Instant::now(),
            fps: 60.0,
            frame_time_ms: 1.0,
        }
    }
}

impl Render for AppView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_active = window.is_window_active();

        let state = self.model.read(cx);
        let is_perf = state.performance_mode;

        // 实时真实帧率与单帧耗时采样检测（直接记录在 AppView 本地，杜绝 render 内部 update 触发死循环）
        self.frame_count += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_fps_update).as_secs_f32();

        if elapsed >= 0.25 {
            self.fps = ((self.frame_count as f32 / elapsed).round()).max(1.0);
            self.frame_time_ms = if self.fps > 0.0 { 1000.0 / self.fps } else { 0.0 };
            self.frame_count = 0;
            self.last_fps_update = now;
        }

        // 动态帧率控制与性能模式调度：
        if is_perf && is_active {
            // 🔥 性能模式：彻底解除一切帧率限制与垂直同步节流，释放 GPU 极限吞吐算力 (500~1000+ FPS)
            cx.on_next_frame(window, |_, _, cx| {
                cx.notify();
            });
            cx.spawn_in(window, async move |view, cx| {
                let _ = cx.update(|_window, cx| {
                    let _ = view.update(cx, |_, cx| {
                        cx.notify();
                    });
                });
            })
            .detach();
        } else {
            // 🌿 标准节能模式：通过微定时器平滑调度 (~120-144 FPS)，兼顾极致流畅与节能
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
        }

        let state = self.model.read(cx);
        let toast = state.toast.clone();
        let current_fps = self.fps;
        let current_frame_time = self.frame_time_ms;

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(hsla(220.0 / 360.0, 0.35, 0.08, 1.0))
            .font_family(".SystemUIFont")
            .text_color(rgb(0xffffff))
            // Top Toolbar
            .child(render_toolbar(state, current_fps, current_frame_time, &self.model))
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
