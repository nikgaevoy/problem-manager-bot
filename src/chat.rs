use std::{env, io::Write};

use teloxide::{
    prelude::*,
    types::{ChatKind, MessageEntityKind, ReplyParameters},
};

use crate::{authors, problem::Problem};

pub async fn handle(bot: &Bot, msg: &Message, hashtag: &str) -> ResponseResult<()> {
    if matches!(msg.chat.kind, ChatKind::Private(_)) {
        return Ok(());
    }
    let target = format!("#{}", hashtag.trim_start_matches('#'));
    let text = match msg.text() {
        Some(t) => t,
        None => return Ok(()),
    };
    // Telegram entity offsets are UTF-16 code units
    let utf16: Vec<u16> = text.encode_utf16().collect();
    let tag_entity = msg.entities().unwrap_or(&[]).iter().find(|e| {
        if !matches!(e.kind, MessageEntityKind::Hashtag) {
            return false;
        }
        let end = e.offset + e.length;
        end <= utf16.len() && String::from_utf16_lossy(&utf16[e.offset..end]) == target
    });
    if let Some(tag) = tag_entity {
        let tag_end = tag.offset + tag.length;
        let rest = String::from_utf16_lossy(utf16.get(tag_end..).unwrap_or(&[]));
        let rest = rest.trim().to_string();

        let reply = ReplyParameters::new(msg.id);
        let author = msg.from.as_ref()
            .map(|u| authors::resolve(u.id.0, u.full_name()))
            .unwrap_or_default();
        match Problem::from_message(message_link(msg), rest, author) {
            Ok(problem) => {
                println!("{:?}", problem);
                log_problem(&problem);
                bot.send_message(msg.chat.id, format!("Saved: {}", problem.name()))
                    .reply_parameters(reply)
                    .await?;
            }
            Err(e) => {
                bot.send_message(msg.chat.id, &e)
                    .reply_parameters(reply)
                    .await?;
            }
        }
    }
    Ok(())
}

fn message_link(msg: &Message) -> String {
    if let Some(username) = msg.chat.username() {
        format!("https://t.me/{}/{}", username, msg.id)
    } else {
        // Group IDs look like -1001234567890; t.me/c/ uses 1234567890
        let id = msg.chat.id.0.unsigned_abs().to_string();
        let short = id.strip_prefix("100").unwrap_or(&id);
        format!("https://t.me/c/{}/{}", short, msg.id)
    }
}

fn log_problem(problem: &Problem) {
    let path = env::var("PROBLEMS_FILE").unwrap_or_else(|_| "problems.jsonl".into());
    let line = match serde_json::to_string(problem) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to serialize problem: {e}");
            return;
        }
    };
    let file = std::fs::OpenOptions::new().create(true).append(true).open(&path);
    match file {
        Ok(mut f) => {
            if let Err(e) = writeln!(f, "{line}") {
                eprintln!("Failed to write to {path}: {e}");
            }
        }
        Err(e) => eprintln!("Failed to open {path}: {e}"),
    }
}
