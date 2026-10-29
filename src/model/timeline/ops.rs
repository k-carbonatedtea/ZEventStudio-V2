use revfl::actor::Actor;
use revfl::common::{ActorIdentifier, StringHolder};
use revfl::enums::TriggerType;
use revfl::timeline::{Clip, Cut, Timeline, Trigger};
use revfl::util::RequiredIndex;

/// 同步并重新生成官方规范要求的剪辑触发器 (Triggers)
/// 任天堂 BFEVFL 规范要求：每个剪辑严格对应两个 Trigger (Enter 和 Leave)
pub fn sync_triggers(timeline: &mut Timeline) {
    timeline.triggers.clear();
    for (i, clip) in timeline.clips.iter().enumerate() {
        timeline.triggers.push(Trigger {
            clip: RequiredIndex::new(i as u16),
            trigger_type: TriggerType::Enter,
            trigger_time: clip.start_time,
        });
        timeline.triggers.push(Trigger {
            clip: RequiredIndex::new(i as u16),
            trigger_type: TriggerType::Leave,
            trigger_time: clip.start_time + clip.duration,
        });
    }
}

/// 添加一个新剪辑 (Clip) 到指定角色
pub fn add_clip_to_timeline(
    timeline: &mut Timeline,
    actor_idx: usize,
    action_idx: usize,
    start_time: f32,
    duration: f32,
) -> usize {
    let mut clip = Clip::new();
    clip.start_time = start_time.max(0.0);
    clip.duration = duration.max(1.0);
    clip.actor = RequiredIndex::new(actor_idx as u16);
    clip.actor_action = RequiredIndex::new(action_idx as u16);
    clip.actor_concurrent_clip = 0;

    timeline.clips.push(clip);
    let new_idx = timeline.clips.len() - 1;

    // 自动扩展时间线总时长以容纳新剪辑
    let clip_end = start_time + duration;
    if clip_end > timeline.duration {
        timeline.duration = clip_end;
    }

    sync_triggers(timeline);
    new_idx
}

/// 移除指定索引的剪辑
pub fn remove_clip_from_timeline(timeline: &mut Timeline, clip_idx: usize) {
    if clip_idx < timeline.clips.len() {
        timeline.clips.remove(clip_idx);
        sync_triggers(timeline);
    }
}

/// 复制指定剪辑并偏移放置
pub fn duplicate_clip_in_timeline(timeline: &mut Timeline, clip_idx: usize) -> Option<usize> {
    if clip_idx >= timeline.clips.len() {
        return None;
    }
    let mut cloned = timeline.clips[clip_idx].clone();
    cloned.start_time += cloned.duration; // 紧随其后
    timeline.clips.push(cloned);
    let new_idx = timeline.clips.len() - 1;

    let end_time = timeline.clips[new_idx].start_time + timeline.clips[new_idx].duration;
    if end_time > timeline.duration {
        timeline.duration = end_time;
    }

    sync_triggers(timeline);
    Some(new_idx)
}

/// 平移剪辑起始时间
pub fn move_clip_start(timeline: &mut Timeline, clip_idx: usize, new_start: f32) {
    if let Some(clip) = timeline.clips.get_mut(clip_idx) {
        clip.start_time = new_start.max(0.0);
        let end_time = clip.start_time + clip.duration;
        if end_time > timeline.duration {
            timeline.duration = end_time;
        }
    }
    sync_triggers(timeline);
}

/// 调整剪辑时间区间 (起始帧与时长)
pub fn resize_clip(timeline: &mut Timeline, clip_idx: usize, new_start: f32, new_duration: f32) {
    if let Some(clip) = timeline.clips.get_mut(clip_idx) {
        clip.start_time = new_start.max(0.0);
        clip.duration = new_duration.max(1.0);
        let end_time = clip.start_time + clip.duration;
        if end_time > timeline.duration {
            timeline.duration = end_time;
        }
    }
    sync_triggers(timeline);
}

/// 向时间线添加新角色 (Actor)
pub fn add_actor_to_timeline(
    timeline: &mut Timeline,
    name: String,
    secondary_name: String,
    initial_actions: Vec<String>,
) -> usize {
    let mut actor = Actor::new();
    actor.identifier = ActorIdentifier {
        name,
        sub_name: secondary_name,
    };
    actor.actions = initial_actions
        .into_iter()
        .map(StringHolder)
        .collect();
    timeline.actors.push(actor);
    timeline.actors.len() - 1
}

/// 为已有角色追加新动作名
#[allow(dead_code)]
pub fn add_action_to_actor(timeline: &mut Timeline, actor_idx: usize, action_name: String) -> usize {
    if let Some(actor) = timeline.actors.get_mut(actor_idx) {
        if let Some(existing) = actor.actions.iter().position(|a| a.0 == action_name) {
            return existing;
        }
        actor.actions.push(StringHolder(action_name));
        actor.actions.len() - 1
    } else {
        0
    }
}

/// 删除指定角色及其所有关联剪辑
pub fn remove_actor_from_timeline(timeline: &mut Timeline, actor_idx: usize) {
    if actor_idx >= timeline.actors.len() {
        return;
    }
    timeline.actors.remove(actor_idx);

    // 删除关联该角色的剪辑，并更新其余剪辑引用的 actor 索引
    let mut i = 0;
    while i < timeline.clips.len() {
        let cur_actor = timeline.clips[i].actor.idx as usize;
        if cur_actor == actor_idx {
            timeline.clips.remove(i);
        } else {
            if cur_actor > actor_idx {
                timeline.clips[i].actor.idx -= 1;
            }
            i += 1;
        }
    }

    // 同样处理 oneshots
    let mut j = 0;
    while j < timeline.oneshots.len() {
        let cur_actor = timeline.oneshots[j].actor.idx as usize;
        if cur_actor == actor_idx {
            timeline.oneshots.remove(j);
        } else {
            if cur_actor > actor_idx {
                timeline.oneshots[j].actor.idx -= 1;
            }
            j += 1;
        }
    }

    sync_triggers(timeline);
}

/// 添加分镜镜头标记 (Cut)
pub fn add_cut_to_timeline(timeline: &mut Timeline, name: String, start_time: f32) -> usize {
    let mut cut = Cut::new();
    cut.name = name;
    cut.start_time = start_time.max(0.0);
    timeline.cuts.push(cut);
    timeline.cuts.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap_or(std::cmp::Ordering::Equal));
    timeline.cuts.iter().position(|c| c.start_time == start_time).unwrap_or(0)
}

/// 移除分镜镜头标记
pub fn remove_cut_from_timeline(timeline: &mut Timeline, cut_idx: usize) {
    if cut_idx < timeline.cuts.len() {
        timeline.cuts.remove(cut_idx);
    }
}
