use crate::model::{auto_layout_graph, Edge, Graph, NodeData, NodeKind, Port};

pub fn create_combat_ai_preset() -> Graph {
    let mut g = Graph::new("⚔️ 怪物战斗 AI 决策流 (Combat AI Behavior Flow)");
    g.description = "敌人 AI 状态决策机：巡逻巡回、感知玩家、距离条件路由、近战/远程技能释放与低血量战术逃跑。".to_string();

    let n_entry = NodeData::new("ai_tick", "AI 决策时钟 (OnAiTick)", NodeKind::EntryPoint, (0.0, 0.0))
        .with_subtitle("Tick Rate: 30Hz")
        .with_property("Actor", "Enemy_Lynel_01", "actor")
        .with_property("IntervalMs", "33", "number");

    let n_query_detect = NodeData::new("q_detect", "索敌感知检查 (DetectTarget)", NodeKind::Query, (0.0, 0.0))
        .with_subtitle("Actor: PerceptionSystem")
        .with_property("PerceptionType", "SightAndHearing", "string")
        .with_property("MaxRange", "35.0m", "number");

    let n_sw_target = NodeData::new("sw_target", "是否发现目标 (HasTarget?)", NodeKind::Switch, (0.0, 0.0))
        .with_subtitle("Condition: TargetFound")
        .with_outputs(vec![
            Port::new_condition_out("sw_lost", "False: 无目标 (Patrol)"),
            Port::new_condition_out("sw_found", "True: 发现玩家 (Combat)"),
        ]);

    let n_act_patrol = NodeData::new("act_patrol", "执行路径巡逻 (PatrolRoute)", NodeKind::Action, (0.0, 0.0))
        .with_subtitle("Actor: NavigationSystem")
        .with_property("RouteId", "Path_NorthGate", "string")
        .with_property("MoveSpeed", "2.0m/s", "number");

    let n_query_dist = NodeData::new("q_dist", "评估目标距离与血量 (EvaluateCombat)", NodeKind::Query, (0.0, 0.0))
        .with_subtitle("Actor: CombatCalculator")
        .with_property("Target", "Player", "actor");

    let n_sw_combat = NodeData::new("sw_combat", "战斗策略分流 (CombatBranch)", NodeKind::Switch, (0.0, 0.0))
        .with_subtitle("Condition: Range & Health")
        .with_outputs(vec![
            Port::new_condition_out("sw_melee", "近距离 (< 5m): 挥刀重击"),
            Port::new_condition_out("sw_ranged", "远距离 (≥ 5m): 射击火箭"),
            Port::new_condition_out("sw_flee", "残血 (HP < 20%): 战术后撤"),
        ]);

    let n_act_melee = NodeData::new("act_melee", "发动蓄力斩击 (MeleeHeavyAttack)", NodeKind::Action, (0.0, 0.0))
        .with_subtitle("Damage: 85")
        .with_property("Animation", "Anim_SlashHeavy", "string")
        .with_property("Knockback", "12.0", "number");

    let n_act_ranged = NodeData::new("act_ranged", "三连弓射击 (ShootTripleFireArrow)", NodeKind::Action, (0.0, 0.0))
        .with_subtitle("Damage: 45 x 3")
        .with_property("Projectile", "Arrow_Fire", "string")
        .with_property("Count", "3", "number");

    let n_act_flee = NodeData::new("act_flee", "投掷烟雾弹撤退 (TacticalRetreat)", NodeKind::Action, (0.0, 0.0))
        .with_subtitle("Defensive Skill")
        .with_property("Item", "SmokeBomb", "string")
        .with_property("RetreatDistance", "25m", "number");

    g.add_node(n_entry);
    g.add_node(n_query_detect);
    g.add_node(n_sw_target);
    g.add_node(n_act_patrol);
    g.add_node(n_query_dist);
    g.add_node(n_sw_combat);
    g.add_node(n_act_melee);
    g.add_node(n_act_ranged);
    g.add_node(n_act_flee);

    g.add_edge(Edge::new("ai_tick", "ai_tick_out", "q_detect", "q_detect_in"));
    g.add_edge(Edge::new("q_detect", "q_detect_out", "sw_target", "sw_target_in"));

    g.add_edge(Edge::new("sw_target", "sw_lost", "act_patrol", "act_patrol_in").with_label("False: 巡逻"));
    g.add_edge(Edge::new("sw_target", "sw_found", "q_dist", "q_dist_in").with_label("True: 交战"));

    g.add_edge(Edge::new("q_dist", "q_dist_out", "sw_combat", "sw_combat_in"));
    g.add_edge(Edge::new("sw_combat", "sw_melee", "act_melee", "act_melee_in").with_label("近距离 (<5m)"));
    g.add_edge(Edge::new("sw_combat", "sw_ranged", "act_ranged", "act_ranged_in").with_label("远距离 (≥5m)"));
    g.add_edge(Edge::new("sw_combat", "sw_flee", "act_flee", "act_flee_in").with_label("残血 (<20%)"));

    auto_layout_graph(&mut g);
    g
}
