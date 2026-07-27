//! Language Server Protocol support for Nasaq.

pub mod analyze;
pub mod completion;
pub mod docs;
pub mod server;

pub use analyze::{analyze_file, analyze_source, AnalysisResult, VERSION};
pub use completion::completion_items;
pub use server::run_stdio;
