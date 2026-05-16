use std::env;

use teloxide::{prelude::*, types::{ChatKind, ReplyParameters}};

use crate::{authors, chat, loader, problem::Problem};

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

        if text.trim() == "/help" {
            let hashtag = env::var("HASHTAG").unwrap_or_else(|_| "problem".into());
            let hashtag = format!("#{}", hashtag.trim_start_matches('#'));
            let reply = format!(
                "\
{hashtag} <Problem Name>
<Legend>

Submit a problem. The first line is the name, the rest is the legend.

/setname <name> — set your display name for attribution
/load — push pending problems to the spreadsheet (group only)
/help — show this message"
            );
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
        Some(text) if text.starts_with("/set_difficulty") => {
            let value = text["/set_difficulty".len()..].trim().to_string();
            set_field(bot, msg, "/set_difficulty", value, Problem::set_difficulty).await?;
        }
        Some(text) if text.starts_with("/set_tags") => {
            let value = text["/set_tags".len()..].trim().to_string();
            set_field(bot, msg, "/set_tags", value, Problem::set_tags).await?;
        }
        _ => {}
    }
    Ok(())
}

async fn set_field(
    bot: &Bot,
    msg: &Message,
    command: &str,
    value: String,
    setter: fn(&mut Problem, String),
) -> ResponseResult<()> {
    let reply = ReplyParameters::new(msg.id);

    if value.is_empty() {
        bot.send_message(msg.chat.id, format!("Usage: {command} <value>"))
            .reply_parameters(reply)
            .await?;
        return Ok(());
    }

    let author = msg.from.as_ref()
        .map(|u| authors::resolve(u.id.0, u.full_name()))
        .unwrap_or_default();

    let target_link = match find_target_link(msg, &author) {
        Some(link) => link,
        None => {
            bot.send_message(msg.chat.id, "No matching problem found in the pending list.")
                .reply_parameters(reply)
                .await?;
            return Ok(());
        }
    };

    let result = match update_problem(&target_link, |p| setter(p, value)) {
        Ok(name) => format!("Updated: {name}"),
        Err(e) => e,
    };

    bot.send_message(msg.chat.id, result)
        .reply_parameters(reply)
        .await?;
    Ok(())
}

fn find_target_link(msg: &Message, author: &str) -> Option<String> {
    if let Some(reply) = msg.reply_to_message() {
        return Some(chat::message_link(reply));
    }
    let prefix = chat::chat_link_prefix(msg);
    let path = env::var("PROBLEMS_FILE").unwrap_or_else(|_| "problems.jsonl".into());
    let content = std::fs::read_to_string(path).ok()?;
    content.lines()
        .filter_map(|l| serde_json::from_str::<Problem>(l).ok())
        .filter(|p| p.link().starts_with(&prefix) && p.author() == author)
        .last()
        .map(|p| p.link().to_string())
}

fn update_problem(link: &str, setter: impl FnOnce(&mut Problem)) -> Result<String, String> {
    let path = env::var("PROBLEMS_FILE").unwrap_or_else(|_| "problems.jsonl".into());
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Cannot read {path}: {e}"))?;

    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

    let idx = lines.iter().position(|l| {
        serde_json::from_str::<Problem>(l).map(|p| p.link() == link).unwrap_or(false)
    }).ok_or_else(|| "Problem not found in pending list.".to_string())?;

    let mut p: Problem = serde_json::from_str(&lines[idx]).map_err(|e| e.to_string())?;
    setter(&mut p);
    let name = p.name().to_string();
    lines[idx] = serde_json::to_string(&p).map_err(|e| e.to_string())?;

    let mut new_content = lines.join("\n");
    if !new_content.is_empty() {
        new_content.push('\n');
    }
    std::fs::write(&path, new_content)
        .map_err(|e| format!("Cannot write {path}: {e}"))?;

    Ok(name)
}
