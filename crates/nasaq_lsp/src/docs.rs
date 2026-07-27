//! Document cache for LSP requests (completion, hover).

use std::collections::HashMap;
use std::sync::Mutex;

static DOCS: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

fn store() -> std::sync::MutexGuard<'static, Option<HashMap<String, String>>> {
    let mut guard = DOCS.lock().unwrap();
    if guard.is_none() {
        *guard = Some(HashMap::new());
    }
    guard
}

pub fn update(uri: &str, text: &str) {
    if let Some(map) = store().as_mut() {
        map.insert(uri.to_string(), text.to_string());
    }
}

pub fn get(uri: &str) -> Option<String> {
    store().as_ref()?.get(uri).cloned()
}
