use crate::model::evfl::load_evfl_from_str;
use crate::model::layout::auto_layout_graph;
use crate::model::Graph;

pub fn create_zelda_100enemy_preset() -> Graph {
    let json_str = include_str!("../../100enemy.json");
    match load_evfl_from_str(json_str) {
        Ok(mut graph) => {
            graph.name = "🗡️ 100enemy (百人斩/剑之试炼真实事件流)".to_string();
            auto_layout_graph(&mut graph);
            graph
        }
        Err(err) => {
            eprintln!("Failed to load 100enemy.json: {}", err);
            Graph::new("100enemy (加载失败)")
        }
    }
}
