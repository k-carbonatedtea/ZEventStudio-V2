use crate::model::{auto_layout_graph, Edge, Graph, NodeData, NodeKind, Port};

pub fn create_signal_math_preset() -> Graph {
    let mut g = Graph::new("🧮 信号处理与数值计算流 (Signal & Math Pipeline)");
    g.description = "实时数据流计算：多源传感器输入、加法器、增益乘法器、阈值比较路由与目标执行器输出。".to_string();

    let n_in_a = NodeData::new("sig_a", "传感器输入 A (Sensor_Pitch)", NodeKind::Math, (0.0, 0.0))
        .with_subtitle("Gyroscope X-Axis")
        .with_inputs(vec![])
        .with_outputs(vec![Port::new_data_out("sig_a_out", "Float: Pitch")])
        .with_property("Channel", "IMU_Gyro_X", "string")
        .with_property("SampleRate", "100Hz", "number");

    let n_in_b = NodeData::new("sig_b", "基准偏移量 B (Offset_Bias)", NodeKind::Math, (0.0, 0.0))
        .with_subtitle("Constant Bias")
        .with_inputs(vec![])
        .with_outputs(vec![Port::new_data_out("sig_b_out", "Float: Offset")])
        .with_property("Value", "+1.25", "number");

    let n_add = NodeData::new("math_add", "加法运算器 (Add_A_B)", NodeKind::Math, (0.0, 0.0))
        .with_subtitle("Formula: A + B")
        .with_inputs(vec![
            Port::new_data_in("add_a", "In A"),
            Port::new_data_in("add_b", "In B"),
        ])
        .with_outputs(vec![Port::new_data_out("add_res", "Sum Out")]);

    let n_gain = NodeData::new("math_gain", "增益乘法器 (GainMultiplier)", NodeKind::Math, (0.0, 0.0))
        .with_subtitle("Factor: 2.5x")
        .with_inputs(vec![Port::new_data_in("gain_in", "Signal In")])
        .with_outputs(vec![Port::new_data_out("gain_out", "Amplified Out")])
        .with_property("GainFactor", "2.5", "number");

    let n_switch = NodeData::new("sw_thresh", "阈值比较分流 (ThresholdCheck)", NodeKind::Switch, (0.0, 0.0))
        .with_subtitle("Compare: Val >= 10.0")
        .with_inputs(vec![Port::new_flow_in("sw_in", "Eval In")])
        .with_outputs(vec![
            Port::new_condition_out("sw_high", "High (≥ 10.0): 启动冷却"),
            Port::new_condition_out("sw_normal", "Normal (< 10.0): 正常维持"),
        ]);

    let n_act_cool = NodeData::new("act_cool", "开启散热风扇 (ActivateCooler)", NodeKind::Action, (0.0, 0.0))
        .with_subtitle("PWM Duty: 100%")
        .with_property("FanSpeed", "4500 RPM", "string");

    let n_act_norm = NodeData::new("act_norm", "维持标准功耗 (StandardMode)", NodeKind::Action, (0.0, 0.0))
        .with_subtitle("PWM Duty: 40%")
        .with_property("FanSpeed", "1800 RPM", "string");

    g.add_node(n_in_a);
    g.add_node(n_in_b);
    g.add_node(n_add);
    g.add_node(n_gain);
    g.add_node(n_switch);
    g.add_node(n_act_cool);
    g.add_node(n_act_norm);

    g.add_edge(Edge::new("sig_a", "sig_a_out", "math_add", "add_a"));
    g.add_edge(Edge::new("sig_b", "sig_b_out", "math_add", "add_b"));
    g.add_edge(Edge::new("math_add", "add_res", "math_gain", "gain_in"));
    g.add_edge(Edge::new("math_gain", "gain_out", "sw_thresh", "sw_in"));

    g.add_edge(Edge::new("sw_thresh", "sw_high", "act_cool", "act_cool_in").with_label("≥ 10.0"));
    g.add_edge(Edge::new("sw_thresh", "sw_normal", "act_norm", "act_norm_in").with_label("< 10.0"));

    auto_layout_graph(&mut g);
    g
}
