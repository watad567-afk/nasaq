use nasaq_ast::{
    AssignOp, BinOp, Block, Expr, ExternFn, FnDecl, Item, Literal, Pattern, Program, Stmt, StructDecl,
    UnaryOp,
};
use nasaq_hir::HirModule;
use nasaq_syntax::Spanned;

#[derive(Debug, Clone)]
pub struct CodegenOptions {
    pub module_name: String,
    pub runtime_import: String,
    pub dom_runtime_import: String,
    pub source_map: bool,
    pub web_mount: Option<(String, String)>,
    pub hydrate: bool,
}

impl Default for CodegenOptions {
    fn default() -> Self {
        Self {
            module_name: "nasaq_module".into(),
            runtime_import: "./runtime/core.nqr".to_string(),
            dom_runtime_import: "./runtime/dom.nqr".to_string(),
            source_map: true,
            web_mount: None,
            hydrate: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedModule {
    pub js: String,
    pub source_map: Option<String>,
    pub entry: Option<String>,
}

pub fn emit_module(hir: &HirModule, options: &CodegenOptions) -> GeneratedModule {
    let mut emitter = Emitter::new(options);
    emitter.emit_program(&hir.program);
    GeneratedModule {
        js: emitter.output,
        source_map: emitter.source_map,
        entry: emitter.entry,
    }
}

struct Emitter {
    options: CodegenOptions,
    output: String,
    indent: usize,
    runtime_symbols: Vec<String>,
    needs_dom: bool,
    entry: Option<String>,
    source_map: Option<String>,
}

impl Emitter {
    fn new(options: &CodegenOptions) -> Self {
        Self {
            options: options.clone(),
            output: String::new(),
            indent: 0,
            runtime_symbols: Vec::new(),
            needs_dom: false,
            entry: None,
            source_map: None,
        }
    }

    fn emit_program(&mut self, program: &Program) {
        for item in &program.items {
            self.collect_runtime(&item.node);
        }

        self.writeln(&format!(
            "import {{ {} }} from '{}';",
            CORE_RUNTIME, self.options.runtime_import
        ));

        if self.needs_dom {
            let dom_imports = if self.options.web_mount.is_some() {
                if self.options.hydrate {
                    "createSignal, effect, hydrateComponent"
                } else {
                    "createSignal, effect, mountComponent"
                }
            } else {
                "createSignal, effect"
            };
            self.writeln(&format!(
                "import {{ {} }} from '{}';",
                dom_imports, self.options.dom_runtime_import
            ));
        }

        if self.needs_dom {
            self.output
                .push_str(&nasaq_dom::compile_program_components(program));
        }

        self.writeln("");

        for item in &program.items {
            self.emit_item(&item.node);
        }

        if let Some((component, selector)) = &self.options.web_mount {
            let mount_fn = if self.options.hydrate {
                "hydrateComponent"
            } else {
                "mountComponent"
            };
            self.writeln(&format!(
                "{mount_fn}({}, \"{}\");",
                component, escape_js_string(selector)
            ));
        } else if self.entry.is_some() {
            self.writeln("main();");
        }

        if self.options.source_map {
            self.source_map = Some(format!(
                "{{\"version\":3,\"file\":\"{}.{}\",\"sources\":[\"{}.{}\"],\"mappings\":\"\"}}",
                self.options.module_name,
                nasaq_syntax::OUTPUT,
                self.options.module_name,
                nasaq_syntax::SOURCE,
            ));
        }
    }

    fn collect_runtime(&mut self, item: &Item) {
        match item {
            Item::Extern(ext) => {
                if let Some(runtime) = runtime_binding(&ext.name.node) {
                    self.runtime_symbols.push(runtime.to_string());
                }
            }
            Item::Export(inner) => self.collect_runtime(&inner.node),
            Item::Component(_) => self.needs_dom = true,
            _ => {}
        }
    }

    fn emit_item(&mut self, item: &Item) {
        match item {
            Item::Function(f) => self.emit_function(f),
            Item::Extern(_) => {}
            Item::Struct(s) => self.emit_struct(s),
            Item::Component(_) => {}
            Item::Import(_) => {}
            Item::Export(inner) => self.emit_item(&inner.node),
        }
    }

    fn emit_function(&mut self, func: &FnDecl) {
        let name = &func.name.node;
        let params = func
            .params
            .iter()
            .map(|p| p.node.name.node.clone())
            .collect::<Vec<_>>()
            .join(", ");

        if func.exported {
            self.write("export ");
            if name == "main" {
                self.entry = Some(name.clone());
            }
        }

        self.writeln(&format!("function {name}({params}) {{"));
        self.indent += 1;
        self.emit_block(&func.body);
        self.indent -= 1;
        self.writeln("}");
        self.writeln("");
    }

    fn emit_struct(&mut self, st: &StructDecl) {
        let name = &st.name.node;
        if st.exported {
            self.write("export ");
        }
        self.writeln(&format!("function {name}(fields) {{"));
        self.indent += 1;
        for field in &st.fields {
            self.writeln(&format!(
                "this.{} = fields.{};",
                field.node.name.node, field.node.name.node
            ));
        }
        self.indent -= 1;
        self.writeln("}");
        self.writeln("");
    }

    fn emit_block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.emit_stmt(&stmt.node);
        }
    }

    fn emit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let {
                mutable,
                name,
                init,
                ..
            } => {
                let kw = if *mutable { "let" } else { "const" };
                self.write_indent();
                self.output
                    .push_str(&format!("{} {} = ", kw, name.node));
                self.emit_expr(&init.node);
                self.writeln(";");
            }
            Stmt::Expr(expr) => {
                self.write_indent();
                self.emit_expr(&expr.node);
                self.writeln(";");
            }
            Stmt::Return { value, .. } => {
                self.write_indent();
                self.output.push_str("return");
                if let Some(value) = value {
                    self.output.push(' ');
                    self.emit_expr(&value.node);
                }
                self.writeln(";");
            }
            Stmt::If {
                cond,
                then_block,
                else_block,
                ..
            } => {
                self.write_indent();
                self.output.push_str("if (");
                self.emit_expr(&cond.node);
                self.output.push_str(") ");
                self.emit_block_stmt(then_block);
                if let Some(else_block) = else_block {
                    self.output.push_str(" else ");
                    self.emit_block_stmt(else_block);
                }
                self.writeln("");
            }
            Stmt::While { cond, body, .. } => {
                self.write_indent();
                self.output.push_str("while (");
                self.emit_expr(&cond.node);
                self.output.push_str(") ");
                self.emit_block_stmt(body);
                self.writeln("");
            }
            Stmt::Break { .. } => self.writeln("break;"),
            Stmt::Continue { .. } => self.writeln("continue;"),
        }
    }

    fn emit_block_stmt(&mut self, block: &Block) {
        self.writeln("{");
        self.indent += 1;
        self.emit_block(block);
        self.indent -= 1;
        self.write_indent();
        self.output.push('}');
    }

    fn emit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(lit) => self.emit_literal(&lit.node),
            Expr::Ident(name) => self.output.push_str(&name.node),
            Expr::Binary { op, left, right, .. } => {
                self.output.push('(');
                self.emit_expr(&left.node);
                self.output.push(' ');
                self.output.push_str(binop(op));
                self.output.push(' ');
                self.emit_expr(&right.node);
                self.output.push(')');
            }
            Expr::Unary { op, expr, .. } => {
                self.output.push('(');
                self.output.push_str(unary_op(op));
                self.emit_expr(&expr.node);
                self.output.push(')');
            }
            Expr::Call { callee, args, .. } => {
                if let Expr::Ident(name) = &callee.node {
                    if let Some(runtime) = runtime_binding(&name.node) {
                        self.output.push_str(runtime);
                    } else {
                        self.output.push_str(&name.node);
                    }
                } else {
                    self.emit_expr(&callee.node);
                }
                self.output.push('(');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.emit_expr(&arg.node);
                }
                self.output.push(')');
            }
            Expr::Assign { target, op, value, .. } => {
                self.emit_expr(&target.node);
                self.output.push(' ');
                self.output.push_str(match op {
                    AssignOp::Assign => "=",
                    AssignOp::AddAssign => "+=",
                    AssignOp::SubAssign => "-=",
                });
                self.output.push(' ');
                self.emit_expr(&value.node);
            }
            Expr::Field { object, field, .. } => {
                self.emit_expr(&object.node);
                self.output.push('.');
                self.output.push_str(&field.node);
            }
            Expr::Index { object, index, .. } => {
                self.emit_expr(&object.node);
                self.output.push('[');
                self.emit_expr(&index.node);
                self.output.push(']');
            }
            Expr::Block(block) => {
                self.output.push_str("(() => { ");
                for stmt in &block.stmts {
                    self.emit_stmt(&stmt.node);
                }
                self.output.push_str(" })()");
            }
            Expr::If {
                cond,
                then_block,
                else_expr,
                ..
            } => {
                self.output.push('(');
                self.emit_expr(&cond.node);
                self.output.push_str(" ? (");
                self.emit_block_expr(then_block);
                self.output.push_str(") : (");
                if let Some(else_expr) = else_expr {
                    self.emit_expr(&else_expr.node);
                } else {
                    self.output.push_str("undefined");
                }
                self.output.push_str("))");
            }
            Expr::Match { scrutinee, arms, .. } => {
                self.output.push_str("(((__s) => { ");
                for (i, arm) in arms.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str("else ");
                    }
                    self.output.push_str("if (");
                    self.emit_match_condition(&arm.node.pattern.node);
                    self.output.push_str(") { return ");
                    self.emit_expr(&arm.node.body.node);
                    self.output.push_str("; } ");
                }
                self.output.push_str("return undefined; })(");
                self.emit_expr(&scrutinee.node);
                self.output.push_str("))");
            }
            Expr::StructLit { name, fields, .. } => {
                self.output.push_str(&format!("new {}({{", name.node));
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(&field.node.name.node);
                    self.output.push_str(": ");
                    self.emit_expr(&field.node.value.node);
                }
                self.output.push_str("})");
            }
            Expr::Array(elems, _) => {
                self.output.push('[');
                for (i, elem) in elems.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.emit_expr(&elem.node);
                }
                self.output.push(']');
            }
            Expr::Tuple(elems, _) => {
                self.output.push('[');
                for (i, elem) in elems.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.emit_expr(&elem.node);
                }
                self.output.push(']');
            }
            Expr::Group(inner, _) => self.emit_expr(&inner.node),
        }
    }

    fn emit_block_expr(&mut self, block: &Block) {
        if block.stmts.is_empty() {
            self.output.push_str("undefined");
            return;
        }
        if let Stmt::Expr(expr) = &block.stmts.last().unwrap().node {
            for stmt in &block.stmts[..block.stmts.len() - 1] {
                self.emit_stmt(&stmt.node);
            }
            self.emit_expr(&expr.node);
        } else {
            self.output.push_str("(() => { ");
            self.emit_block(block);
            self.output.push_str(" return undefined; })()");
        }
    }

    fn emit_literal(&mut self, lit: &Literal) {
        match lit {
            Literal::Int(v) => self.output.push_str(&v.to_string()),
            Literal::Float(v) => self.output.push_str(v),
            Literal::String(v) => {
                self.output.push('"');
                for ch in v.chars() {
                    match ch {
                        '"' => self.output.push_str("\\\""),
                        '\\' => self.output.push_str("\\\\"),
                        '\n' => self.output.push_str("\\n"),
                        '\r' => self.output.push_str("\\r"),
                        '\t' => self.output.push_str("\\t"),
                        c => self.output.push(c),
                    }
                }
                self.output.push('"');
            }
            Literal::Char(c) => {
                self.output.push('\'');
                self.output.push(*c);
                self.output.push('\'');
            }
            Literal::Bool(b) => self.output.push_str(if *b { "true" } else { "false" }),
        }
    }

    fn write_indent(&mut self) {
        self.output.push_str(&"    ".repeat(self.indent));
    }

    fn write(&mut self, s: &str) {
        self.write_indent();
        self.output.push_str(s);
    }

    fn writeln(&mut self, s: &str) {
        self.write(s);
        self.output.push('\n');
    }

    fn emit_match_condition(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Wildcard => self.output.push_str("true"),
            Pattern::Literal(lit) => {
                self.output.push_str("Object.is(__s, ");
                self.emit_literal(&lit.node);
                self.output.push(')');
            }
            Pattern::Ident { .. } | Pattern::Struct { .. } => self.output.push_str("true"),
        }
    }
}

