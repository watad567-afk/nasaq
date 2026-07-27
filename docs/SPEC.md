# Nasaq Language Specification v1.0

**Nasaq (نَسَق)** — independent web language. File extension: **`.nq`**

## File types

| Extension | Role |
|-----------|------|
| `.nq` | Source + compiled module |
| `.nqr` | Runtime library |
| `nasaq.toml` | Project manifest |
| `nq.pkg.json` | Package manifest |

## Syntax (summary)

```nq
module my_app
import "std/math";
import "./App";

export fn main() -> Int { return 42 }

export component App() {
    state count: Int = 0
    view { <button on:click={ count = count + 1 }>{ count }</button> }
    style scoped { button { cursor: pointer; } }
}
```

## Types

`Int`, `String`, `Bool`, `Option<T>`, `Result<T,E>`, user structs and components.

## Toolchain

| Command | Purpose |
|---------|---------|
| `nasaq new` | Create project |
| `nasaq build` | Compile to `.nq` |
| `nasaq run` | Execute |
| `nasaq dev` | Browser dev server |
| `nasaq add` | Registry package |
| `nasaq test` | Unit tests |
| `nasaq-lsp` | IDE diagnostics |

## Standard library

```nq
import "std/math";
let x = add(1, 2)
```

## Package registry

```bash
nasaq search
nasaq add http
nasaq install
```

Vendor path: `vendor/<name>/`

## Comparison positioning

| Feature | Nasaq | TypeScript+React |
|---------|-------|------------------|
| UI in language | ✅ `view {}` | ❌ JSX separate |
| Signals | ✅ built-in | ⚠️ libraries |
| Arabic/RTL | ✅ first-class | ⚠️ manual |
| Extension brand | `.nq` | `.ts`/`.tsx` |
| Own compiler | ✅ | ❌ (tsc) |
