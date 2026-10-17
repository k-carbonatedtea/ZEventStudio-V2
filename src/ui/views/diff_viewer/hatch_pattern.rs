use gpui::*;

/// 渲染 45° 对角斜纹阴影占位区域 (IDE 经典的 Empty Line Diagonal Hatch Pattern)
/// 严格复刻图2中新增行对应侧的斜纹阴影视觉效果
pub fn render_diagonal_hatch_placeholder(
    height: Pixels,
    theme: &crate::ui::theme::ThemePalette,
) -> impl IntoElement {
    let stroke_color = if theme.is_dark {
        rgba(0xffffff18) // 微弱透光白斜纹 (图2质感)
    } else {
        rgba(0x0000001a) // 微弱透光灰斜纹
    };
    let bg_color = if theme.is_dark {
        rgba(0x00000030) // 深色暗背景
    } else {
        rgba(0x0000000c)
    };

    div()
        .w_full()
        .h(height)
        .bg(bg_color)
        .overflow_hidden()
        .child(
            canvas(
                move |bounds, _, _| bounds,
                move |bounds, _, window, _| {
                    let orig_x: f32 = bounds.origin.x.into();
                    let orig_y: f32 = bounds.origin.y.into();
                    let w: f32 = bounds.size.width.into();
                    let h: f32 = bounds.size.height.into();

                    if w <= 0.0 || h <= 0.0 {
                        return;
                    }

                    let step: f32 = 8.0; // 45° 斜线条间距 (像素)
                    let mut builder = PathBuilder::stroke(px(1.0));

                    // 统一对齐到全局网格相位，使跨多行的连续占位区斜线完美连贯
                    let phase = ((orig_x - orig_y) % step + step) % step;
                    let mut offset = -h + phase;

                    while offset <= w {
                        let (p1_x, p1_y) = if offset >= 0.0 {
                            (orig_x + offset, orig_y)
                        } else {
                            (orig_x, orig_y - offset)
                        };

                        let (p2_x, p2_y) = if h + offset <= w {
                            (orig_x + h + offset, orig_y + h)
                        } else {
                            (orig_x + w, orig_y + w - offset)
                        };

                        if (p1_x - p2_x).abs() > 0.1 || (p1_y - p2_y).abs() > 0.1 {
                            builder.move_to(point(px(p1_x), px(p1_y)));
                            builder.line_to(point(px(p2_x), px(p2_y)));
                        }

                        offset += step;
                    }

                    if let Ok(path) = builder.build() {
                        window.paint_path(path, stroke_color);
                    }
                },
            )
            .size_full(),
        )
}
