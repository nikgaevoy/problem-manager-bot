use teloxide::{prelude::*, types::ChatKind};

use crate::loader;

pub async fn handle(bot: &Bot, msg: &Message) -> ResponseResult<()> {
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
