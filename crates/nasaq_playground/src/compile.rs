//! Compile Nasaq source snippets for the browser playground.

use nasaq_codegen_js::{CodegenOptions, emit_module};
use nasaq_diagnostics::DiagnosticBag;
use nasaq_hir::lower;
use nasaq_parser::parse_program;
use nasaq_resolver::resolve;
use nasaq_types::typecheck;

pub struct CompileResult {
    pub js: String,
    pub diagnostics: DiagnosticBag,
}

pub fn compile_snippet(source: &str, module_name: &str) -> CompileResult {
    let mut diagnostics = DiagnosticBag::new();
    let parsed = parse_program(source);
    diagnostics.merge(parsed.diagnostics);
    let Some(program) = parsed.program else {
        return CompileResult {
            js: String::new(),
            diagnostics,
        };
    };

    diagnostics.merge(resolve(&program).diagnostics);
    diagnostics.merge(typecheck(&program).diagnostics);
    if diagnostics.has_errors() {
        return CompileResult {
            js: String::new(),
            diagnostics,
        };
    }

    let hir = lower(program);
    let generated = emit_module(
        &hir,
        &CodegenOptions {
            module_name: module_name.to_string(),
            runtime_import: format!("./runtime/{}", nasaq_syntax::with_runtime_ext("core")),
            dom_runtime_import: format!("./runtime/{}", nasaq_syntax::with_runtime_ext("dom")),
            source_map: false,
            web_mount: None,
            hydrate: false,
        },
    );
    CompileResult {
        js: generated.js,
        diagnostics,
    }
}

pub fn compile_snippet_json(source: &str) -> String {
    let result = compile_snippet(source, "playground");
    if result.diagnostics.has_errors() {
        let errors: Vec<_> = result
            .diagnostics
            .diagnostics
            .iter()
            .map(|d| d.message.clone())
            .collect();
        return serde_json::json!({ "ok": false, "errors": errors }).to_string();
    }
    serde_json::json!({ "ok": true, "js": result.js }).to_string()
}
