# Nasaq Grammar (Initial)

## Program

```
program     ::= module_decl? item*
module_decl ::= "module" ident
item        ::= "export"? (fn_decl | struct_decl | extern_decl | import_decl)
```

## Functions

```
fn_decl     ::= "fn" ident "(" params? ")" ("->" type)? block
extern_decl ::= "extern" fn_decl
params      ::= param ("," param)*
param       ::= ident ":" type ("=" expr)?
```

## Types

```
type ::= "Int" | "Float" | "Bool" | "String" | "Char" | "Void" | ident
       | "(" type ("," type)* ")" ("->" type)?
       | type "<" type ("," type)* ">"
```

## Statements

```
block       ::= "{" stmt* "}"
stmt        ::= let_stmt | return_stmt | if_stmt | while_stmt | expr_stmt
let_stmt    ::= "let" "mut"? ident (":" type)? "=" expr
return_stmt ::= "return" expr?
```

## Expressions

Precedence (low to high): assign, `||`, `&&`, equality, comparison, `+`/`-`, `*`/`/`/`%`, unary, call/member/index, primary.

## Future

```
component Counter(x: Int) { state ... view { ... } style scoped { ... } }
```

Not yet implemented in the bootstrap parser.
