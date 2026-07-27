//! JavaScript ESM code generator.

mod emit;

pub use emit::{CodegenOptions, GeneratedModule, emit_module};
