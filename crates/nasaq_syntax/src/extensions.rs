//! Official Nasaq file extensions (branded, like `.ts` / `.rs`).

/// Source file extension — `App.nq`
pub const SOURCE: &str = "nq";

/// Legacy source alias — `App.nasaq`
pub const SOURCE_LEGACY: &str = "nasaq";

/// Compiled module extension — `profile.nq` (ESM output, no `.js` in user-facing artifacts)
pub const OUTPUT: &str = "nq";

/// Runtime module extension — `core.nqr`, `dom.nqr`
pub const RUNTIME: &str = "nqr";

pub fn source_exts() -> [&'static str; 2] {
    [SOURCE, SOURCE_LEGACY]
}

pub fn with_source_ext(base: &str) -> String {
    format!("{base}.{SOURCE}")
}

pub fn with_output_ext(name: &str) -> String {
    format!("{name}.{OUTPUT}")
}

pub fn with_runtime_ext(name: &str) -> String {
    format!("{name}.{RUNTIME}")
}

pub fn resolve_source_path(base: &std::path::Path) -> std::path::PathBuf {
    use std::path::PathBuf;
    if base.extension().is_some() {
        return base.to_path_buf();
    }
    let nq = base.with_extension(SOURCE);
    if nq.exists() {
        return nq;
    }
    base.with_extension(SOURCE_LEGACY)
}
