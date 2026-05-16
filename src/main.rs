mod chat;
mod commands;
mod personal;

use std::env;
use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();

    let hashtag = env::var("HASHTAG").expect("HASHTAG not set");
    let bot = Bot::from_env();

    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let hashtag = hashtag.clone();
        async move {
            commands::handle(&bot, &msg).await?;
            personal::handle(&msg);
            chat::handle(&msg, &hashtag);
            Ok(())
        }
    })
    .await;
}
