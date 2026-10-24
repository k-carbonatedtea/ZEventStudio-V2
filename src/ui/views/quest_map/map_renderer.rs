use gpui::*;
use crate::ui::views::quest_map::types::world_to_screen;

/// 渲染 2D 矢量海拉鲁全景地图底图、地形轮廓与分区网格
pub fn draw_hyrule_base_map(
    bounds: Bounds<Pixels>,
    center: (f32, f32),
    pan: (f32, f32),
    zoom: f32,
    window: &mut Window,
) {
    let w: f32 = bounds.size.width.into();
    let h: f32 = bounds.size.height.into();
    if w <= 0.0 || h <= 0.0 || zoom < 0.005 {
        return;
    }

    // 1. 绘制海拉鲁大陆多边形轮廓与海岸线
    draw_landmass_polygon(center, pan, zoom, window);

    // 2. 绘制主要地理特征 (死亡之山火山口、初始台地断崖、海利亚湖水系、海布拉雪峰)
    draw_geographic_features(center, pan, zoom, window);
}

/// 绘制海拉鲁主体大陆多边形与海岸线
fn draw_landmass_polygon(center: (f32, f32), pan: (f32, f32), zoom: f32, window: &mut Window) {
    let land_pts = [
        (4700.0, -3200.0), // 阿卡莱海角
        (3800.0, -3700.0), // 奥尔汀北麓
        (2600.0, -3850.0), // 死亡之山北壁
        (400.0, -3600.0),  // 德依布朗遗迹北
        (-1800.0, -3750.0), // 海布拉北麓
        (-3800.0, -3600.0), // 海布拉西北角
        (-4700.0, -2200.0), // 库库甲谷北
        (-4600.0, -600.0),  // 塔邦挞西崖
        (-4800.0, 1200.0),  // 格鲁德高地西壁
        (-4700.0, 3600.0),  // 格鲁德沙漠西南
        (-2200.0, 3800.0),  // 格鲁德沙漠东南
        (-400.0, 3700.0),   // 费罗尼海岬
        (1800.0, 3800.0),   // 沃托里海岸
        (4200.0, 3500.0),   // 哈特诺海角
        (4600.0, 1800.0),   // 哈特诺东海湾
        (4700.0, -600.0),   // 拉聂尔海湾
    ];

    let mut path = PathBuilder::fill();
    for (idx, (wx, wz)) in land_pts.iter().enumerate() {
        let (sx, sy) = world_to_screen(*wx, *wz, center, pan, zoom);
        if idx == 0 {
            path.move_to(point(px(sx), px(sy)));
        } else {
            path.line_to(point(px(sx), px(sy)));
        }
    }
    path.close();

    if let Ok(p) = path.build() {
        // 大陆深邃黑青蓝基底
        window.paint_path(p, rgba(0x131f33f0));
    }
}

/// 绘制主要地理特征与水体高地
fn draw_geographic_features(center: (f32, f32), pan: (f32, f32), zoom: f32, window: &mut Window) {
    // 1. 初始台地 (Great Plateau) 高耸断崖 (D-7 ~ E-7)
    let plateau_pts = [
        (-1200.0, 1450.0),
        (-400.0, 1450.0),
        (-350.0, 2150.0),
        (-1100.0, 2250.0),
    ];
    let mut plateau = PathBuilder::fill();
    for (idx, (wx, wz)) in plateau_pts.iter().enumerate() {
        let (sx, sy) = world_to_screen(*wx, *wz, center, pan, zoom);
        if idx == 0 {
            plateau.move_to(point(px(sx), px(sy)));
        } else {
            plateau.line_to(point(px(sx), px(sy)));
        }
    }
    plateau.close();
    if let Ok(p) = plateau.build() {
        window.paint_path(p, rgba(0x1d3655ee));
    }

    // 2. 海利亚湖水系 (Lake Hylia)
    let (lake_sx, lake_sy) = world_to_screen(-200.0, 2600.0, center, pan, zoom);
    let lake_rx = 480.0 * zoom;
    let lake_ry = 650.0 * zoom;
    if lake_rx > 2.0 && lake_ry > 2.0 {
        let mut lake = PathBuilder::fill();
        lake.move_to(point(px(lake_sx - lake_rx), px(lake_sy)));
        lake.line_to(point(px(lake_sx), px(lake_sy - lake_ry)));
        lake.line_to(point(px(lake_sx + lake_rx), px(lake_sy)));
        lake.line_to(point(px(lake_sx), px(lake_sy + lake_ry)));
        lake.close();
        if let Ok(p) = lake.build() {
            window.paint_path(p, rgba(0x0e7490cc));
        }
    }

    // 3. 死亡之山火山圈 (Death Mountain)
    let (dm_sx, dm_sy) = world_to_screen(2400.0, -2500.0, center, pan, zoom);
    let dm_r = 750.0 * zoom;
    if dm_r > 3.0 {
        let mut volcano = PathBuilder::fill();
        volcano.move_to(point(px(dm_sx - dm_r), px(dm_sy)));
        volcano.line_to(point(px(dm_sx), px(dm_sy - dm_r)));
        volcano.line_to(point(px(dm_sx + dm_r), px(dm_sy)));
        volcano.line_to(point(px(dm_sx), px(dm_sy + dm_r)));
        volcano.close();
        if let Ok(p) = volcano.build() {
            window.paint_path(p, rgba(0xc2410c55));
        }
    }

    // 4. 海布拉大雪山极寒雪原 (Hebra Mountains)
    let hebra_pts = [
        (-3800.0, -3200.0),
        (-2200.0, -3400.0),
        (-1800.0, -2000.0),
        (-3200.0, -1800.0),
    ];
    let mut hebra = PathBuilder::fill();
    for (idx, (wx, wz)) in hebra_pts.iter().enumerate() {
        let (sx, sy) = world_to_screen(*wx, *wz, center, pan, zoom);
        if idx == 0 {
            hebra.move_to(point(px(sx), px(sy)));
        } else {
            hebra.line_to(point(px(sx), px(sy)));
        }
    }
    hebra.close();
    if let Ok(p) = hebra.build() {
        window.paint_path(p, rgba(0x38bdf826));
    }

    // 5. 格鲁德大沙漠广袤沙海 (Gerudo Desert)
    let gerudo_pts = [
        (-4600.0, 1400.0),
        (-2600.0, 1800.0),
        (-1800.0, 3600.0),
        (-4400.0, 3500.0),
    ];
    let mut gerudo = PathBuilder::fill();
    for (idx, (wx, wz)) in gerudo_pts.iter().enumerate() {
        let (sx, sy) = world_to_screen(*wx, *wz, center, pan, zoom);
        if idx == 0 {
            gerudo.move_to(point(px(sx), px(sy)));
        } else {
            gerudo.line_to(point(px(sx), px(sy)));
        }
    }
    gerudo.close();
    if let Ok(p) = gerudo.build() {
        window.paint_path(p, rgba(0xf59e0b1f));
    }
}

