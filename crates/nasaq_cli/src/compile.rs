use nasaq_codegen_js::{CodegenOptions, emit_module};
use nasaq_diagnostics::DiagnosticBag;
use nasaq_hir::lower;
use nasaq_loader::LoadedProgram;
use nasaq_resolver::resolve;
use nasaq_types::typecheck;

pub struct CompileOutput {
    pub js: String,
    pub source_map: Option<String>,
    pub diagnostics: DiagnosticBag,
    pub module_name: String,
}

pub fn compile_loaded(
    loaded: &LoadedProgram,
    module_name: &str,
    runtime_import: &str,
    web_mount: Option<(String, String)>,
    hydrate: bool,
) -> CompileOutput {
    let mut diagnostics = DiagnosticBag::new();
    diagnostics
        .diagnostics
        .extend_from_slice(&loaded.diagnostics.diagnostics);

    let resolved = resolve(&loaded.program);
    diagnostics.merge(resolved.diagnostics);

    let typed = typecheck(&loaded.program);
    diagnostics.merge(typed.diagnostics);

    if diagnostics.has_errors() {
        return CompileOutput {
            js: String::new(),
            source_map: None,
            diagnostics,
            module_name: module_name.to_string(),
        };
    }

    let hir = lower(loaded.program.clone());
    let generated = emit_module(
        &hir,
        &CodegenOptions {
            module_name: module_name.to_string(),
            runtime_import: runtime_import.to_string(),
            dom_runtime_import: format!("./runtime/{}", nasaq_syntax::with_runtime_ext("dom")),
            source_map: true,
            web_mount,
            hydrate,
        },
    );

    CompileOutput {
        js: generated.js,
        source_map: generated.source_map,
        diagnostics,
        module_name: module_name.to_string(),
    }
}

pub fn render_loaded_diagnostics(
    loaded: &LoadedProgram,
    diagnostics: &DiagnosticBag,
) -> String {
    diagnostics
        .diagnostics
        .iter()
        .map(|d| {
            if d.span == nasaq_syntax::Span::EMPTY {
                return format!("error: {}", d.message);
            }
            for source in loaded.sources.values() {
                if d.span.end as usize <= source.contents.len() {
                    return d.render(source);
                }
            }
            format!("error: {}", d.message)
        })
        .collect::<Vec<_>>()
        .join("\n")
}
