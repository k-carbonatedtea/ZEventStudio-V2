use gpui::KeyDownEvent;

/// 在指定索引处查找词的起止范围 (双击选词算法)
pub fn find_word_range_at(chars: &[char], idx: usize) -> (usize, usize) {
    if chars.is_empty() {
        return (0, 0);
    }
    let idx = idx.min(chars.len() - 1);
    let is_delim = |c: char| c.is_whitespace() || ",.:;()[]{}\"'/\\!@#$%^&*+-=<>\n\r".contains(c);
    let target_delim = is_delim(chars[idx]);

    let mut start = idx;
    while start > 0 && is_delim(chars[start - 1]) == target_delim {
        start -= 1;
    }
    let mut end = idx + 1;
    while end < chars.len() && is_delim(chars[end]) == target_delim {
        end += 1;
    }
    (start, end)
}

/// 查找前一个词边界 (Ctrl+Left / Ctrl+Backspace)
pub fn find_prev_word_boundary(chars: &[char], cursor: usize) -> usize {
    if cursor == 0 {
        return 0;
    }
    let mut i = cursor.min(chars.len());
    while i > 0 && chars[i - 1].is_whitespace() {
        i -= 1;
    }
    while i > 0 && !chars[i - 1].is_whitespace() {
        i -= 1;
    }
    i
}

/// 查找后一个词边界 (Ctrl+Right / Ctrl+Delete)
pub fn find_next_word_boundary(chars: &[char], cursor: usize) -> usize {
    let len = chars.len();
    if cursor >= len {
        return len;
    }
    let mut i = cursor;
    while i < len && !chars[i].is_whitespace() {
        i += 1;
    }
    while i < len && chars[i].is_whitespace() {
        i += 1;
    }
    i
}

/// 在字符串的指定光标/选区处插入或替换内容
pub fn splice_string_at(
    target: &mut String,
    cursor: &mut usize,
    select_anchor: &mut Option<usize>,
    text: &str,
) {
    let mut chars: Vec<char> = target.chars().collect();
    let total = chars.len();
    let cur = (*cursor).min(total);

    let (start, end) = if let Some(a) = *select_anchor {
        let s = a.min(cur).min(total);
        let e = a.max(cur).min(total);
        if s < e { (s, e) } else { (cur, cur) }
    } else {
        (cur, cur)
    };

    let inserted_chars: Vec<char> = text.chars().collect();
    let inserted_len = inserted_chars.len();

    chars.splice(start..end, inserted_chars);
    *target = chars.into_iter().collect();
    *cursor = start + inserted_len;
    *select_anchor = None;
}

