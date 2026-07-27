//! Nasaq package registry — install packages into `vendor/`.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RegistryIndex {
    version: u32,
    packages: HashMap<String, RegistryEntry>,
}

#[derive(Debug, Deserialize)]
struct RegistryEntry {
    version: String,
    path: String,
}

#[derive(Debug, Deserialize)]
struct PackageManifest {
    name: String,
    version: String,
    entry: String,
}

pub fn registry_root() -> PathBuf {
    std::env::var("NASAQ_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
        })
}

pub fn install_package(project_root: &Path, name: &str) -> Result<PathBuf, String> {
    let index_path = registry_root().join("registry/index.json");
    let index_raw = fs::read_to_string(&index_path)
        .map_err(|e| format!("registry not found: {e}"))?;
    let index: RegistryIndex =
        serde_json::from_str(&index_raw).map_err(|e| format!("bad registry index: {e}"))?;
    let entry = index
        .packages
        .get(name)
        .ok_or_else(|| format!("package `{name}` not in registry"))?;

    let src = registry_root().join("registry").join(&entry.path);
    let dest = project_root.join("vendor").join(name);
    if dest.exists() {
        fs::remove_dir_all(&dest).map_err(|e| e.to_string())?;
    }
    copy_dir_recursive(&src, &dest)?;
    Ok(dest)
}

pub fn list_registry() -> Result<Vec<String>, String> {
    let index_path = registry_root().join("registry/index.json");
    let index_raw = fs::read_to_string(&index_path).map_err(|e| e.to_string())?;
    let index: RegistryIndex = serde_json::from_str(&index_raw).map_err(|e| e.to_string())?;
    Ok(index.packages.keys().cloned().collect())
}

pub fn package_entry(project_root: &Path, name: &str) -> Option<PathBuf> {
    let vendor = project_root.join("vendor").join(name);
    let manifest_path = vendor.join("nq.pkg.json");
    if !manifest_path.exists() {
        return None;
    }
    let raw = fs::read_to_string(&manifest_path).ok()?;
    let manifest: PackageManifest = serde_json::from_str(&raw).ok()?;
    Some(vendor.join(manifest.entry))
}

pub fn find_project_root(from: &Path) -> Option<PathBuf> {
    let mut dir = from.parent()?.to_path_buf();
    loop {
        if dir.join("nasaq.toml").exists() {
            return Some(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

pub fn find_std_root(from: &Path) -> Option<PathBuf> {
    if let Some(root) = find_project_root(from) {
        let std_dir = root.join("std");
        if std_dir.is_dir() {
            return Some(std_dir);
        }
    }
    let env_std = registry_root().join("std");
    if env_std.is_dir() {
        return Some(env_std);
    }
    None
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let ty = entry.file_type().map_err(|e| e.to_string())?;
        let to = dest.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &to)?;
        } else {
            fs::copy(entry.path(), &to).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
