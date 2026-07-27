use std::collections::HashMap;

use nasaq_ast::{BinOp, Block, Expr, FnDecl, Item, Literal, Program, Stmt, UnaryOp};
use nasaq_syntax::Spanned;
use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction, Module,
    TypeSection, ValType,
};
use wasmprinter::print_bytes;

pub fn empty_module(name: &str) -> super::WasmOutput {
    let mut types = TypeSection::new();
    types.ty().function(vec![], vec![ValType::I32]);
    let mut functions = FunctionSection::new();
    functions.function(0);
    let mut exports = ExportSection::new();
    exports.export(&format!("{name}_init"), ExportKind::Func, 0);
    let mut codes = CodeSection::new();
    let mut body = Function::new(vec![]);
    body.instruction(&Instruction::I32Const(0));
    body.instruction(&Instruction::End);
    codes.function(&body);
    let mut module = Module::new();
    module.section(&types);
    module.section(&functions);
    module.section(&exports);
    module.section(&codes);
    let bytes = module.finish();
    let wat = print_bytes(&bytes).unwrap_or_else(|_| format!("(module (export \"{name}_init\" (func (result i32) i32.const 0)))"));
    super::WasmOutput { bytes, wat }
}

pub fn compile_program(program: &Program) -> super::WasmOutput {
    let mut fns: Vec<FnDecl> = Vec::new();
    for item in &program.items {
        collect_fns(&item.node, &mut fns);
    }
    if fns.is_empty() {
        let name = program
            .module
            .as_ref()
            .map(|m| m.node.name.node.clone())
            .unwrap_or_else(|| "nasaq_module".into());
        return empty_module(&name);
    }

    let mut types = TypeSection::new();
    let mut fn_types: HashMap<String, u32> = HashMap::new();
    for f in &fns {
        let param_count = f.params.len() as u32;
        let params = vec![ValType::I32; param_count as usize];
        let idx = types.len() as u32;
        types.ty().function(params, vec![ValType::I32]);
        fn_types.insert(f.name.node.clone(), idx);
    }

    let mut functions = FunctionSection::new();
    for f in &fns {
        let _ = f;
        functions.function(*fn_types.get(&f.name.node).unwrap_or(&0));
    }

    let mut fn_indices: HashMap<String, u32> = HashMap::new();
    for (i, f) in fns.iter().enumerate() {
        fn_indices.insert(f.name.node.clone(), i as u32);
    }

    let mut codes = CodeSection::new();
    for f in &fns {
        let body = compile_fn_body(f, &fn_indices);
        codes.function(&body);
    }

    let mut exports = ExportSection::new();
    for (i, f) in fns.iter().enumerate() {
        if f.exported {
            exports.export(&f.name.node, ExportKind::Func, i as u32);
        }
    }
    if exports.len() == 0 {
        exports.export("main", ExportKind::Func, 0);
    }

    let mut module = Module::new();
    module.section(&types);
    module.section(&functions);
    module.section(&exports);
    module.section(&codes);
    let bytes = module.finish();
    let wat = print_bytes(&bytes).unwrap_or_else(|_| "(module)".into());
    super::WasmOutput { bytes, wat }
}

fn collect_fns(item: &Item, out: &mut Vec<FnDecl>) {
    match item {
        Item::Function(f) => out.push(f.clone()),
        Item::Export(inner) => collect_fns(&inner.node, out),
        _ => {}
    }
}

fn compile_fn_body(f: &FnDecl, fn_indices: &HashMap<String, u32>) -> Function {
    let mut locals: HashMap<String, u32> = HashMap::new();
    for (i, p) in f.params.iter().enumerate() {
        locals.insert(p.node.name.node.clone(), i as u32);
    }
    let mut extra_locals: u32 = 0;
    let mut next_local = f.params.len() as u32;

    let mut body = Function::new(vec![]);
    let returned = emit_block_stmts(
        &f.body,
        &mut body,
        &mut locals,
        &mut extra_locals,
        &mut next_local,
        &f.params,
        fn_indices,
    );
    if returned.is_none() {
        body.instruction(&Instruction::I32Const(0));
    }
    body.instruction(&Instruction::End);
    let _ = extra_locals;
    body
}

