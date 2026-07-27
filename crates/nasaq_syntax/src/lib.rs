//! Shared syntax primitives: spans, source files, and future CST nodes.

mod span;
mod source;
mod extensions;

pub use span::{Span, Spanned};
pub use source::SourceFile;
pub use extensions::{
    with_output_ext, with_runtime_ext, with_source_ext, resolve_source_path, OUTPUT, RUNTIME,
    SOURCE, SOURCE_LEGACY,
};
