use teloxide::types::{ChatKind, Message, MessageEntityKind};

pub fn handle(msg: &Message, hashtag: &str) {
    if matches!(msg.chat.kind, ChatKind::Private(_)) {
        return;
    }
    let target = format!("#{}", hashtag.trim_start_matches('#'));
    let text = match msg.text() {
        Some(t) => t,
        None => return,
    };
    // Telegram entity offsets are UTF-16 code units
    let utf16: Vec<u16> = text.encode_utf16().collect();
    let has_tag = msg.entities().unwrap_or(&[]).iter().any(|e| {
        if !matches!(e.kind, MessageEntityKind::Hashtag) {
            return false;
        }
        let end = e.offset + e.length;
        end <= utf16.len() && String::from_utf16_lossy(&utf16[e.offset..end]) == target
    });
    if has_tag {
        println!("{}", text);
    }
}
