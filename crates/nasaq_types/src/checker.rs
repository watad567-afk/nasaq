use indexmap::IndexMap;
use nasaq_ast::{
    BinOp, ComponentDecl, Expr, FnDecl, Item, Literal, Pattern, Program, Stmt, UnaryOp,
};
use nasaq_diagnostics::{Diagnostic, DiagnosticBag};
use nasaq_syntax::Spanned;

use crate::types::Ty;

pub struct TypeCheckResult {
    pub diagnostics: DiagnosticBag,
}

pub fn typecheck(program: &Program) -> TypeCheckResult {
    let mut checker = Checker::default();
    checker.check_program(program);
    TypeCheckResult {
        diagnostics: checker.diagnostics,
    }
}

#[derive(Default)]
struct Checker {
    diagnostics: DiagnosticBag,
    functions: IndexMap<String, FunctionSig>,
    structs: IndexMap<String, StructSig>,
    components: IndexMap<String, ComponentSig>,
    scopes: Vec<IndexMap<String, Binding>>,
}

#[derive(Clone)]
struct ComponentSig {
    params: IndexMap<String, Ty>,
}

#[derive(Clone)]
struct FunctionSig {
    params: Vec<(String, Ty)>,
    ret: Ty,
}

#[derive(Clone)]
struct StructSig {
    fields: IndexMap<String, Ty>,
}

#[derive(Clone)]
struct Binding {
    ty: Ty,
    mutable: bool,
}

impl Checker {
    fn check_program(&mut self, program: &Program) {
        for item in &program.items {
            self.collect_item(&item.node);
        }
        for item in &program.items {
            self.check_item(&item.node);
        }
    }

    fn collect_item(&mut self, item: &Item) {
        match item {
            Item::Function(f) => {
                self.functions.insert(
                    f.name.node.clone(),
                    FunctionSig {
                        params: f
                            .params
                            .iter()
                            .map(|p| (p.node.name.node.clone(), Ty::from_ast(&p.node.ty.node)))
                            .collect(),
                        ret: f
                            .return_type
                            .as_ref()
                            .map(|t| Ty::from_ast(&t.node))
                            .unwrap_or(Ty::Void),
                    },
                );
            }
            Item::Struct(s) => {
                let mut fields = IndexMap::new();
                for field in &s.fields {
                    fields.insert(
                        field.node.name.node.clone(),
                        Ty::from_ast(&field.node.ty.node),
                    );
                }
                self.structs.insert(s.name.node.clone(), StructSig { fields });
            }
            Item::Component(c) => {
                let mut params = IndexMap::new();
                for param in &c.params {
                    params.insert(
                        param.node.name.node.clone(),
                        Ty::from_ast(&param.node.ty.node),
                    );
                }
                self.components.insert(c.name.node.clone(), ComponentSig { params });
            }
            Item::Export(inner) => self.collect_item(&inner.node),
            Item::Extern(f) => {
                self.functions.insert(
                    f.name.node.clone(),
                    FunctionSig {
                        params: f
                            .params
                            .iter()
                            .map(|p| (p.node.name.node.clone(), Ty::from_ast(&p.node.ty.node)))
                            .collect(),
                        ret: f
                            .return_type
                            .as_ref()
                            .map(|t| Ty::from_ast(&t.node))
                            .unwrap_or(Ty::Void),
                    },
                );
            }
            Item::Import(_) => {}
        }
    }

    fn check_item(&mut self, item: &Item) {
        match item {
            Item::Function(f) => self.check_function(f),
            Item::Extern(f) => {
                self.functions.insert(
                    f.name.node.clone(),
                    FunctionSig {
                        params: f
                            .params
                            .iter()
                            .map(|p| (p.node.name.node.clone(), Ty::from_ast(&p.node.ty.node)))
                            .collect(),
                        ret: f
                            .return_type
                            .as_ref()
                            .map(|t| Ty::from_ast(&t.node))
                            .unwrap_or(Ty::Void),
                    },
                );
            }
            Item::Struct(_) => {}
            Item::Component(c) => self.check_component(c),
            Item::Import(_) => {}
            Item::Export(inner) => self.check_item(&inner.node),
        }
    }

