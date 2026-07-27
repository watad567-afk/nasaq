# Contributing to Nasaq — نَسَق

Thank you for helping build an independent web language.

## Development setup

```bash
git clone https://github.com/nasaq-lang/nasaq
cd nasaq
cargo build --release -p nasaq_cli --target-dir target3
cargo test --workspace
```

Use `target3/release/nasaq.exe` if `target/release` is locked by a running dev server.

## Project layout

| Path | Purpose |
|------|---------|
| `crates/` | Rust compiler, LSP, CLI |
| `std/` | Standard library (`.nq`) |
| `registry/` | Package registry |
| `tests/conformance/` | Language conformance tests |
| `website/` | Official site (written in Nasaq) |
| `examples/` | Sample projects |

## Pull requests

1. Fork and create a feature branch
2. Run `cargo test --workspace`
3. Add conformance tests in `tests/conformance/` for language changes
4. Keep `.nq` as the primary extension in docs and examples
5. Open a PR with a clear description and test plan

## Code style

- Rust: follow existing crate patterns, minimal scope
- Nasaq: match examples in `showcase/` and `docs/SPEC.md`

## Community

- Issues: bug reports and feature requests welcome
- Discord: join when announced at [github.com/nasaq-lang/nasaq](https://github.com/nasaq-lang/nasaq)

## License

By contributing, you agree your code is licensed under Apache-2.0 OR MIT (same as the project).