fn escape_js_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

const CORE_RUNTIME: &str =
    "println, println_int, println_str, assert_eq, __str_len, __json_stringify, __json_parse";

fn binop(op: &BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::Mod => "%",
        BinOp::Eq => "===",
        BinOp::Ne => "!==",
        BinOp::Lt => "<",
        BinOp::Le => "<=",
        BinOp::Gt => ">",
        BinOp::Ge => ">=",
        BinOp::And => "&&",
        BinOp::Or => "||",
    }
}

fn unary_op(op: &UnaryOp) -> &'static str {
    match op {
        UnaryOp::Neg => "-",
        UnaryOp::Not => "!",
    }
}

fn runtime_binding(name: &str) -> Option<&'static str> {
    match name {
        "println" | "print" | "log" => Some("println"),
        "println_int" => Some("println_int"),
        "println_str" => Some("println_str"),
        "assert_eq" => Some("assert_eq"),
        "__str_len" => Some("__str_len"),
        "__json_stringify" => Some("__json_stringify"),
        "__json_parse" => Some("__json_parse"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nasaq_hir::lower;
    use nasaq_parser::parse_program;

    #[test]
    fn emits_hello_world_js() {
        let source = r#"
            module hello
            extern fn println(value: String);

            export fn main() {
                println("Hello, Nasaq!");
            }
        "#;
        let parsed = parse_program(source).program.unwrap();
        let hir = lower(parsed);
        let out = emit_module(&hir, &CodegenOptions::default());
        assert!(out.js.contains("println"));
        assert!(out.js.contains("Hello, Nasaq!"));
        assert!(out.js.contains("export function main"));
    }
}
