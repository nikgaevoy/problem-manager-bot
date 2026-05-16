mod chat;
mod commands;
mod personal;
mod problem;
mod sheets;

use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
};

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
        Command::Load => load().await,
    }
}

async fn listen() {
    let hashtag = env::var("HASHTAG").expect("HASHTAG not set");
    let bot = Bot::from_env();

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

async fn load() {
    let path = env::var("PROBLEMS_FILE").unwrap_or_else(|_| "problems.jsonl".into());
    let spreadsheet_id = env::var("SPREADSHEET_ID").expect("SPREADSHEET_ID not set");

    let file = File::open(&path).unwrap_or_else(|e| panic!("Cannot open {path}: {e}"));
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        if line.trim().is_empty() {
            continue;
        }
        let problem: problem::Problem =
            serde_json::from_str(&line).expect("Failed to parse problem");
        sheets::append_problem(&problem, &spreadsheet_id)
            .await
            .unwrap_or_else(|e| eprintln!("Failed to append problem: {e}"));
    }
}
