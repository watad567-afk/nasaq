//! Load Nasaq programs with `import` resolution across multiple files.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use nasaq_ast::{Item, Program};
use nasaq_diagnostics::{Diagnostic, DiagnosticBag};
use nasaq_parser::parse_program;
use nasaq_syntax::{SourceFile, Span, Spanned};

pub struct LoadedProgram {
    pub program: Program,
    pub sources: IndexMap<PathBuf, SourceFile>,
    pub entry: PathBuf,
    pub diagnostics: DiagnosticBag,
}

pub fn load_program(entry: &Path) -> LoadedProgram {
    let mut loader = Loader {
        entry: entry.to_path_buf(),
        sources: IndexMap::new(),
        diagnostics: DiagnosticBag::new(),
        visiting: HashSet::new(),
        loaded: IndexMap::new(),
    };
    loader.load_recursive(entry);
    let program = loader.merge_program();
    LoadedProgram {
        program,
        sources: loader.sources,
        entry: loader.entry,
        diagnostics: loader.diagnostics,
    }
}

struct Loader {
    entry: PathBuf,
    sources: IndexMap<PathBuf, SourceFile>,
    diagnostics: DiagnosticBag,
    visiting: HashSet<PathBuf>,
    loaded: IndexMap<PathBuf, Program>,
}

impl Loader {
    fn load_recursive(&mut self, path: &Path) {
        let canonical = match fs::canonicalize(path) {
            Ok(p) => p,
            Err(_) => {
                self.diagnostics.push(Diagnostic::error(
                    format!("cannot open module `{}`", path.display()),
                    Span::EMPTY,
                ));
                return;
            }
        };

        if self.loaded.contains_key(&canonical) {
            return;
        }
        if !self.visiting.insert(canonical.clone()) {
            self.diagnostics.push(Diagnostic::error(
                format!("circular import detected at `{}`", canonical.display()),
                Span::EMPTY,
            ));
            return;
        }

        let contents = match fs::read_to_string(&canonical) {
            Ok(c) => c,
            Err(err) => {
                self.diagnostics.push(Diagnostic::error(
                    format!("failed to read `{}`: {err}", canonical.display()),
                    Span::EMPTY,
                ));
                self.visiting.remove(&canonical);
                return;
            }
        };

        let source = SourceFile::new(canonical.to_string_lossy(), contents);
        let parsed = parse_program(&source.contents);
        self.diagnostics.merge(parsed.diagnostics);

        let Some(program) = parsed.program else {
            self.visiting.remove(&canonical);
            return;
        };

        for item in &program.items {
            if let Item::Import(import) = &item.node {
                let dep = resolve_import_path(&import.path.node, &canonical);
                self.load_recursive(&dep);
            }
        }

        self.sources.insert(canonical.clone(), source);
        self.loaded.insert(canonical.clone(), program);
        self.visiting.remove(&canonical);
    }

    fn merge_program(&self) -> Program {
        let entry_canonical = fs::canonicalize(&self.entry).unwrap_or(self.entry.clone());
        let Some(entry_program) = self.loaded.get(&entry_canonical) else {
            return Program {
                module: None,
                items: Vec::new(),
                span: Span::EMPTY,
            };
        };

        let mut merged_items = Vec::new();
        let mut seen = HashSet::new();

        for (path, program) in &self.loaded {
            if path == &entry_canonical {
                continue;
            }
            for item in &program.items {
                if let Some(name) = exported_name(&item.node) {
                    if seen.insert(name.clone()) {
                        merged_items.push(clone_exported(item));
                    }
                }
            }
        }

        for item in &entry_program.items {
            if matches!(item.node, Item::Import(_)) {
                continue;
            }
            if let Some(name) = exported_name(&item.node) {
                seen.insert(name);
            }
            merged_items.push(item.clone());
        }

        Program {
            module: entry_program.module.clone(),
            items: merged_items,
            span: entry_program.span,
        }
    }
}

fn resolve_import_path(import_path: &str, from_file: &Path) -> PathBuf {
    let clean = import_path.trim_matches('"');

    if let Some(stripped) = clean.strip_prefix("std/") {
        if let Some(std_root) = nasaq_registry::find_std_root(from_file) {
            let mut candidate = std_root.join(stripped);
            if candidate.extension().is_none() {
                candidate.set_extension(nasaq_syntax::SOURCE);
            }
            return candidate;
        }
    }

    if !clean.starts_with('.') && !clean.contains('/') {
        if let Some(root) = nasaq_registry::find_project_root(from_file) {
            if let Some(entry) = nasaq_registry::package_entry(&root, clean) {
                return entry;
            }
        }
    }

    let base = from_file.parent().unwrap_or_else(|| Path::new("."));
    let mut candidate = base.join(clean);
    if candidate.extension().is_none() {
        let nq = candidate.with_extension(nasaq_syntax::SOURCE);
        if nq.exists() {
            candidate = nq;
        } else {
            candidate.set_extension(nasaq_syntax::SOURCE_LEGACY);
        }
    }
    candidate
}

fn exported_name(item: &Item) -> Option<String> {
    match item {
        Item::Function(f) if f.exported => Some(f.name.node.clone()),
        Item::Struct(s) if s.exported => Some(s.name.node.clone()),
        Item::Component(c) if c.exported => Some(c.name.node.clone()),
        Item::Export(inner) => exported_name(&inner.node),
        _ => None,
    }
}

fn clone_exported(item: &Spanned<Item>) -> Spanned<Item> {
    match &item.node {
        Item::Function(f) => Spanned::new(Item::Function(f.clone()), item.span),
        Item::Struct(s) => Spanned::new(Item::Struct(s.clone()), item.span),
        Item::Component(c) => Spanned::new(Item::Component(c.clone()), item.span),
        Item::Export(inner) => clone_exported(&Spanned::new(*inner.node.clone(), inner.span)),
        other => Spanned::new(other.clone(), item.span),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_imported_module() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/import_demo/src/main.nq");
        if !root.exists() {
            return;
        }
        let loaded = load_program(&root);
        assert!(
            !loaded.diagnostics.has_errors(),
            "{:?}",
            loaded.diagnostics.diagnostics
        );
        let names: Vec<_> = loaded
            .program
            .items
            .iter()
            .filter_map(|i| exported_name(&i.node))
            .collect();
        assert!(names.iter().any(|n| n == "add"));
        assert!(names.iter().any(|n| n == "main"));
    }
}
