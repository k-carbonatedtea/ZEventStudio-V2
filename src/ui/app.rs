use gpui::*;
use crate::ui::canvas::render_flow_canvas;
use crate::ui::components::render_toast;
use crate::ui::inspector::render_inspector;
use crate::ui::sidebar::render_sidebar;
use crate::ui::state::AppState;
use crate::ui::toolbar::render_toolbar;

pub struct AppView {
    pub model: Entity<AppState>,
    pub frame_count: u32,
    pub last_fps_update: std::time::Instant,
}

impl AppView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let model = cx.new(|_| AppState::new());
        Self {
            model,
            frame_count: 0,
            last_fps_update: std::time::Instant::now(),
        }
    }
}

impl Render for AppView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Real-time FPS sampling
        self.frame_count += 1;
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_fps_update).as_secs_f32();
        if elapsed >= 0.35 {
            let current_fps = ((self.frame_count as f32 / elapsed).round()).max(1.0);
            self.frame_count = 0;
            self.last_fps_update = now;
            self.model.update(cx, |state, _| {
                state.fps = current_fps;
            });
        }



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
