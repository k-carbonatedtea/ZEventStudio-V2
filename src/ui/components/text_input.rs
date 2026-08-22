use gpui::*;
use std::time::{SystemTime, UNIX_EPOCH};

/// 渲染带平滑时间闪烁效果的输入光标
pub fn render_blinking_cursor(color: Hsla) -> impl IntoElement {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    // 530ms 周期闪烁 (前 530ms 显示，后 530ms 隐藏)
    let is_visible = (now_ms / 530).is_multiple_of(2);


    div()
        .w(px(2.0))
        .h(px(13.0))
        .rounded_xs()
        .bg(if is_visible {
            color
        } else {
            hsla(0.0, 0.0, 0.0, 0.0)
        })
}

/// 处理用户键盘输入事件
/// 返回 (文本是否变动, 是否按下 Enter 确认, 是否按下 Escape 取消)
pub fn handle_text_key_event(target: &mut String, event: &KeyDownEvent) -> (bool, bool, bool) {
    let key = event.keystroke.key.as_str();

    if key == "backspace" {
        target.pop();
        return (true, false, false);
    }
    if key == "enter" {
        return (false, true, false);
    }
    if key == "escape" {
        return (false, false, true);
    }
    if key == "space" {
        target.push(' ');
        return (true, false, false);
    }

    if let Some(ref ch) = event.keystroke.key_char {
        if !event.keystroke.modifiers.control && !event.keystroke.modifiers.alt {
            target.push_str(ch);
            return (true, false, false);
        }
    } else if key.chars().count() == 1 && !event.keystroke.modifiers.control && !event.keystroke.modifiers.alt {
        if event.keystroke.modifiers.shift {
            target.push_str(&key.to_uppercase());
        } else {
            target.push_str(key);
        }
        return (true, false, false);
    } else {
        let mapped = match key {
            "underscore" => Some('_'),
            "minus" => Some('-'),
            "period" => Some('.'),
            "comma" => Some(','),
            "colon" => Some(':'),
            "semicolon" => Some(';'),
            "slash" => Some('/'),
            "backslash" => Some('\\'),
            _ => None,
        };
        if let Some(c) = mapped {
            target.push(c);
            return (true, false, false);
        }
    }
    (false, false, false)
}
