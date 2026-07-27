//! Incremental compilation cache keyed by source file hashes.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use nasaq_loader::LoadedProgram;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheManifest {
    pub version: u32,
    pub entries: HashMap<String, CacheEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub hash: String,
    pub js_path: String,
    pub updated_ms: u128,
}

pub struct CacheStore {
    path: PathBuf,
    manifest: CacheManifest,
}

impl CacheStore {
    pub fn open(project_root: &Path) -> Self {
        let path = project_root.join(".nasaq").join("cache.json");
        let manifest = fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(CacheManifest {
                version: 1,
                entries: HashMap::new(),
            });
        Self { path, manifest }
    }

    pub fn is_fresh(&self, loaded: &LoadedProgram, out_js: &Path) -> bool {
        let key = loaded.entry.to_string_lossy().to_string();
        let Some(entry) = self.manifest.entries.get(&key) else {
            return false;
        };
        if !out_js.exists() {
            return false;
        }
        let current = hash_loaded(loaded);
        entry.hash == current
    }

    pub fn record(&mut self, loaded: &LoadedProgram, out_js: &Path) {
        let key = loaded.entry.to_string_lossy().to_string();
        let hash = hash_loaded(loaded);
        self.manifest.entries.insert(
            key,
            CacheEntry {
                hash,
                js_path: out_js.to_string_lossy().to_string(),
                updated_ms: now_ms(),
            },
        );
        if let Some(parent) = self.path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.manifest) {
            let _ = fs::write(&self.path, json);
        }
    }
}

fn hash_loaded(loaded: &LoadedProgram) -> String {
    let mut hasher = Sha256::new();
    for (path, source) in &loaded.sources {
        hasher.update(path.to_string_lossy().as_bytes());
        hasher.update(source.contents.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

fn now_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_roundtrip() {
        let mut m = CacheManifest::default();
        m.entries.insert(
            "main.nasaq".into(),
            CacheEntry {
                hash: "abc".into(),
                js_path: "dist/app.js".into(),
                updated_ms: 1,
            },
        );
        let json = serde_json::to_string(&m).unwrap();
        let back: CacheManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.entries.len(), 1);
    }
}
