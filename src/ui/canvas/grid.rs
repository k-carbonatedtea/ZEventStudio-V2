use gpui::*;

pub fn draw_canvas_grid(
    bounds: Bounds<Pixels>,
    pan: (f32, f32),
    zoom: f32,
    window: &mut Window,
) {
    let orig_x: f32 = bounds.origin.x.into();
    let orig_y: f32 = bounds.origin.y.into();
    let w: f32 = bounds.size.width.into();
    let h: f32 = bounds.size.height.into();

    let grid_step = 28.0 * zoom;
    let dot_color = hsla(240.0 / 360.0, 0.05, 0.22, 0.40);
    let major_dot_color = hsla(240.0 / 360.0, 0.10, 0.35, 0.65);

    let start_x = (pan.0 % grid_step) + orig_x;
    let start_y = (pan.1 % grid_step) + orig_y;

    let mut x = start_x;
    let mut col_idx = 0;
    while x <= orig_x + w {
        let mut y = start_y;
        let mut row_idx = 0;
        while y <= orig_y + h {
            let is_major = (col_idx % 4 == 0) && (row_idx % 4 == 0);
            let dot_size = if is_major { 2.2 * zoom.max(0.7) } else { 1.2 * zoom.max(0.7) };
            let color = if is_major { major_dot_color } else { dot_color };

            window.paint_quad(fill(
                Bounds {
                    origin: Point {
                        x: px(x - dot_size / 2.0),
                        y: px(y - dot_size / 2.0),
                    },
                    size: size(px(dot_size), px(dot_size)),
                },
                color,
            ));

            y += grid_step;
            row_idx += 1;
        }
        x += grid_step;
        col_idx += 1;
    }
}
