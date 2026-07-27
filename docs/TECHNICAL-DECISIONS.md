# Technical Decisions (Phase 0–1)

## Rust for the compiler
Memory safety, performance, static binaries, Wasm for playground. **Rejected:** C++ maintenance cost; TS-as-core (violates independence).

## JavaScript ESM primary output
Full browser/Node/npm compatibility without a custom VM.

## No LLVM in Phase 1
Web-first; Wasm optional later for pure functions.

## Hand-written lexer + RD parser
Control over diagnostics, recovery, and future view syntax.

## No React runtime
`@nasaq/runtime` is minimal JS. React adapter optional in Phase 4.

## Optional semicolons
Newlines may separate statements in blocks for ergonomics.

## Deferred bundler choice
SWC/Oxc/Rolldown only after benchmark ADR.

## Provisional naming
“Nasaq / نَسَق” pending trademark and npm scope verification.