fn emit_block_stmts(
    block: &Block,
    body: &mut Function,
    locals: &mut HashMap<String, u32>,
    extra_locals: &mut u32,
    next_local: &mut u32,
    params: &[Spanned<nasaq_ast::Param>],
    fn_indices: &HashMap<String, u32>,
) -> Option<()> {
    let mut returned = false;
    for stmt in &block.stmts {
        if returned {
            break;
        }
        match &stmt.node {
            Stmt::Let { name, init, .. } => {
                emit_expr(&init.node, body, locals, params, fn_indices);
                let idx = *next_local;
                *next_local += 1;
                *extra_locals += 1;
                locals.insert(name.node.clone(), idx);
                body.instruction(&Instruction::LocalSet(idx));
            }
            Stmt::Return { value, .. } => {
                if let Some(v) = value {
                    emit_expr(&v.node, body, locals, params, fn_indices);
                } else {
                    body.instruction(&Instruction::I32Const(0));
                }
                returned = true;
            }
            Stmt::Expr(expr) => {
                emit_expr(&expr.node, body, locals, params, fn_indices);
                body.instruction(&Instruction::Drop);
            }
            _ => {
                body.instruction(&Instruction::I32Const(0));
                body.instruction(&Instruction::Drop);
            }
        }
    }
    if returned { Some(()) } else { None }
}

fn emit_expr(
    expr: &Expr,
    body: &mut Function,
    locals: &HashMap<String, u32>,
    params: &[Spanned<nasaq_ast::Param>],
    fn_indices: &HashMap<String, u32>,
) {
    match expr {
        Expr::Literal(lit) => {
            match &lit.node {
                Literal::Int(v) => {
                    body.instruction(&Instruction::I32Const(*v as i32));
                }
                _ => {
                    body.instruction(&Instruction::I32Const(0));
                }
            }
        }
        Expr::Ident(name) => {
            if let Some(&idx) = locals.get(&name.node) {
                body.instruction(&Instruction::LocalGet(idx));
            } else if let Some(pos) = params.iter().position(|p| p.node.name.node == name.node) {
                body.instruction(&Instruction::LocalGet(pos as u32));
            } else {
                body.instruction(&Instruction::I32Const(0));
            }
        }
        Expr::Binary { op, left, right, .. } => {
            emit_expr(&left.node, body, locals, params, fn_indices);
            emit_expr(&right.node, body, locals, params, fn_indices);
            body.instruction(&match op {
                BinOp::Add => Instruction::I32Add,
                BinOp::Sub => Instruction::I32Sub,
                BinOp::Mul => Instruction::I32Mul,
                BinOp::Div => Instruction::I32DivS,
                BinOp::Mod => Instruction::I32RemS,
                BinOp::Eq => Instruction::I32Eq,
                BinOp::Ne => Instruction::I32Ne,
                BinOp::Lt => Instruction::I32LtS,
                BinOp::Le => Instruction::I32LeS,
                BinOp::Gt => Instruction::I32GtS,
                BinOp::Ge => Instruction::I32GeS,
                BinOp::And => Instruction::I32And,
                BinOp::Or => Instruction::I32Or,
            });
        }
        Expr::Unary { op, expr, .. } => {
            emit_expr(&expr.node, body, locals, params, fn_indices);
            if matches!(op, UnaryOp::Neg) {
                body.instruction(&Instruction::I32Const(0));
                body.instruction(&Instruction::I32Sub);
            }
        }
        Expr::Call { callee, args, .. } => {
            for arg in args {
                emit_expr(&arg.node, body, locals, params, fn_indices);
            }
            if let Expr::Ident(name) = &callee.node {
                let idx = fn_indices.get(&name.node).copied().unwrap_or(0);
                body.instruction(&Instruction::Call(idx));
            } else {
                body.instruction(&Instruction::I32Const(0));
            }
        }
        _ => {
            body.instruction(&Instruction::I32Const(0));
        }
    }
}
