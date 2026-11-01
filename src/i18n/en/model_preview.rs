//! 3D Model Preview (BFRES) English localization dictionary

pub fn get_en_model_preview(key: &str) -> Option<&'static str> {
    match key {
        "model_preview.mode_shaded" => Some("Shaded"),
        "model_preview.mode_wireframe" => Some("Wireframe"),
        "model_preview.mode_shaded_wire" => Some("Shaded Wire"),
        "model_preview.mode_points" => Some("Points"),

        "model_preview.toggle_grid" => Some("Grid"),
        "model_preview.toggle_axes" => Some("Axes"),
        "model_preview.toggle_bounds" => Some("Bounds"),
        "model_preview.toggle_stats" => Some("Stats"),

        "model_preview.view_front" => Some("Front"),
        "model_preview.view_side" => Some("Side"),
        "model_preview.view_top" => Some("Top"),
        "model_preview.view_iso" => Some("Iso"),
        "model_preview.fit_view" => Some("Fit View"),

        "model_preview.stat_meshes" => Some("Visible Meshes:"),
        "model_preview.stat_vertices" => Some("Total Vertices:"),
        "model_preview.stat_triangles" => Some("Total Triangles:"),
        "model_preview.stat_dimensions" => Some("Dimensions:"),
        "model_preview.stat_distance" => Some("Camera Distance:"),
        "model_preview.stat_textures" => Some("Textures:"),

        "model_preview.hint_orbit" => Some("• Left Drag: Orbit Camera"),
        "model_preview.hint_pan" => Some("• Right Drag: Pan Viewport"),
        "model_preview.hint_zoom" => Some("• Scroll: Zoom In / Out"),

        "model_preview.meshes_title" => Some("Submeshes"),
        "model_preview.filter_placeholder" => Some("Filter by name or material..."),
        "model_preview.no_model_selected" => Some("No 3D Model Selected"),
        "model_preview.window_title" => Some("3D Model Preview"),
        "model_preview.toggle_sidebar" => Some("Submeshes Panel"),
        "model_preview.select_all" => Some("Select All"),
        "model_preview.deselect_all" => Some("Deselect All"),
        _ => None,
    }
}

