use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

const FOCUS_DURATION: Duration = Duration::from_secs(20 * 60);

fn store() -> &'static Mutex<HashMap<u64, (String, Instant)>> {
    static STORE: OnceLock<Mutex<HashMap<u64, (String, Instant)>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn set(user_id: u64, link: String) {
    store().lock().unwrap().insert(user_id, (link, Instant::now()));
}

pub fn get(user_id: u64) -> Option<String> {
    let store = store().lock().unwrap();
    let (link, set_at) = store.get(&user_id)?;
    if set_at.elapsed() < FOCUS_DURATION {
        Some(link.clone())
    } else {
        None
    }
}

pub fn clear(user_id: u64) {
    store().lock().unwrap().remove(&user_id);
}
