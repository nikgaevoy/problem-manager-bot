use std::env;

use teloxide::{prelude::*, types::{BotCommand, BotCommandScope, ChatKind, ReplyParameters}};

use crate::{authors, chat, focus, loader, problem::Problem};

pub async fn register_commands(bot: &Bot) {
    let private = vec![
        BotCommand::new("set_name", "Set your display name for attribution"),
        BotCommand::new("help", "Show available commands"),
    ];
    let group = vec![
        BotCommand::new("set_name", "Set your display name for attribution"),
        BotCommand::new("set_difficulty", "Set difficulty for the target problem"),
        BotCommand::new("set_tags", "Set tags for the target problem"),
        BotCommand::new("editorial", "Reply to attach editorial to the target problem"),
        BotCommand::new("solution", "Alias for /editorial"),
        BotCommand::new("focus_problem", "Reply to focus a problem for 20 minutes"),
        BotCommand::new("clear_focus", "Clear the focused problem"),
BotCommand::new("load", "Push pending problems to the spreadsheet"),
        BotCommand::new("leave", "Make the bot leave the chat"),
        BotCommand::new("help", "Show available commands"),
    ];
    bot.set_my_commands(private)
        .scope(BotCommandScope::AllPrivateChats)
        .await
        .unwrap();
    bot.set_my_commands(group)
        .scope(BotCommandScope::AllGroupChats)
        .await
        .unwrap();
}

fn strip_command<'a>(text: &'a str, cmd: &str) -> Option<&'a str> {
    let rest = text.strip_prefix(cmd)?;
    let rest = match rest.chars().next() {
        None | Some(' ') | Some('\n') => rest,
        Some('@') => {
            let after_at = &rest[1..];
            match after_at.find(|c: char| c.is_whitespace()) {
                Some(i) => &after_at[i..],
                None => "",
            }
        }
        _ => return None,
    };
    Some(rest.trim_start())
}

pub async fn handle(bot: &Bot, msg: &Message) -> ResponseResult<()> {
    if let Some(text) = msg.text() {
        if let Some(name) = strip_command(text, "/set_name") {
            let name = name.to_string();
            let reply = match msg.from.as_ref() {
                _ if name.is_empty() => "Usage: /set_name <your name>".to_string(),
                Some(user) => {
                    authors::set(user.id.0, name.clone());
                    format!("Your name is now \"{name}\"")
                }
                None => "Could not identify you as a user.".to_string(),
            };
            bot.send_message(msg.chat.id, reply).await?;
            return Ok(());
        }

        if strip_command(text, "/help").is_some() {
            let hashtag = env::var("HASHTAG").unwrap_or_else(|_| "problem".into());
            let hashtag = format!("#{}", hashtag.trim_start_matches('#'));
            let reply = format!(
                "\
{hashtag} <Legend>
or
{hashtag} <Problem Name>
<Legend>

Submit a problem. If the text after the hashtag contains a newline, the first line is the name and the rest is the legend. Otherwise the whole text is treated as the legend.

/set_name <name> — set your display name for attribution
/set_difficulty <value> — set difficulty for the target problem: reply > focus > last (group only)
/set_tags <value> — set tags for the target problem: reply > focus > last (group only)
/editorial (or /solution) — reply to an editorial message to attach it to the target problem: reply-to-reply > focus > last (group only)
/focus_problem — reply to a problem to focus it for 20 minutes; without a reply, clears focus (group only)
/clear_focus — clear the focused problem (group only)
/load — push pending problems to the spreadsheet (group only)
/leave — make the bot leave the chat (group only)
/help — show this message"
            );
            bot.send_message(msg.chat.id, reply).await?;
            return Ok(());
        }
    }

    if matches!(msg.chat.kind, ChatKind::Private(_)) {
        return Ok(());
    }

    if let Some(text) = msg.text() {
        if strip_command(text, "/leave").is_some() {
            bot.leave_chat(msg.chat.id).await?;
        } else if strip_command(text, "/load").is_some() {
            let result = loader::load().await;
            bot.send_message(msg.chat.id, result).await?;
        } else if let Some(value) = strip_command(text, "/set_difficulty") {
            set_field(bot, msg, "/set_difficulty", value.to_string(), Problem::set_difficulty).await?;
        } else if let Some(value) = strip_command(text, "/set_tags") {
            set_field(bot, msg, "/set_tags", value.to_string(), Problem::set_tags).await?;
        } else if strip_command(text, "/editorial").is_some() || strip_command(text, "/solution").is_some() {
            set_editorial(bot, msg).await?;
        } else if strip_command(text, "/focus_problem").is_some() {
            focus_problem(bot, msg).await?;
        } else if strip_command(text, "/clear_focus").is_some() {
            clear_focus(bot, msg).await?;
        }
    }
    Ok(())
}

