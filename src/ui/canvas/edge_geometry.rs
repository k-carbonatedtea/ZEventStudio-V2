use gpui::*;
use crate::model::graph::Graph;
use crate::model::node::NodeKind;

pub type EdgeItem = (
    String,
    String,
    Option<String>,
    NodeKind,
    bool,
    (f32, f32),
    (f32, f32),
    Vec<(f32, f32)>,
);

/// 提取图谱中所有边的数据
pub fn extract_edge_items(graph: &Graph) -> Vec<EdgeItem> {
    graph
        .edges
        .iter()
        .filter_map(|edge| {
            let s_node = graph.find_node(&edge.source_node_id)?;
            let t_node = graph.find_node(&edge.target_node_id)?;

            let is_dashed = t_node.kind == NodeKind::Join
                && graph.edges.iter().filter(|e| e.target_node_id == t_node.id).count() > 1;

            let s_port_offset = s_node.get_port_offset(&edge.source_port_id, crate::model::PortDirection::Output);
            let t_port_offset = t_node.get_port_offset(&edge.target_port_id, crate::model::PortDirection::Input);

            let sx = s_node.pos.0 + s_port_offset.0;
            let sy = s_node.pos.1 + s_port_offset.1;
            let ex = t_node.pos.0 + t_port_offset.0;
            let ey = t_node.pos.1 + t_port_offset.1;

            Some((
                edge.source_node_id.clone(),
                edge.target_node_id.clone(),
                edge.label.clone(),
                s_node.kind.clone(),
                is_dashed,
                (sx, sy),
                (ex, ey),
                edge.waypoints.clone(),
            ))
        })
        .collect()
}

/// 绘制带圆角过渡的正交折线 / 阶梯回路连线 (Orthogonal Manhattan / Step Line with Rounded Chamfers)
pub fn draw_orthogonal_step_line(builder: &mut PathBuilder, pts: &[(f32, f32)], zoom: f32) {
    if pts.is_empty() {
        return;
    }
    builder.move_to(point(px(pts[0].0), px(pts[0].1)));

    if pts.len() == 2 {
        let (sx, sy) = pts[0];
        let (ex, ey) = pts[1];

        // 1. 同 X 轴垂直纯直线：直接一笔直落
        if (sx - ex).abs() < 1.5 {
            builder.line_to(point(px(ex), px(ey)));
            return;
        }

        // 2. 正向下行正交折线 (Vertical -> Horizontal -> Vertical)
        if ey >= sy + 12.0 * zoom {
            let mid_y = (sy + ey) / 2.0;
            let dx = ex - sx;
            let dir_x = if dx > 0.0 { 1.0 } else { -1.0 };
            let r = ((ey - sy) * 0.35).min(dx.abs() * 0.45).min(10.0 * zoom).max(0.0);

            builder.line_to(point(px(sx), px(mid_y - r)));
            builder.cubic_bezier_to(
                point(px(sx + dir_x * r), px(mid_y)),
                point(px(sx), px(mid_y)),
                point(px(sx + dir_x * r * 0.55), px(mid_y)),
            );
            builder.line_to(point(px(ex - dir_x * r), px(mid_y)));
            builder.cubic_bezier_to(
                point(px(ex), px(mid_y + r)),
                point(px(ex), px(mid_y)),
                point(px(ex), px(mid_y + r * 0.55)),
            );
            builder.line_to(point(px(ex), px(ey)));
        } else {
            // 3. 反向/侧向回路正交绕行
            let side_offset = if ex >= sx { 48.0 * zoom } else { -48.0 * zoom };
            let side_x = (if ex >= sx { sx.max(ex) } else { sx.min(ex) }) + side_offset;
            let r = (8.0 * zoom).clamp(0.0, 10.0);
            let dir_out = if side_x > sx { 1.0 } else { -1.0 };

            let drop_y = sy + 24.0 * zoom;
            let enter_y = ey - 24.0 * zoom;

            builder.line_to(point(px(sx), px(drop_y - r)));
            builder.cubic_bezier_to(
                point(px(sx + dir_out * r), px(drop_y)),
                point(px(sx), px(drop_y)),
                point(px(sx + dir_out * r * 0.55), px(drop_y)),
            );
            builder.line_to(point(px(side_x - dir_out * r), px(drop_y)));
            builder.cubic_bezier_to(
                point(px(side_x), px(drop_y + (if enter_y >= drop_y { r } else { -r }))),
                point(px(side_x), px(drop_y)),
                point(px(side_x), px(drop_y + (if enter_y >= drop_y { r * 0.55 } else { -r * 0.55 }))),
            );
            builder.line_to(point(px(side_x), px(enter_y - (if enter_y >= drop_y { r } else { -r }))));
            builder.cubic_bezier_to(
                point(px(side_x - dir_out * r), px(enter_y)),
                point(px(side_x), px(enter_y)),
                point(px(side_x - dir_out * r * 0.55), px(enter_y)),
            );
            builder.line_to(point(px(ex + dir_out * r), px(enter_y)));
            builder.cubic_bezier_to(
                point(px(ex), px(enter_y + r)),
                point(px(ex), px(enter_y)),
                point(px(ex), px(enter_y + r * 0.55)),
            );
            builder.line_to(point(px(ex), px(ey)));
        }
    } else {
        // 多航点正交折线
        for i in 0..pts.len() - 1 {
            let (x1, y1) = pts[i];
            let (x2, y2) = pts[i + 1];
            if (x1 - x2).abs() < 1.5 {
                builder.line_to(point(px(x2), px(y2)));
            } else {
                let mid_y = (y1 + y2) / 2.0;
                let dx = x2 - x1;
                let dir_x = if dx > 0.0 { 1.0 } else { -1.0 };
                let r = ((y2 - y1).abs() * 0.35).min(dx.abs() * 0.45).min(8.0 * zoom).max(0.0);
                builder.line_to(point(px(x1), px(mid_y - r)));
                builder.cubic_bezier_to(
                    point(px(x1 + dir_x * r), px(mid_y)),
                    point(px(x1), px(mid_y)),
                    point(px(x1 + dir_x * r * 0.55), px(mid_y)),
                );
                builder.line_to(point(px(x2 - dir_x * r), px(mid_y)));
                builder.cubic_bezier_to(
                    point(px(x2), px(mid_y + (if y2 >= y1 { r } else { -r }))),
                    point(px(x2), px(mid_y)),
                    point(px(x2), px(mid_y + (if y2 >= y1 { r * 0.55 } else { -r * 0.55 }))),
                );
                builder.line_to(point(px(x2), px(y2)));
            }
        }
    }
}
