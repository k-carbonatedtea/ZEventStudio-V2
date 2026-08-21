use crate::model::evfl::load_bfevfl_from_bytes;
use crate::model::layout::auto_layout_graph;
use crate::model::Graph;

pub fn create_zelda_100enemy_preset() -> Graph {
    let bfevfl_bytes = include_bytes!("../../100enemy.bfevfl");
    match load_bfevfl_from_bytes(bfevfl_bytes) {
        Ok(mut graph) => {
            graph.name = "🗡️ 100enemy.bfevfl (Revfl 原生二进制解析)".to_string();
            auto_layout_graph(&mut graph);
            graph
        }
        Err(err) => {
            eprintln!("Failed to load 100enemy.bfevfl: {}", err);
            Graph::new("100enemy.bfevfl (加载失败)")
        }
    }
}
