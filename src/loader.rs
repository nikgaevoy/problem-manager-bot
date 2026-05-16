use std::{
    env,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
};

use crate::{problem::Problem, sheets};

pub async fn load() -> String {
    let path = env::var("PROBLEMS_FILE").unwrap_or_else(|_| "problems.jsonl".into());
    let loaded_path =
        env::var("LOADED_PROBLEMS_FILE").unwrap_or_else(|_| "loaded_problems.jsonl".into());
    let spreadsheet_id = match env::var("SPREADSHEET_ID") {
        Ok(id) => id,
        Err(_) => return "SPREADSHEET_ID not set".into(),
    };

    let lines: Vec<String> = match File::open(&path) {
        Ok(f) => BufReader::new(f)
            .lines()
            .filter_map(|l| l.ok())
            .filter(|l| !l.trim().is_empty())
            .collect(),
        Err(e) => return format!("Cannot open {path}: {e}"),
    };

    let mut loaded_file = match OpenOptions::new().create(true).append(true).open(&loaded_path) {
        Ok(f) => f,
        Err(e) => return format!("Cannot open {loaded_path}: {e}"),
    };

    let mut ok = 0usize;
    let mut failed: Vec<String> = Vec::new();

    for line in &lines {
        let problem: Problem = match serde_json::from_str(line) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to parse problem: {e}");
                failed.push(line.clone());
                continue;
            }
        };
        match sheets::append_problem(&problem, &spreadsheet_id).await {
            Ok(_) => {
                ok += 1;
                if let Err(e) = writeln!(loaded_file, "{line}") {
                    eprintln!("Failed to write to {loaded_path}: {e}");
                }
            }
            Err(e) => {
                eprintln!("Failed to append problem: {e}");
                failed.push(line.clone());
            }
        }
    }

    match File::create(&path) {
        Ok(mut f) => {
            for line in &failed {
                if let Err(e) = writeln!(f, "{line}") {
                    eprintln!("Failed to rewrite {path}: {e}");
                }
            }
        }
        Err(e) => eprintln!("Failed to rewrite {path}: {e}"),
    }

    let errors = failed.len();
    if errors == 0 {
        format!("Loaded {ok} problems")
    } else {
        format!("Loaded {ok} problems, {errors} failed (kept in {path})")
    }
}
