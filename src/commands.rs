use teloxide::{prelude::*, types::ChatKind};

pub async fn handle(bot: &Bot, msg: &Message) -> ResponseResult<()> {
    if matches!(msg.chat.kind, ChatKind::Private(_)) {
        return Ok(());
    }
    if msg.text() == Some("/leave") {
        bot.leave_chat(msg.chat.id).await?;
    }
    Ok(())
}
