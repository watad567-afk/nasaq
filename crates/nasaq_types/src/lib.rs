//! Type checker for Nasaq (Phase 1 subset).

mod checker;
mod types;

pub use checker::{TypeCheckResult, typecheck};
pub use types::Ty;
