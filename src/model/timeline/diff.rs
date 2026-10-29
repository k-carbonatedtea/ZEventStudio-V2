use revfl::timeline::Timeline;

/// 深度比对当前时间线与原始快照是否发生了实质性变动
pub fn is_timeline_modified(current: &Timeline, original: &Timeline) -> bool {
    // 1. 基础属性比对
    if current.name != original.name {
        return true;
    }
    if (current.duration - original.duration).abs() > 0.001 {
        return true;
    }

    // 2. 数量比对
    if current.actors.len() != original.actors.len()
        || current.clips.len() != original.clips.len()
        || current.cuts.len() != original.cuts.len()
        || current.oneshots.len() != original.oneshots.len()
        || current.subtimelines.len() != original.subtimelines.len()
    {
        return true;
    }

    // 3. 角色比对
    for (curr_act, orig_act) in current.actors.iter().zip(original.actors.iter()) {
        if curr_act.identifier.name != orig_act.identifier.name
            || curr_act.identifier.sub_name != orig_act.identifier.sub_name
            || curr_act.argument_name != orig_act.argument_name
            || curr_act.actions.len() != orig_act.actions.len()
        {
            return true;
        }
        for (ca, oa) in curr_act.actions.iter().zip(orig_act.actions.iter()) {
            if ca.0 != oa.0 {
                return true;
            }
        }
    }

    // 4. 剪辑比对
    for (curr_clip, orig_clip) in current.clips.iter().zip(original.clips.iter()) {
        if (curr_clip.start_time - orig_clip.start_time).abs() > 0.001
            || (curr_clip.duration - orig_clip.duration).abs() > 0.001
            || curr_clip.actor.idx != orig_clip.actor.idx
            || curr_clip.actor_action.idx != orig_clip.actor_action.idx
            || curr_clip.actor_concurrent_clip != orig_clip.actor_concurrent_clip
        {
            return true;
        }
    }

    // 5. 分镜比对
    for (curr_cut, orig_cut) in current.cuts.iter().zip(original.cuts.iter()) {
        if (curr_cut.start_time - orig_cut.start_time).abs() > 0.001
            || curr_cut.name != orig_cut.name
        {
            return true;
        }
    }

    // 6. 单帧触发器比对
    for (curr_os, orig_os) in current.oneshots.iter().zip(original.oneshots.iter()) {
        if (curr_os.time - orig_os.time).abs() > 0.001
            || curr_os.actor.idx != orig_os.actor.idx
            || curr_os.actor_action.idx != orig_os.actor_action.idx
        {
            return true;
        }
    }

    false
}
