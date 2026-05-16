use std::{collections::HashMap, env};

fn path() -> String {
    env::var("AUTHORS_FILE").unwrap_or_else(|_| "authors.json".into())
}

fn load() -> HashMap<u64, String> {
    let content = std::fs::read_to_string(path()).unwrap_or_default();
    serde_json::from_str(&content).unwrap_or_default()
}

fn save(map: &HashMap<u64, String>) {
    let path = path();
    match serde_json::to_string_pretty(map) {
        Ok(s) => {
            if let Err(e) = std::fs::write(&path, s) {
                eprintln!("Failed to write {path}: {e}");
            }
        }
        Err(e) => eprintln!("Failed to serialize authors: {e}"),
    }
}

pub fn resolve(user_id: u64, fallback: String) -> String {
    load().get(&user_id).cloned().unwrap_or(fallback)
}

pub fn set(user_id: u64, name: String) {
    let mut map = load();
    map.insert(user_id, name);
    save(&map);
}
