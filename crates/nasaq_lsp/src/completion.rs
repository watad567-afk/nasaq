//! LSP completion — keywords, snippets, and symbols from source.

use nasaq_ast::{Item, Program};
use nasaq_parser::parse_program;
use serde_json::{json, Value};

const KEYWORDS: &[(&str, &str)] = &[
    ("export", "keyword"),
    ("fn", "keyword"),
    ("component", "keyword"),
    ("state", "keyword"),
    ("view", "keyword"),
    ("style", "keyword"),
    ("scoped", "keyword"),
    ("import", "keyword"),
    ("extern", "keyword"),
    ("let", "keyword"),
    ("mut", "keyword"),
    ("if", "keyword"),
    ("else", "keyword"),
    ("while", "keyword"),
    ("return", "keyword"),
    ("match", "keyword"),
    ("struct", "keyword"),
    ("module", "keyword"),
    ("break", "keyword"),
    ("continue", "keyword"),
];

const SNIPPETS: &[(&str, &str, &str)] = &[
    (
        "component",
        "component",
        "export component ${1:Name}() {\n    state ${2:count}: Int = 0\n\n    view {\n        <div>{ $2 }</div>\n    }\n}",
    ),
    (
        "fn",
        "function",
        "export fn ${1:name}(${2:n}: Int) -> Int {\n    return $2\n}",
    ),
    (
        "view",
        "view block",
        "view {\n    <${1:div}>${2:content}</${1:div}>\n}",
    ),
    (
        "import",
        "import",
        "import \"${1:std/math}\";",
    ),
];

pub fn completion_items(source: &str, prefix: &str) -> Vec<Value> {
    let mut items = Vec::new();
    let prefix_lower = prefix.to_lowercase();

    for (label, kind) in KEYWORDS {
        if prefix.is_empty() || label.starts_with(prefix) || label.starts_with(&prefix_lower) {
            items.push(completion_item(label, kind, None));
        }
    }

    for (label, detail, insert) in SNIPPETS {
        if prefix.is_empty() || label.starts_with(prefix) {
            items.push(snippet_item(label, detail, insert));
        }
    }

    if let Some(program) = parse_program(source).program {
        collect_symbol_completions(&program, prefix, &mut items);
    }

    items
}

fn collect_symbol_completions(program: &Program, prefix: &str, items: &mut Vec<Value>) {
    for item in &program.items {
        symbol_from_item(&item.node, prefix, items);
    }
}

fn symbol_from_item(item: &Item, prefix: &str, items: &mut Vec<Value>) {
    match item {
        Item::Function(f) => {
            push_symbol(&f.name.node, "function", prefix, items);
        }
        Item::Component(c) => {
            push_symbol(&c.name.node, "class", prefix, items);
        }
        Item::Struct(s) => {
            push_symbol(&s.name.node, "struct", prefix, items);
        }
        Item::Export(inner) => symbol_from_item(&inner.node, prefix, items),
        _ => {}
    }
}

fn push_symbol(name: &str, kind: &str, prefix: &str, items: &mut Vec<Value>) {
    if prefix.is_empty() || name.starts_with(prefix) {
        items.push(completion_item(name, kind, Some(name)));
    }
}

fn completion_item(label: &str, kind: &str, insert: Option<&str>) -> Value {
    json!({
        "label": label,
        "kind": lsp_kind(kind),
        "insertText": insert.unwrap_or(label),
        "detail": kind
    })
}

fn snippet_item(label: &str, detail: &str, insert: &str) -> Value {
    json!({
        "label": label,
        "kind": 15,
        "detail": detail,
        "insertTextFormat": 2,
        "insertText": insert
    })
}

fn lsp_kind(kind: &str) -> u32 {
    match kind {
        "keyword" => 14,
        "function" => 3,
        "class" => 7,
        "struct" => 22,
        _ => 1,
    }
}

pub fn prefix_at_position(source: &str, line: u32, character: u32) -> String {
    let line_text = source.lines().nth(line as usize).unwrap_or("");
    let col = character as usize;
    let before = if col <= line_text.len() {
        &line_text[..col]
    } else {
        line_text
    };
    before
        .chars()
        .rev()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect::<String>()
        .chars()
        .rev()
        .collect()
}