/// 处理用户键盘输入事件 (支持全字符光标导航、选区操作、剪贴板与删除)
pub fn handle_text_key_event(
    target: &mut String,
    cursor: &mut usize,
    select_anchor: &mut Option<usize>,
    event: &KeyDownEvent,
    clipboard: Option<&str>,
) -> (bool, bool, bool) {
    let key = event.keystroke.key.as_str();
    let is_ctrl = event.keystroke.modifiers.control;
    let is_shift = event.keystroke.modifiers.shift;
    let is_alt = event.keystroke.modifiers.alt;

    let chars: Vec<char> = target.chars().collect();
    let total = chars.len();
    *cursor = (*cursor).min(total);

    let sel = select_anchor.and_then(|a| {
        let s = a.min(*cursor).min(total);
        let e = a.max(*cursor).min(total);
        if s < e { Some((s, e)) } else { None }
    });

    // 1. 全选 Ctrl+A
    if is_ctrl && !is_alt && key.eq_ignore_ascii_case("a") {
        *select_anchor = Some(0);
        *cursor = total;
        return (true, false, false);
    }

    // 2. 粘贴 Ctrl+V
    if is_ctrl && !is_alt && key.eq_ignore_ascii_case("v") {
        if let Some(clip) = clipboard {
            let clean: String = clip.chars().filter(|&c| c != '\r' && c != '\x08' && c != '\x1b').collect();
            if !clean.is_empty() {
                splice_string_at(target, cursor, select_anchor, &clean);
                return (true, false, false);
            }
        }
        return (false, false, false);
    }

    // 3. 退出 / 确认
    if key.eq_ignore_ascii_case("escape") {
        *select_anchor = None;
        return (false, false, true);
    }
    if key.eq_ignore_ascii_case("enter") {
        *select_anchor = None;
        return (false, true, false);
    }

    // 4. 左移 Left
    if key.eq_ignore_ascii_case("left") {
        let target_idx = if is_ctrl {
            find_prev_word_boundary(&chars, *cursor)
        } else {
            (*cursor).saturating_sub(1)
        };
        if is_shift {
            if select_anchor.is_none() { *select_anchor = Some(*cursor); }
            *cursor = target_idx;
        } else if let Some((s, _)) = sel {
            *cursor = if is_ctrl { target_idx } else { s };
            *select_anchor = None;
        } else {
            *cursor = target_idx;
            *select_anchor = None;
        }
        return (true, false, false);
    }

    // 5. 右移 Right
    if key.eq_ignore_ascii_case("right") {
        let target_idx = if is_ctrl {
            find_next_word_boundary(&chars, *cursor)
        } else {
            (*cursor + 1).min(total)
        };
        if is_shift {
            if select_anchor.is_none() { *select_anchor = Some(*cursor); }
            *cursor = target_idx;
        } else if let Some((_, e)) = sel {
            *cursor = if is_ctrl { target_idx } else { e };
            *select_anchor = None;
        } else {
            *cursor = target_idx;
            *select_anchor = None;
        }
        return (true, false, false);
    }

    // 6. Home / End
    if key.eq_ignore_ascii_case("home") {
        if is_shift {
            if select_anchor.is_none() { *select_anchor = Some(*cursor); }
        } else {
            *select_anchor = None;
        }
        *cursor = 0;
        return (true, false, false);
    }
    if key.eq_ignore_ascii_case("end") {
        if is_shift {
            if select_anchor.is_none() { *select_anchor = Some(*cursor); }
        } else {
            *select_anchor = None;
        }
        *cursor = total;
        return (true, false, false);
    }

    // 7. Backspace
    if key.eq_ignore_ascii_case("backspace") {
        if let Some((s, e)) = sel {
            let mut chs = chars;
            chs.drain(s..e);
            *target = chs.into_iter().collect();
            *cursor = s;
            *select_anchor = None;
            return (true, false, false);
        }
        if *cursor > 0 {
            let mut chs = chars;
            let del_start = if is_ctrl { find_prev_word_boundary(&chs, *cursor) } else { *cursor - 1 };
            if del_start < *cursor {
                chs.drain(del_start..*cursor);
                *target = chs.into_iter().collect();
                *cursor = del_start;
                *select_anchor = None;
                return (true, false, false);
            }
        }
        return (false, false, false);
    }

    // 8. Delete
    if key.eq_ignore_ascii_case("delete") {
        if let Some((s, e)) = sel {
            let mut chs = chars;
            chs.drain(s..e);
            *target = chs.into_iter().collect();
            *cursor = s;
            *select_anchor = None;
            return (true, false, false);
        }
        if *cursor < total {
            let mut chs = chars;
            let del_end = if is_ctrl { find_next_word_boundary(&chs, *cursor) } else { *cursor + 1 };
            if del_end > *cursor {
                chs.drain(*cursor..del_end.min(total));
                *target = chs.into_iter().collect();
                *select_anchor = None;
                return (true, false, false);
            }
        }
        return (false, false, false);
    }

    // 9. 空格输入
    if key.eq_ignore_ascii_case("space") && !is_ctrl && !is_alt {
        splice_string_at(target, cursor, select_anchor, " ");
        return (true, false, false);
    }

    // 10. 普通字符与符号输入
    if !is_ctrl && !is_alt {
        if let Some(ref ch) = event.keystroke.key_char {
            splice_string_at(target, cursor, select_anchor, ch);
            return (true, false, false);
        }
        if key.chars().count() == 1 {
            let s = if is_shift { key.to_uppercase() } else { key.to_string() };
            splice_string_at(target, cursor, select_anchor, &s);
            return (true, false, false);
        }
        let mapped = match key {
            "underscore" => Some('_'), "minus" => Some('-'), "period" => Some('.'),
            "comma" => Some(','), "colon" => Some(':'), "semicolon" => Some(';'),
            "slash" => Some('/'), "backslash" => Some('\\'), "less" => Some('<'), "greater" => Some('>'),
            "bracketleft" => Some(if is_shift { '{' } else { '[' }),
            "bracketright" => Some(if is_shift { '}' } else { ']' }),
            "equal" => Some(if is_shift { '+' } else { '=' }), "plus" => Some('+'),
            "quote" => Some(if is_shift { '"' } else { '\'' }),
            "backquote" => Some(if is_shift { '~' } else { '`' }),
            "exclamation" => Some('!'), "question" => Some('?'), _ => None,
        };
        if let Some(c) = mapped {
            splice_string_at(target, cursor, select_anchor, &c.to_string());
            return (true, false, false);
        }
    }

    (false, false, false)
}
