# Nasaq Architecture

See the repository [README](../README.md) for quick start. This document describes the compiler and platform architecture.

## Pipeline

```
Source (.nasaq) → Lexer → Parser → AST → Resolver → Type checker → HIR → JS Codegen → dist/
```

## Crates

| Crate | Role |
|-------|------|
| `nasaq_lexer` | Tokenization with spans |
| `nasaq_parser` | Recursive descent + Pratt |
| `nasaq_ast` | AST nodes |
| `nasaq_diagnostics` | Human-friendly errors |
| `nasaq_resolver` | Symbol tables |
| `nasaq_types` | Static type checking |
| `nasaq_hir` | Typed IR |
| `nasaq_codegen_js` | ESM output |
| `nasaq_runtime` | Official JS runtime |
| `nasaq_cli` | `nasaq` command |

Future: `nasaq_dom`, `nasaq_ssr`, `nasaq_lsp`, `nasaq_codegen_wasm`.

## Independence guarantee

Nasaq does **not** translate syntax into React components or depend on Babel/TypeScript as the language core. JavaScript is an **output target**, not the semantic definition of the language.

## Nasaq Web (Phase 2)

Components with `state`, `view`, and scoped `style` compile to signal-driven DOM updates without a virtual DOM by default.
