# Self-hosting compiler (Phase 3)

Long-term goal: rewrite the Nasaq compiler in Nasaq itself.

## Bootstrap plan

1. **Lexer** — tokenize `.nq` source (`compiler/lexer.nq`)
2. **Parser** — build AST from tokens
3. **Typechecker** — validate programs
4. **Codegen** — emit `.nq` / `.nqr` modules
5. **Driver** — replace `nasaq_cli` entry point

The Rust compiler remains the reference implementation until bootstrap passes the full conformance suite.

## Current status

- `compiler/lexer.nq` — first bootstrap module (character classes + keywords)
- Rust crates in `crates/` — production compiler

## Run bootstrap (future)

```bash
nasaq run compiler/lexer.nq
```