    fn check_function(&mut self, func: &FnDecl) {
        self.scopes.push(IndexMap::new());
        for param in &func.params {
            self.bind(
                &param.node.name.node,
                Ty::from_ast(&param.node.ty.node),
                false,
            );
        }
        for stmt in &func.body.stmts {
            self.check_stmt(&stmt.node);
        }
        self.scopes.pop();
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let {
                mutable,
                name,
                ty,
                init,
            } => {
                let init_ty = self.check_expr(&init.node);
                let binding_ty = if let Some(explicit) = ty {
                    let explicit_ty = Ty::from_ast(&explicit.node);
                    if !self.compatible(&explicit_ty, &init_ty) {
                        self.error(
                            format!(
                                "expected `{}`, found `{}`",
                                self.ty_name(&explicit_ty),
                                self.ty_name(&init_ty)
                            ),
                            init.span,
                        );
                    }
                    explicit_ty
                } else {
                    init_ty
                };
                self.bind(&name.node, binding_ty, *mutable);
            }
            Stmt::Expr(expr) => {
                self.check_expr(&expr.node);
            }
            Stmt::Return { value, span } => {
                if let Some(value) = value {
                    self.check_expr(&value.node);
                }
                let _ = span;
            }
            Stmt::If {
                cond,
                then_block,
                else_block,
                ..
            } => {
                let cond_ty = self.check_expr(&cond.node);
                if cond_ty != Ty::Bool {
                    self.error("condition must be Bool", cond.span);
                }
                self.check_block(then_block);
                if let Some(else_block) = else_block {
                    self.check_block(else_block);
                }
            }
            Stmt::While { cond, body, .. } => {
                let cond_ty = self.check_expr(&cond.node);
                if cond_ty != Ty::Bool {
                    self.error("loop condition must be Bool", cond.span);
                }
                self.check_block(body);
            }
            Stmt::Break { .. } | Stmt::Continue { .. } => {}
        }
    }

    fn check_block(&mut self, block: &nasaq_ast::Block) {
        self.scopes.push(IndexMap::new());
        for stmt in &block.stmts {
            self.check_stmt(&stmt.node);
        }
        self.scopes.pop();
    }

    fn check_expr(&mut self, expr: &Expr) -> Ty {
        match expr {
            Expr::Literal(lit) => self.literal_ty(&lit.node),
            Expr::Ident(name) => self
                .lookup(&name.node)
                .map(|b| b.ty.clone())
                .unwrap_or_else(|| {
                    self.error(format!("unknown variable `{}`", name.node), name.span);
                    Ty::Unknown
                }),
            Expr::Binary { op, left, right, span } => {
                let l = self.check_expr(&left.node);
                let r = self.check_expr(&right.node);
                match op {
                    BinOp::Add if l == Ty::String || r == Ty::String => Ty::String,
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                        if !l.is_numeric() || !r.is_numeric() {
                            self.error("numeric operands required", *span);
                        }
                        if l == Ty::Float || r == Ty::Float {
                            Ty::Float
                        } else {
                            Ty::Int
                        }
                    }
                    BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                        Ty::Bool
                    }
                    BinOp::And | BinOp::Or => {
                        if l != Ty::Bool || r != Ty::Bool {
                            self.error("logical operands must be Bool", *span);
                        }
                        Ty::Bool
                    }
                }
            }
            Expr::Unary { op, expr, span } => {
                let inner = self.check_expr(&expr.node);
                match op {
                    UnaryOp::Neg if inner.is_numeric() => inner,
                    UnaryOp::Neg => {
                        self.error("unary `-` requires numeric operand", *span);
                        Ty::Unknown
                    }
                    UnaryOp::Not if inner == Ty::Bool => Ty::Bool,
                    UnaryOp::Not => {
                        self.error("unary `!` requires Bool operand", *span);
                        Ty::Unknown
                    }
                }
            }
            Expr::Call { callee, args, span } => {
                if let Expr::Ident(name) = &callee.node {
                    if let Some(sig) = self.functions.get(&name.node).cloned() {
                        if sig.params.len() != args.len() {
                            self.error(
                                format!(
                                    "function `{}` expects {} arguments, found {}",
                                    name.node,
                                    sig.params.len(),
                                    args.len()
                                ),
                                *span,
                            );
                        }
                        for (arg, (_, expected)) in args.iter().zip(sig.params.iter()) {
                            let arg_ty = self.check_expr(&arg.node);
                            if !self.compatible(expected, &arg_ty) {
                                self.error(
                                    format!(
                                        "expected `{}`, found `{}`",
                                        self.ty_name(expected),
                                        self.ty_name(&arg_ty)
                                    ),
                                    arg.span,
                                );
                            }
                        }
                        return sig.ret;
                    }
                }
                self.error("call to unknown function", *span);
                Ty::Unknown
            }
            Expr::Assign { target, value, op, span } => {
                if let Expr::Ident(name) = &target.node {
                    if let Some(binding) = self.lookup(&name.node).cloned() {
                        if !binding.mutable {
                            self.error(
                                format!("cannot assign to immutable binding `{}`", name.node),
                                name.span,
                            )
                            .with_suggestion(format!("declare `{0}` with `let mut {0}`", name.node));
                        } else {
                            let val_ty = self.check_expr(&value.node);
                            if !self.compatible(&binding.ty, &val_ty) {
                                self.error(
                                    format!(
                                        "expected `{}`, found `{}`",
                                        self.ty_name(&binding.ty),
                                        self.ty_name(&val_ty)
                                    ),
                                    value.span,
                                );
                            }
                            return binding.ty;
                        }
                    } else {
                        self.error(format!("unknown variable `{}`", name.node), name.span);
                    }
                } else {
                    self.error("invalid assignment target", *span);
                }
                Ty::Void
            }
            Expr::Field { object, field, span } => {
                let obj_ty = self.check_expr(&object.node);
                if let Ty::Named(name) = obj_ty {
                    if let Some(st) = self.structs.get(&name) {
                        return st
                            .fields
                            .get(&field.node)
                            .cloned()
                            .unwrap_or_else(|| {
                                self.error(
                                    format!("struct `{name}` has no field `{}`", field.node),
                                    *span,
                                );
                                Ty::Unknown
                            });
                    }
                }
                self.error("field access on non-struct value", *span);
                Ty::Unknown
            }
            Expr::Index { object, index, span } => {
                let _ = self.check_expr(&object.node);
                let idx = self.check_expr(&index.node);
                if idx != Ty::Int {
                    self.error("array index must be Int", index.span);
                }
                Ty::Unknown
            }
            Expr::Block(block) => {
                self.check_block(block);
                Ty::Void
            }
            Expr::If { cond, then_block, else_expr, .. } => {
                let cond_ty = self.check_expr(&cond.node);
                if cond_ty != Ty::Bool {
                    self.error("if condition must be Bool", cond.span);
                }
                self.check_block(then_block);
                if let Some(else_expr) = else_expr {
                    self.check_expr(&else_expr.node)
                } else {
                    Ty::Void
                }
            }
            Expr::Match { scrutinee, arms, span } => {
                let subject = self.check_expr(&scrutinee.node);
                let mut arm_tys = Vec::new();
                for arm in arms {
                    self.scopes.push(IndexMap::new());
                    self.check_pattern(&arm.node.pattern.node, &subject);
                    if let Some(guard) = &arm.node.guard {
                        let guard_ty = self.check_expr(&guard.node);
                        if guard_ty != Ty::Bool {
                            self.error("match guard must be Bool", guard.span);
                        }
                    }
                    arm_tys.push(self.check_expr(&arm.node.body.node));
                    self.scopes.pop();
                }
                arm_tys
                    .into_iter()
                    .reduce(|a, b| if a == b { a } else { Ty::Unknown })
                    .unwrap_or(Ty::Void)
            }
            Expr::StructLit { name, fields, span } => {
                if let Some(st) = self.structs.get(&name.node).cloned() {
                    for field in fields {
                        if let Some(expected) = st.fields.get(&field.node.name.node) {
                            let expected = expected.clone();
                            let value_ty = self.check_expr(&field.node.value.node);
                            if !self.compatible(&expected, &value_ty) {
                                self.error(
                                    format!(
                                        "field `{}` expected `{}`, found `{}`",
                                        field.node.name.node,
                                        self.ty_name(&expected),
                                        self.ty_name(&value_ty)
                                    ),
                                    field.node.value.span,
                                );
                            }
                        } else {
                            self.error(
                                format!("struct `{}` has no field `{}`", name.node, field.node.name.node),
                                field.node.name.span,
                            );
                        }
                    }
                    Ty::Named(name.node.clone())
                } else {
                    self.error(format!("unknown struct `{}`", name.node), *span);
                    Ty::Unknown
                }
            }
            Expr::Array(_, _) | Expr::Tuple(_, _) | Expr::Group(_, _) => Ty::Unknown,
        }
    }

    fn literal_ty(&self, lit: &Literal) -> Ty {
        match lit {
            Literal::Int(_) => Ty::Int,
            Literal::Float(_) => Ty::Float,
            Literal::String(_) => Ty::String,
            Literal::Char(_) => Ty::Char,
            Literal::Bool(_) => Ty::Bool,
        }
    }

    fn bind(&mut self, name: &str, ty: Ty, mutable: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), Binding { ty, mutable });
        }
    }

    fn lookup(&self, name: &str) -> Option<&Binding> {
        for scope in self.scopes.iter().rev() {
            if let Some(binding) = scope.get(name) {
                return Some(binding);
            }
        }
        None
    }

    fn check_component(&mut self, component: &ComponentDecl) {
        self.scopes.push(IndexMap::new());
        for param in &component.params {
            self.bind(
                &param.node.name.node,
                Ty::from_ast(&param.node.ty.node),
                false,
            );
        }
        for state in &component.states {
            let init_ty = self.check_expr(&state.node.init.node);
            let binding_ty = if let Some(explicit) = &state.node.ty {
                let explicit_ty = Ty::from_ast(&explicit.node);
                if !self.compatible(&explicit_ty, &init_ty) {
                    self.error(
                        format!(
                            "state `{}` expected `{}`, found `{}`",
                            state.node.name.node,
                            self.ty_name(&explicit_ty),
                            self.ty_name(&init_ty)
                        ),
                        state.node.init.span,
                    );
                }
                explicit_ty
            } else {
                init_ty
            };
            self.bind(&state.node.name.node, binding_ty, true);
        }
        if let Some(view) = &component.view {
            self.check_view_nodes(&view.node.nodes);
        }
        self.scopes.pop();
    }

    fn check_view_nodes(&mut self, nodes: &[nasaq_syntax::Spanned<nasaq_ast::ViewNode>]) {
        for node in nodes {
            match &node.node {
                nasaq_ast::ViewNode::Interpolation(expr) => {
                    self.check_expr(&expr.node);
                }
                nasaq_ast::ViewNode::Element(el) => {
                    if Self::is_component_tag(&el.tag.node) {
                        self.check_component_usage(el);
                    } else {
                        for attr in &el.attrs {
                            if let nasaq_ast::HtmlAttr::Event { handler, .. } = &attr.node {
                                self.check_expr(&handler.node);
                            } else if let nasaq_ast::HtmlAttr::Attribute { value, .. } = &attr.node {
                                if let nasaq_ast::AttrValue::Expr(expr) = &value.node {
                                    self.check_expr(&expr.node);
                                }
                            }
                        }
                        self.check_view_nodes(&el.children);
                    }
                }
                nasaq_ast::ViewNode::Text(_) => {}
            }
        }
    }

    fn check_component_usage(&mut self, el: &nasaq_ast::HtmlElement) {
        let name = &el.tag.node;
        let Some(sig) = self.components.get(name).cloned() else {
            self.error(format!("unknown component `{name}`"), el.span);
            return;
        };
        for attr in &el.attrs {
            match &attr.node {
                nasaq_ast::HtmlAttr::Attribute { name, value } => {
                    if let nasaq_ast::AttrValue::Expr(expr) = &value.node {
                        let found = self.check_expr(&expr.node);
                        if let Some(expected) = sig.params.get(&name.node) {
                            if !self.compatible(expected, &found) {
                                self.error(
                                    format!(
                                        "prop `{}` expected `{}`, found `{}`",
                                        name.node,
                                        self.ty_name(expected),
                                        self.ty_name(&found)
                                    ),
                                    expr.span,
                                );
                            }
                        }
                    }
                }
                nasaq_ast::HtmlAttr::Event { handler, .. } => {
                    self.check_expr(&handler.node);
                }
            }
        }
    }

    fn is_component_tag(tag: &str) -> bool {
        tag.chars()
            .next()
            .map(|c| c.is_ascii_uppercase())
            .unwrap_or(false)
    }

    fn check_pattern(&mut self, pattern: &Pattern, subject: &Ty) {
        match pattern {
            Pattern::Wildcard => {}
            Pattern::Literal(lit) => {
                let pat_ty = self.literal_ty(&lit.node);
                if !self.compatible(subject, &pat_ty) {
                    self.error(
                        format!(
                            "pattern type `{}` does not match scrutinee `{}`",
                            self.ty_name(&pat_ty),
                            self.ty_name(subject)
                        ),
                        lit.span,
                    );
                }
            }
            Pattern::Ident { name, mutable } => {
                self.bind(&name.node, subject.clone(), *mutable);
            }
            Pattern::Struct { name, fields } => {
                if let Ty::Named(struct_name) = subject {
                    if struct_name != &name.node {
                        self.error(
                            format!("expected struct `{struct_name}`, found `{}`", name.node),
                            name.span,
                        );
                    }
                }
                if let Some(st) = self.structs.get(&name.node).cloned() {
                    for (field_name, field_pat) in fields {
                        if let Some(field_ty) = st.fields.get(&field_name.node).cloned() {
                            self.check_pattern(&field_pat.node, &field_ty);
                        } else {
                            self.error(
                                format!("struct `{}` has no field `{}`", name.node, field_name.node),
                                field_name.span,
                            );
                        }
                    }
                } else {
                    self.error(format!("unknown struct `{}`", name.node), name.span);
                }
            }
        }
    }

    fn compatible(&self, expected: &Ty, found: &Ty) -> bool {
        expected == found
            || matches!(found, Ty::Unknown)
            || (matches!(expected, Ty::Int) && matches!(found, Ty::Float))
    }

    fn ty_name(&self, ty: &Ty) -> String {
        match ty {
            Ty::Int => "Int".into(),
            Ty::Float => "Float".into(),
            Ty::Bool => "Bool".into(),
            Ty::String => "String".into(),
            Ty::Char => "Char".into(),
            Ty::Void => "Void".into(),
            Ty::Unknown => "Unknown".into(),
            Ty::Named(n) => n.clone(),
            Ty::Option(inner) => format!("Option<{}>", self.ty_name(inner)),
            Ty::Result { ok, err } => format!(
                "Result<{}, {}>",
                self.ty_name(ok),
                self.ty_name(err)
            ),
            Ty::Function { .. } => "Function".into(),
            Ty::Struct { name, .. } => name.clone(),
        }
    }

    fn error(&mut self, message: impl Into<String>, span: nasaq_syntax::Span) -> Diagnostic {
        let diag = Diagnostic::error(message, span);
        self.diagnostics.push(diag.clone());
        diag
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nasaq_parser::parse_program;

    #[test]
    fn catches_immutable_assignment() {
        let parsed = parse_program(
            r#"
            export fn main() {
                let count = 0
                count = 1
            }
            "#,
        );
        let result = typecheck(parsed.program.as_ref().unwrap());
        assert!(result.diagnostics.has_errors());
    }

    #[test]
    fn catches_match_arm_types() {
        let parsed = parse_program(
            r#"
            export fn main() {
                let x = 1
                let y = match x {
                    1 => 10,
                    _ => 20,
                }
            }
            "#,
        );
        let result = typecheck(parsed.program.as_ref().unwrap());
        assert!(!result.diagnostics.has_errors(), "{:?}", result.diagnostics.diagnostics);
    }
}