async fn set_editorial(bot: &Bot, msg: &Message) -> ResponseResult<()> {
    let reply = ReplyParameters::new(msg.id);

    let editorial_msg = match msg.reply_to_message() {
        Some(m) => m,
        None => {
            bot.send_message(msg.chat.id, "Reply to a message containing the editorial.")
                .reply_parameters(reply)
                .await?;
            return Ok(());
        }
    };

    let editorial = match editorial_msg.text() {
        Some(t) => t.to_string(),
        None => {
            bot.send_message(msg.chat.id, "The replied message has no text.")
                .reply_parameters(reply)
                .await?;
            return Ok(());
        }
    };

    let editorial_link = chat::message_link(editorial_msg);

    let target_link = if let Some(statement_msg) = editorial_msg.reply_to_message() {
        Some(chat::message_link(statement_msg))
    } else {
        let author = msg.from.as_ref()
            .map(|u| authors::resolve(u.id.0, u.full_name()))
            .unwrap_or_default();
        find_target_link(msg, &author)
    };

    let target_link = match target_link {
        Some(link) => link,
        None => {
            bot.send_message(msg.chat.id, "No matching problem found in the pending list.")
                .reply_parameters(reply)
                .await?;
            return Ok(());
        }
    };

    let result = match update_problem(&target_link, |p| {
        p.set_editorial(editorial);
        p.set_editorial_link(editorial_link);
    }) {
        Ok(name) => format!("Updated: {name}"),
        Err(e) => e,
    };

    bot.send_message(msg.chat.id, result)
        .reply_parameters(reply)
        .await?;
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
    if let Some(user_id) = msg.from.as_ref().map(|u| u.id.0) {
        if let Some(link) = focus::get(user_id) {
            return Some(link);
        }
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

async fn focus_problem(bot: &Bot, msg: &Message) -> ResponseResult<()> {
    let reply_params = ReplyParameters::new(msg.id);

    let reply_msg = match msg.reply_to_message() {
        Some(m) => m,
        None => {
            if let Some(user) = msg.from.as_ref() {
                focus::clear(user.id.0);
            }
            bot.send_message(msg.chat.id, "Focus cleared.")
                .reply_parameters(reply_params)
                .await?;
            return Ok(());
        }
    };

    let link = chat::message_link(reply_msg);

    let path = env::var("PROBLEMS_FILE").unwrap_or_else(|_| "problems.jsonl".into());
    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let is_problem = content.lines()
        .filter_map(|l| serde_json::from_str::<Problem>(l).ok())
        .any(|p| p.link() == link);

    if !is_problem {
        bot.send_message(msg.chat.id, "The replied message is not a known pending problem.")
            .reply_parameters(reply_params)
            .await?;
        return Ok(());
    }

    if let Some(user) = msg.from.as_ref() {
        focus::set(user.id.0, link);
        bot.send_message(msg.chat.id, "Focus set for 20 minutes.")
            .reply_parameters(reply_params)
            .await?;
    }
    Ok(())
}

async fn clear_focus(bot: &Bot, msg: &Message) -> ResponseResult<()> {
    let reply_params = ReplyParameters::new(msg.id);
    if let Some(user) = msg.from.as_ref() {
        focus::clear(user.id.0);
    }
    bot.send_message(msg.chat.id, "Focus cleared.")
        .reply_parameters(reply_params)
        .await?;
    Ok(())
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
    let name = p.name();
    let display = if name.is_empty() { link } else { name }.to_string();
    lines[idx] = serde_json::to_string(&p).map_err(|e| e.to_string())?;

    let mut new_content = lines.join("\n");
    if !new_content.is_empty() {
        new_content.push('\n');
    }
    std::fs::write(&path, new_content)
        .map_err(|e| format!("Cannot write {path}: {e}"))?;

    Ok(display)
}
