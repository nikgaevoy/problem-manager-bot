use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
};

use crate::{problem::Problem, sheets};

pub async fn load() -> String {
    let path = env::var("PROBLEMS_FILE").unwrap_or_else(|_| "problems.jsonl".into());
    let spreadsheet_id = match env::var("SPREADSHEET_ID") {
        Ok(id) => id,
        Err(_) => return "SPREADSHEET_ID not set".into(),
    };

    let file = match File::open(&path) {
        Ok(f) => f,
        Err(e) => return format!("Cannot open {path}: {e}"),
    };

    let mut ok = 0usize;
    let mut errors = 0usize;

    for line in BufReader::new(file).lines() {
        let line = match line {
            Ok(l) if !l.trim().is_empty() => l,
            Ok(_) => continue,
            Err(e) => {
                eprintln!("Failed to read line: {e}");
                errors += 1;
                continue;
            }
        };
        let problem: Problem = match serde_json::from_str(&line) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to parse problem: {e}");
                errors += 1;
                continue;
            }
        };
        match sheets::append_problem(&problem, &spreadsheet_id).await {
            Ok(_) => ok += 1,
            Err(e) => {
                eprintln!("Failed to append problem: {e}");
                errors += 1;
            }
        }
    }

    if errors == 0 {
        format!("Loaded {ok} problems")
    } else {
        format!("Loaded {ok} problems, {errors} errors")
    }
}
