//! Typed HIR lowering (Phase 1 passes through AST).

use nasaq_ast::Program;

#[derive(Debug, Clone)]
pub struct HirModule {
    pub program: Program,
}

pub fn lower(program: Program) -> HirModule {
    HirModule { program }
}
