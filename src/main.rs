mod authors;
mod chat;
mod commands;
mod focus;
mod loader;
mod personal;
mod problem;
mod sheets;

use std::env;

use clap::{Parser, Subcommand};
use teloxide::prelude::*;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the Telegram bot and listen for messages
    Listen,
    /// Load problems from the log file into the Google Spreadsheet
    Load,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();

    match Cli::parse().command {
        Command::Listen => listen().await,
        Command::Load => println!("{}", loader::load().await),
    }
}

async fn listen() {
    let hashtag = env::var("HASHTAG").expect("HASHTAG not set");
    let bot = Bot::from_env();
    commands::register_commands(&bot).await;

    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let hashtag = hashtag.clone();
        async move {
            commands::handle(&bot, &msg).await?;
            personal::handle(&msg);
            chat::handle(&bot, &msg, &hashtag).await?;
            Ok(())
        }
    })
    .await;
}
