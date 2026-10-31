use gpui_component::highlighter::{HighlightTheme, SyntaxHighlighter};
use gpui_component::input::{Rope, RopeExt};

#[test]
fn test_yaml_above_32768_lines_highlighting() {
    let mut yaml = String::new();
    yaml.push_str("root:\n");
    // Test large file like ActorInfo.product.sbyml (190,000+ lines)
    for i in 1..200000 {
        yaml.push_str(&format!("  key_{}: {}\n", i, i));
    }
    let rope = Rope::from_str(&yaml);
    let mut highlighter = SyntaxHighlighter::new("yaml");
    highlighter.update(None, &rope, None);

    let tree = highlighter.tree().unwrap();
    assert!(!tree.root_node().has_error(), "Parsed tree must not have syntax errors");

    let theme = HighlightTheme::default_dark();

    // Verify lines across various thresholds: 32768, 65536, 131072, 190755
    for line_idx in [32767, 32768, 32769, 65535, 65536, 131072, 190755] {
        let line_start = rope.line_start_offset(line_idx);
        let line_end = rope.line_start_offset(line_idx + 1);
        let styles = highlighter.styles(&(line_start..line_end), &*theme);

        println!("Line {} styles count: {}", line_idx, styles.len());
        let colored_spans = styles.iter().filter(|(_, s)| s.color.is_some()).count();
        assert!(
            colored_spans >= 2,
            "Line {} must have at least 2 colored spans (key and value), found {}",
            line_idx,
            colored_spans
        );

        let node = tree.root_node().descendant_for_byte_range(line_start + 2, line_end - 1);
        assert!(node.is_some(), "Node must exist at line {}", line_idx);
        let node_kind = node.unwrap().kind();
        assert_ne!(node_kind, "ERROR", "Line {} node cannot be ERROR", line_idx);
        assert_ne!(node_kind, "stream", "Line {} node cannot fall back to stream", line_idx);
    }
}
