# Nasaq Type System (Initial)

## Primitives
`Int`, `Float`, `Bool`, `String`, `Char`, `Void`

## Planned
`Option<T>`, `Result<T,E>`, `Unknown`, generics, ADTs, enums, traits, async/await.

## Phase 1 implemented
- Primitive and named types
- Function signatures and return types
- Struct definitions and literals
- `let` / `let mut` with immutability errors
- Binary/unary operator checks
- Extern function call arity

## Principles
- Immutable by default
- No implicit `any`
- Inference on `let` when unannotated
- Human diagnostics with suggestions
