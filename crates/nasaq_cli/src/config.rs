use std::fs;
use std::path::{Path, PathBuf};

use nasaq_syntax::SourceFile;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct NasaqConfig {
    pub package: PackageConfig,
    pub build: Option<BuildConfig>,
    pub web: Option<WebConfig>,
}

#[derive(Debug, Deserialize)]
pub struct WebConfig {
    pub mount: String,
    pub component: String,
}

#[derive(Debug, Deserialize)]
pub struct PackageConfig {
    pub name: String,
    pub entry: String,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BuildConfig {
    pub out_dir: Option<String>,
    pub runtime: Option<String>,
}

pub fn load_project(root: &Path) -> miette::Result<NasaqConfig> {
    let manifest = root.join("nasaq.toml");
    let contents = fs::read_to_string(&manifest)
        .into_diagnostic()
        .wrap_err_with(|| format!("missing project manifest: {}", manifest.display()))?;
    toml::from_str(&contents)
        .into_diagnostic()
        .wrap_err("failed to parse nasaq.toml")
}

pub fn entry_file(root: &Path, config: &NasaqConfig) -> PathBuf {
    root.join(&config.package.entry)
}

use miette::{IntoDiagnostic, WrapErr};

pub fn read_source(path: &Path) -> miette::Result<SourceFile> {
    let contents = fs::read_to_string(path)
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to read {}", path.display()))?;
    Ok(SourceFile::new(
        path.to_string_lossy().to_string(),
        contents,
    ))
}
