# Nasaq — نَسَق

**The independent web language.** Own compiler · own extensions · own ecosystem.

| | Nasaq | TypeScript |
|---|-------|------------|
| Extension | **`.nq`** | `.ts` |
| UI | **`view {}` in language** | JSX + React |
| Runtime file | **`.nqr`** | `.js` |
| Package tool | **`nasaq add`** | npm |

## Quick start

```bash
git clone https://github.com/watad567-afk/nasaq
cd nasaq
cargo build --release -p nasaq_cli --target-dir target3
./target3/release/nasaq new myapp --template web
cd myapp
../target3/release/nasaq dev .
```

## Global toolchain

```bash
nasaq new / init / build / run / test / dev / ssr / wasm
nasaq add http          # registry package
nasaq search            # list packages
nasaq install           # vendor deps
nasaq bench             # compiler benchmarks
nasaq website           # official .nq site
```

## Project files

```
myapp/
  nasaq.toml
  src/main.nq           # entry
  src/App.nq            # component
  std/                  # standard library (repo)
  vendor/               # installed packages
  dist/myapp.nq         # compiled output
```

## Standard library

```nq
import "std/math";
export fn main() {
    println_int(add(10, 32))
}
```

## Community

- [Contributing](CONTRIBUTING.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Language Spec](docs/SPEC.md)
- [Global Roadmap](docs/GLOBAL-ROADMAP.md)
- [Architecture](docs/ARCHITECTURE.md)

## Status — v1.0 Global MVP

Compiler · `.nq` branding · std · registry · LSP · SSR · website · benchmarks · **100+ conformance tests** · **component imports** · **@nasaq/lang npm**

**License:** Apache-2.0 OR MIT
