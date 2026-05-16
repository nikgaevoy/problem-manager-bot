use teloxide::{prelude::*, types::ChatKind};

use crate::{authors, loader};

pub async fn handle(bot: &Bot, msg: &Message) -> ResponseResult<()> {
    if let Some(text) = msg.text() {
        if let Some(name) = text.strip_prefix("/setname") {
            let name = name.trim().to_string();
            let reply = match msg.from.as_ref() {
                _ if name.is_empty() => "Usage: /setname <your name>".to_string(),
                Some(user) => {
                    authors::set(user.id.0, name.clone());
                    format!("Your name is now \"{name}\"")
                }
                None => "Could not identify you as a user.".to_string(),
            };
            bot.send_message(msg.chat.id, reply).await?;
            return Ok(());
        }
    }

    if matches!(msg.chat.kind, ChatKind::Private(_)) {
        return Ok(());
    }
    match msg.text() {
        Some("/leave") => {
            bot.leave_chat(msg.chat.id).await?;
        }
        Some("/load") => {
            let result = loader::load().await;
            bot.send_message(msg.chat.id, result).await?;
        }
        _ => {}
    }
    Ok(())
}
