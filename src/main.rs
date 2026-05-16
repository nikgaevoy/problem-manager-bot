use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();

    let bot = Bot::from_env();

    teloxide::repl(bot, |_bot: Bot, msg: Message| async move {
        if let Some(text) = msg.text() {
            println!("{}", text);
        }
        Ok(())
    })
    .await;
}
