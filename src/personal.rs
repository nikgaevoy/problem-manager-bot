use teloxide::types::{ChatKind, Message};

pub fn handle(msg: &Message) {
    if !matches!(msg.chat.kind, ChatKind::Private(_)) {
        return;
    }
    if let Some(text) = msg.text() {
        println!("{}", text);
    }
}
