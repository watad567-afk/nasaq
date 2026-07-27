//! DOM compiler: lowers Nasaq components to fine-grained reactive JavaScript.

mod compile;

pub use compile::{CompiledComponent, compile_component, compile_program_components};
