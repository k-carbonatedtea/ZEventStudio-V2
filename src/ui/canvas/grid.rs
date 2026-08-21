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

    if w <= 0.0 || h <= 0.0 {
        return;
    }

    // 自适应网格间距 (Adaptive Grid LOD)：
    // 无论视图缩小还是放大，屏幕点阵间隙始终稳定在 [24.0, 56.0] 像素之间，
    // 彻底解决视图缩小 (zoom out) 时点阵数量暴增 40 倍引发的严重卡顿与掉帧。
    let base_spacing = 32.0;
    let mut grid_step = base_spacing * zoom;
    while grid_step < 24.0 {
        grid_step *= 2.0;
    }
    while grid_step > 56.0 {
        grid_step /= 2.0;
    }

    let dot_color = hsla(240.0 / 360.0, 0.05, 0.22, 0.40);
    let major_dot_color = hsla(240.0 / 360.0, 0.10, 0.35, 0.65);

    let start_x = ((pan.0 % grid_step) + grid_step) % grid_step + orig_x;
    let start_y = ((pan.1 % grid_step) + grid_step) % grid_step + orig_y;

    let dot_size = (1.6 * zoom.clamp(0.6, 1.2)).clamp(1.0, 2.4);
    let major_dot_size = dot_size * 1.5;

    let mut normal_dots = PathBuilder::fill();
    let mut major_dots = PathBuilder::fill();
    let mut has_normal = false;
    let mut has_major = false;

    let mut x = start_x;
    let mut col_idx = 0;
    while x <= orig_x + w {
        let mut y = start_y;
        let mut row_idx = 0;
        while y <= orig_y + h {
            let is_major = (col_idx % 4 == 0) && (row_idx % 4 == 0);
            let cur_dot_size = if is_major { major_dot_size } else { dot_size };
            let r = cur_dot_size * 0.5;

            if is_major {
                has_major = true;
                major_dots.move_to(point(px(x - r), px(y - r)));
                major_dots.line_to(point(px(x + r), px(y - r)));
                major_dots.line_to(point(px(x + r), px(y + r)));
                major_dots.line_to(point(px(x - r), px(y + r)));
                major_dots.close();
            } else {
                has_normal = true;
                normal_dots.move_to(point(px(x - r), px(y - r)));
                normal_dots.line_to(point(px(x + r), px(y - r)));
                normal_dots.line_to(point(px(x + r), px(y + r)));
                normal_dots.line_to(point(px(x - r), px(y + r)));
                normal_dots.close();
            }

            y += grid_step;
            row_idx += 1;
        }
        x += grid_step;
        col_idx += 1;
    }

    if has_normal {
        if let Ok(path) = normal_dots.build() {
            window.paint_path(path, dot_color);
        }
    }
    if has_major {
        if let Ok(path) = major_dots.build() {
            window.paint_path(path, major_dot_color);
        }
    }
}
