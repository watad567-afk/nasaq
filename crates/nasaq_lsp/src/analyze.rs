//! Diagnostic analysis for Nasaq sources.

use nasaq_diagnostics::DiagnosticBag;
use nasaq_loader::load_program;
use nasaq_parser::parse_program;
use nasaq_resolver::resolve;
use nasaq_types::typecheck;
use std::path::Path;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct AnalysisResult {
    pub diagnostics: DiagnosticBag,
}

pub fn analyze_source(source: &str) -> AnalysisResult {
    let mut diagnostics = DiagnosticBag::new();
    let parsed = parse_program(source);
    diagnostics.merge(parsed.diagnostics);
    if let Some(program) = parsed.program {
        diagnostics.merge(resolve(&program).diagnostics);
        diagnostics.merge(typecheck(&program).diagnostics);
    }
    AnalysisResult { diagnostics }
}

pub fn analyze_file(path: &Path) -> AnalysisResult {
    let loaded = load_program(path);
    let mut diagnostics = loaded.diagnostics;
    diagnostics.merge(resolve(&loaded.program).diagnostics);
    diagnostics.merge(typecheck(&loaded.program).diagnostics);
    AnalysisResult { diagnostics }
}

pub fn capabilities() -> &'static [&'static str] {
    &[
        "textDocument/publishDiagnostics",
        "textDocument/completion",
        "textDocument/didOpen",
        "textDocument/didChange",
        "textDocument/didSave",
    ]
}
