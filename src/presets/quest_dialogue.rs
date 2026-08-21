use crate::model::{auto_layout_graph, Edge, Graph, NodeData, NodeKind, Port};

pub fn create_quest_dialogue_preset() -> Graph {
    let mut g = Graph::new("🎮 NPC 任务对话与条件分支事件流 (Quest & Dialogue Flow)");
    g.description = "经典 RPG 任务对话流：包含事件入口、状态查询、三路条件分支、并行音效/道具分流、汇合点与子流程自动存档。".to_string();

    // 1. Entry Point
    let n_entry = NodeData::new("ep_talk", "任务触发入口 (OnTalkToNPC)", NodeKind::EntryPoint, (0.0, 0.0))
        .with_subtitle("Event Trigger: Talk_Npc_Zora001")
        .with_property("Actor", "Npc_Zora001", "actor")
        .with_property("TriggerRadius", "2.5m", "number");

    // 2. Query
    let n_query = NodeData::new("q_state", "检查任务状态 (CheckQuestState)", NodeKind::Query, (0.0, 0.0))
        .with_subtitle("Actor: QuestManager")
        .with_property("Actor", "QuestManager", "actor")
        .with_property("QueryFunc", "GetQuestState", "string")
        .with_property("QuestId", "LostMasterSword", "string");

    // 3. Switch (3 branches)
    let n_switch = NodeData::new("sw_branch", "任务进度分流 (BranchOnState)", NodeKind::Switch, (0.0, 0.0))
        .with_subtitle("Condition: QuestState")
        .with_property("Variable", "QuestState", "string")
        .with_outputs(vec![
            Port::new_condition_out("sw_case0", "Case 0: 未接取 (NotStarted)"),
            Port::new_condition_out("sw_case1", "Case 1: 进行中 (InProgress)"),
            Port::new_condition_out("sw_case2", "Case 2: 已完成 (Completed)"),
        ]);

    // 4. Branch 0: Not Started -> Talk Intro -> Accept Quest
    let n_talk_intro = NodeData::new("act_talk_intro", "NPC 对话: 委托 (NPC_Talk)", NodeKind::Action, (0.0, 0.0))
        .with_subtitle("Actor: Npc_Zora001")
        .with_property("Actor", "Npc_Zora001", "actor")
        .with_property("Action", "Talk", "string")
        .with_property("MessageId", "Talk_Quest_Intro_01", "event")
        .with_property("Text", "海拉鲁勇者啊！你能帮我们在北边神庙附近找回遗失的誓约之剑吗？", "string");

    let n_accept = NodeData::new("act_accept", "接取任务 (AcceptQuest)", NodeKind::Action, (0.0, 0.0))
        .with_subtitle("Actor: QuestManager")
        .with_property("Actor", "QuestManager", "actor")
        .with_property("Action", "SetState", "string")
        .with_property("QuestId", "LostMasterSword", "string")
        .with_property("NewState", "Active", "string");

    // 5. Branch 1: In Progress -> Hint Talk
    let n_talk_hint = NodeData::new("act_talk_hint", "NPC 对话: 提示 (NPC_Talk)", NodeKind::Action, (0.0, 0.0))
        .with_subtitle("Actor: Npc_Zora001")
        .with_property("Actor", "Npc_Zora001", "actor")
        .with_property("Action", "Talk", "string")
        .with_property("MessageId", "Talk_Quest_Hint_01", "event")
        .with_property("Text", "你还在寻找那把誓约之剑吗？据说它闪耀着微蓝色的光芒。", "string");

    // 6. Branch 2: Completed -> Talk Reward -> Fork -> (GiveItem + PlaySound) -> Join -> SubFlow (AutoSave) -> CloseQuest
    let n_talk_reward = NodeData::new("act_talk_reward", "NPC 对话: 致谢 (NPC_Talk)", NodeKind::Action, (0.0, 0.0))
        .with_subtitle("Actor: Npc_Zora001")
        .with_property("Actor", "Npc_Zora001", "actor")
        .with_property("Action", "Talk", "string")
        .with_property("MessageId", "Talk_Quest_Reward_01", "event")
        .with_property("Text", "太感谢你了！誓约之剑已归位，这是属于你的谢礼！", "string");

    let n_fork = NodeData::new("fork_reward", "并行发放分流 (ForkRewards)", NodeKind::Fork, (0.0, 0.0))
        .with_subtitle("Async Parallel Split")
        .with_outputs(vec![
            Port::new_flow_out("fork_out0", "Branch A (Item)"),
            Port::new_flow_out("fork_out1", "Branch B (BGM)"),
        ]);

    let n_give_item = NodeData::new("act_give_item", "获得道具奖励 (GiveItem)", NodeKind::Action, (0.0, 0.0))
        .with_subtitle("Actor: InventoryManager")
        .with_property("Actor", "InventoryManager", "actor")
        .with_property("Action", "AddItem", "string")
        .with_property("ItemName", "MasterOre_Rare", "string")
        .with_property("Quantity", "3", "number");

    let n_play_bgm = NodeData::new("act_play_bgm", "播放任务通关音效 (PlayBGM)", NodeKind::Action, (0.0, 0.0))
        .with_subtitle("Actor: SoundSystem")
        .with_property("Actor", "SoundSystem", "actor")
        .with_property("Action", "PlayFanfare", "string")
        .with_property("BGM_Key", "Fanfare_QuestSuccess", "string")
        .with_property("Volume", "1.0", "number");

    let n_join = NodeData::new("join_reward", "汇合同步点 (JoinRewards)", NodeKind::Join, (0.0, 0.0))
        .with_subtitle("Sync Parallel Execution")
        .with_inputs(vec![
            Port::new_flow_in("join_in0", "Sync A"),
            Port::new_flow_in("join_in1", "Sync B"),
        ]);

    let n_subflow = NodeData::new("sub_save", "执行自动存档 (SubFlow: Save)", NodeKind::SubFlow, (0.0, 0.0))
        .with_subtitle("SubFlow: System_AutoSave")
        .with_property("SubFlowName", "System_AutoSave", "string")
        .with_property("SaveSlot", "AutoSlot_01", "string");

    let n_close_quest = NodeData::new("act_close", "结算关闭任务 (CompleteQuest)", NodeKind::Action, (0.0, 0.0))
        .with_subtitle("Actor: QuestManager")
        .with_property("Actor", "QuestManager", "actor")
        .with_property("Action", "CloseQuest", "string")
        .with_property("QuestId", "LostMasterSword", "string");

    // Add all nodes
    g.add_node(n_entry);
    g.add_node(n_query);
    g.add_node(n_switch);
    g.add_node(n_talk_intro);
    g.add_node(n_accept);
    g.add_node(n_talk_hint);
    g.add_node(n_talk_reward);
    g.add_node(n_fork);
    g.add_node(n_give_item);
    g.add_node(n_play_bgm);
    g.add_node(n_join);
    g.add_node(n_subflow);
    g.add_node(n_close_quest);

    // Connect all edges
    g.add_edge(Edge::new("ep_talk", "ep_talk_out", "q_state", "q_state_in"));
    g.add_edge(Edge::new("q_state", "q_state_out", "sw_branch", "sw_branch_in"));

    // Switch -> Cases
    g.add_edge(Edge::new("sw_branch", "sw_case0", "act_talk_intro", "act_talk_intro_in").with_label("Case 0: 未接取"));
    g.add_edge(Edge::new("act_talk_intro", "act_talk_intro_out", "act_accept", "act_accept_in"));

    g.add_edge(Edge::new("sw_branch", "sw_case1", "act_talk_hint", "act_talk_hint_in").with_label("Case 1: 进行中"));

    g.add_edge(Edge::new("sw_branch", "sw_case2", "act_talk_reward", "act_talk_reward_in").with_label("Case 2: 已完成"));
    g.add_edge(Edge::new("act_talk_reward", "act_talk_reward_out", "fork_reward", "fork_reward_in"));

    // Fork -> Parallel Branches -> Join
    g.add_edge(Edge::new("fork_reward", "fork_out0", "act_give_item", "act_give_item_in"));
    g.add_edge(Edge::new("fork_reward", "fork_out1", "act_play_bgm", "act_play_bgm_in"));

    g.add_edge(Edge::new("act_give_item", "act_give_item_out", "join_reward", "join_in0"));
    g.add_edge(Edge::new("act_play_bgm", "act_play_bgm_out", "join_reward", "join_in1"));

    // Join -> SubFlow -> Close
    g.add_edge(Edge::new("join_reward", "join_reward_out", "sub_save", "sub_save_in"));
    g.add_edge(Edge::new("sub_save", "sub_save_out", "act_close", "act_close_in"));

    // Run DAG automatic hierarchical layout
    auto_layout_graph(&mut g);

    g
}
