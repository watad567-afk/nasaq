//! Browser playground assets for Nasaq.

mod compile;

pub use compile::{compile_snippet, compile_snippet_json, CompileResult};

pub const PLAYGROUND_HTML: &str = include_str!("../playground/index.html");

pub fn playground_page() -> &'static str {
    PLAYGROUND_HTML
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
