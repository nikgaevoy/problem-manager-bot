mod authors;
mod chat;
mod commands;
mod focus;
mod loader;
mod personal;
mod problem;
mod scanner;
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
    /// Scan a Telegram Desktop HTML export and add new problems
    Scan {
        /// Path to the export directory containing messages*.html files
        dir: String,
        /// Username of the group (without @) for constructing message links in public groups
        #[arg(long)]
        chat_username: Option<String>,
        /// Numeric ID of the group for constructing message links in private groups
        #[arg(long)]
        chat_id: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();

    match Cli::parse().command {
        Command::Listen => listen().await,
        Command::Load => println!("{}", loader::load().await),
        Command::Scan { dir, chat_username, chat_id } => {
            println!("{}", scanner::scan(&dir, chat_username.as_deref(), chat_id.as_deref()))
        }
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