/// 绘制 BotW 12x10 分区网格 (Z-0 至 K-9)
pub fn draw_section_grid(
    center: (f32, f32),
    pan: (f32, f32),
    zoom: f32,
    hovered_coord: Option<(f32, f32)>,
    window: &mut Window,
) {
    // 悬停分区高亮计算
    let hovered_tile = hovered_coord.and_then(|(wx, wz)| {
        let c = ((wx + 6000.0) / 1000.0).floor() as i32;
        let r = ((wz + 5000.0) / 1000.0).floor() as i32;
        if (0..12).contains(&c) && (0..10).contains(&r) {
            Some((c, r))
        } else {
            None
        }
    });

    // 绘制高亮选中的分区瓦片
    if let Some((hc, hr)) = hovered_tile {
        let wx0 = -6000.0 + (hc as f32) * 1000.0;
        let wz0 = -5000.0 + (hr as f32) * 1000.0;
        let (s_x0, s_y0) = world_to_screen(wx0, wz0, center, pan, zoom);
        let (s_x1, s_y1) = world_to_screen(wx0 + 1000.0, wz0 + 1000.0, center, pan, zoom);

        let mut h_box = PathBuilder::fill();
        h_box.move_to(point(px(s_x0), px(s_y0)));
        h_box.line_to(point(px(s_x1), px(s_y0)));
        h_box.line_to(point(px(s_x1), px(s_y1)));
        h_box.line_to(point(px(s_x0), px(s_y1)));
        h_box.close();
        if let Ok(p) = h_box.build() {
            window.paint_path(p, rgba(0x06b6d425));
        }
    }

    // 绘制网格竖线 (12 列：col 0..=12 对应世界 X -6000 到 +6000)
    let mut grid_lines = PathBuilder::fill();
    for col in 0..=12 {
        let wx = -6000.0 + (col as f32) * 1000.0;
        let (x_top, y_top) = world_to_screen(wx, -5000.0, center, pan, zoom);
        let (x_bot, y_bot) = world_to_screen(wx, 5000.0, center, pan, zoom);

        let th = if col == 0 || col == 12 { 1.5 } else { 0.75 };
        grid_lines.move_to(point(px(x_top - th), px(y_top)));
        grid_lines.line_to(point(px(x_top + th), px(y_top)));
        grid_lines.line_to(point(px(x_bot + th), px(y_bot)));
        grid_lines.line_to(point(px(x_bot - th), px(y_bot)));
        grid_lines.close();
    }

    // 绘制网格横线 (10 行：row 0..=10 对应世界 Z -5000 到 +5000)
    for row in 0..=10 {
        let wz = -5000.0 + (row as f32) * 1000.0;
        let (x_left, y_left) = world_to_screen(-6000.0, wz, center, pan, zoom);
        let (x_right, y_right) = world_to_screen(6000.0, wz, center, pan, zoom);

        let th = if row == 0 || row == 10 { 1.5 } else { 0.75 };
        grid_lines.move_to(point(px(x_left), px(y_left - th)));
        grid_lines.line_to(point(px(x_right), px(y_right - th)));
        grid_lines.line_to(point(px(x_right), px(y_right + th)));
        grid_lines.line_to(point(px(x_left), px(y_left + th)));
        grid_lines.close();
    }

    if let Ok(p) = grid_lines.build() {
        window.paint_path(p, rgba(0x38bdf82a));
    }
}
