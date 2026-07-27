//! Package manager commands.

use std::fs;
use std::path::Path;

use miette::{IntoDiagnostic, Result, WrapErr};

pub fn add_package(path: &str, name: &str) -> Result<()> {
    let root = Path::new(path);
    let dest = nasaq_registry::install_package(root, name).map_err(|e| miette::miette!(e))?;
    update_manifest_deps(root, name)?;
    println!("✓ added `{name}` → {}", dest.display());
    Ok(())
}

pub fn install_deps(path: &str) -> Result<()> {
    let root = Path::new(path);
    let manifest = root.join("nasaq.toml");
    let raw = fs::read_to_string(&manifest).into_diagnostic()?;
    let mut count = 0usize;
    for line in raw.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(|c: char| c.is_ascii_alphabetic() || c == '_') {
            if let Some((name, _)) = rest.split_once('=') {
                let name = name.trim();
                if !name.is_empty() && !["name", "version", "entry", "out_dir", "runtime", "mount", "component"].contains(&name) {
                    if nasaq_registry::install_package(root, name).is_ok() {
                        count += 1;
                        println!("✓ installed `{name}`");
                    }
                }
            }
        }
    }
    if count == 0 {
        println!("✓ no [dependencies] to install — use: nasaq add <package>");
    }
    Ok(())
}

pub fn search_packages() -> Result<()> {
    let names = nasaq_registry::list_registry().map_err(|e| miette::miette!(e))?;
    println!("Nasaq Registry (nq.pkg):");
    for name in names {
        println!("  • {name}");
    }
    Ok(())
}

fn update_manifest_deps(root: &Path, name: &str) -> Result<()> {
    let manifest = root.join("nasaq.toml");
    let mut raw = fs::read_to_string(&manifest).into_diagnostic()?;
    if !raw.contains("[dependencies]") {
        raw.push_str("\n[dependencies]\n");
    }
    if !raw.contains(&format!("{name} =")) {
        raw.push_str(&format!("{name} = \"0.1.0\"\n"));
    }
    fs::write(&manifest, raw)
        .into_diagnostic()
        .wrap_err("failed to update nasaq.toml")?;
    Ok(())
}
