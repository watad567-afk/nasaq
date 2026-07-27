//! Name resolution (Phase 1: module-level symbol collection).

use indexmap::IndexMap;
use nasaq_ast::{Item, Program};
use nasaq_diagnostics::{Diagnostic, DiagnosticBag};

pub struct ResolveResult {
    pub symbols: IndexMap<String, SymbolKind>,
    pub diagnostics: DiagnosticBag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Struct,
    Extern,
    Component,
}

pub fn resolve(program: &Program) -> ResolveResult {
    let mut diagnostics = DiagnosticBag::new();
    let mut symbols = IndexMap::new();

    for item in &program.items {
        register_item(&item.node, &mut symbols, &mut diagnostics);
    }

    ResolveResult {
        symbols,
        diagnostics,
    }
}

fn register_item(item: &Item, symbols: &mut IndexMap<String, SymbolKind>, diagnostics: &mut DiagnosticBag) {
    match item {
        Item::Function(f) => insert_symbol(&f.name.node, SymbolKind::Function, f.name.span, symbols, diagnostics),
        Item::Extern(f) => insert_symbol(&f.name.node, SymbolKind::Extern, f.name.span, symbols, diagnostics),
        Item::Struct(s) => insert_symbol(&s.name.node, SymbolKind::Struct, s.name.span, symbols, diagnostics),
        Item::Component(c) => insert_symbol(&c.name.node, SymbolKind::Component, c.name.span, symbols, diagnostics),
        Item::Export(inner) => register_item(&inner.node, symbols, diagnostics),
        Item::Import(_) => {}
    }
}

fn insert_symbol(
    name: &str,
    kind: SymbolKind,
    span: nasaq_syntax::Span,
    symbols: &mut IndexMap<String, SymbolKind>,
    diagnostics: &mut DiagnosticBag,
) {
    if symbols.insert(name.to_string(), kind).is_some() {
        diagnostics.push(
            Diagnostic::error(format!("duplicate definition of `{name}`"), span)
                .with_code("E003"),
        );
    }
}
