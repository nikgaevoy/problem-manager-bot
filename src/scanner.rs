use std::{collections::HashSet, env, fs, io::Write};

use chrono::{DateTime, NaiveDateTime, Utc};
use scraper::{ElementRef, Html, Selector, node::Node};

use crate::problem::Problem;

const DATE_FMT: &str = "%d.%m.%Y %H:%M:%S";

fn element_text(el: ElementRef) -> String {
    let mut text = String::new();
    for child in el.children() {
        match child.value() {
            Node::Text(t) => text.push_str(t),
            Node::Element(e) if e.name() == "br" => text.push('\n'),
            _ => {
                if let Some(child_el) = ElementRef::wrap(child) {
                    text.push_str(&element_text(child_el));
                }
            }
        }
    }
    text
}

fn make_link(msg_id: &str, chat_username: Option<&str>, chat_id: Option<&str>) -> Option<String> {
    if let Some(username) = chat_username {
        Some(format!("https://t.me/{}/{}", username.trim_start_matches('@'), msg_id))
    } else if let Some(id) = chat_id {
        Some(format!("https://t.me/c/{}/{}", id.trim_start_matches('-'), msg_id))
    } else {
        None
    }
}

fn collect_links(path: &str) -> HashSet<String> {
    let content = fs::read_to_string(path).unwrap_or_default();
    content
        .lines()
        .filter_map(|l| serde_json::from_str::<Problem>(l).ok())
        .map(|p| p.link().to_string())
        .collect()
}

fn sorted_html_files(dir: &str) -> Vec<std::path::PathBuf> {
    let mut files: Vec<_> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("messages") && n.ends_with(".html"))
                .unwrap_or(false)
        })
        .collect();
    files.sort_by_key(|p| {
        let stem = p.file_stem().and_then(|n| n.to_str()).unwrap_or("");
        stem.strip_prefix("messages")
            .and_then(|s| if s.is_empty() { Some(1u64) } else { s.parse().ok() })
            .unwrap_or(u64::MAX)
    });
    files
}

pub fn scan(dir: &str, chat_username: Option<&str>, chat_id: Option<&str>) -> String {

    let hashtag_raw = env::var("HASHTAG").unwrap_or_else(|_| "problem".into());
    let target_hashtag = format!("#{}", hashtag_raw.trim_start_matches('#'));

    let pending_path = env::var("PROBLEMS_FILE").unwrap_or_else(|_| "problems.jsonl".into());
    let loaded_path =
        env::var("LOADED_PROBLEMS_FILE").unwrap_or_else(|_| "loaded_problems.jsonl".into());

    let mut existing_links = collect_links(&pending_path);
    existing_links.extend(collect_links(&loaded_path));

    let mut pending_file = match fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&pending_path)
    {
        Ok(f) => f,
        Err(e) => return format!("Cannot open {pending_path}: {e}"),
    };

    let message_sel = Selector::parse(".message.default").unwrap();
    let from_sel = Selector::parse(".from_name").unwrap();
    let date_sel = Selector::parse(".date").unwrap();
    let text_sel = Selector::parse(".text").unwrap();
    let hashtag_sel = Selector::parse("a").unwrap();

    let files = sorted_html_files(dir);
    println!("Found {} HTML file(s): {:?}", files.len(), files);

    let mut current_author = String::new();
    let mut total_messages = 0usize;
    let mut no_date = 0usize;
    let mut no_text = 0usize;
    let mut no_tag = 0usize;
    let mut bad_date = 0usize;
    let mut bad_problem = 0usize;
    let mut added = 0usize;
    let mut skipped = 0usize;

    for path in files {
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Cannot read {}: {e}", path.display());
                continue;
            }
        };
        let doc = Html::parse_document(&content);

        for msg_el in doc.select(&message_sel) {
            total_messages += 1;

            if let Some(from_el) = msg_el.select(&from_sel).next() {
                current_author = from_el.text().collect::<String>().trim().to_string();
            }

            let Some(date_el) = msg_el.select(&date_sel).next() else {
                no_date += 1;
                continue;
            };
            let Some(date_str) = date_el.value().attr("title") else {
                no_date += 1;
                continue;
            };
            let Some(text_el) = msg_el.select(&text_sel).next() else {
                no_text += 1;
                continue;
            };

            let has_tag = text_el
                .select(&hashtag_sel)
                .any(|el| el.text().collect::<String>().trim() == target_hashtag);
            if !has_tag {
                no_tag += 1;
                continue;
            }

            let Some(msg_id) = msg_el.value().attr("id").and_then(|s| s.strip_prefix("message"))
            else {
                continue;
            };
            let link_opt = make_link(msg_id, chat_username, chat_id);
            let link = link_opt.as_deref().unwrap_or(msg_id).to_string();

            if link_opt.is_some() && existing_links.contains(&link) {
                skipped += 1;
                continue;
            }

            let date_time: String = date_str.splitn(3, ' ').take(2).collect::<Vec<_>>().join(" ");
            let Ok(naive) = NaiveDateTime::parse_from_str(&date_time, DATE_FMT) else {
                eprintln!("Bad date {:?}", date_str);
                bad_date += 1;
                continue;
            };
            let date: DateTime<Utc> = naive.and_utc();

            let full_text = element_text(text_el);
            let rest = full_text
                .find(&target_hashtag)
                .map(|i| full_text[i + target_hashtag.len()..].trim())
                .unwrap_or("")
                .to_string();

            match Problem::from_message(link.clone(), rest.clone(), current_author.clone(), date) {
                Ok(problem) => match serde_json::to_string(&problem) {
                    Ok(line) => {
                        if let Err(e) = writeln!(pending_file, "{line}") {
                            eprintln!("Failed to write problem: {e}");
                        } else {
                            if link_opt.is_some() {
                                existing_links.insert(link);
                            }
                            added += 1;
                        }
                    }
                    Err(e) => eprintln!("Failed to serialize: {e}"),
                },
                Err(e) => {
                    eprintln!("Skipping malformed problem at {link}: {e}");
                    eprintln!("  Author:  {current_author}");
                    eprintln!("  Date:    {date}");
                    eprintln!("  Content: {rest}");
                    bad_problem += 1;
                }
            }
        }
    }

    println!(
        "Messages: {total_messages} total, {no_date} no-date, {no_text} no-text, \
         {no_tag} no-tag, {bad_date} bad-date, {bad_problem} bad-problem, \
         {skipped} duplicate, {added} added"
    );
    format!("Added {added} new problems, skipped {skipped} duplicates")
}
