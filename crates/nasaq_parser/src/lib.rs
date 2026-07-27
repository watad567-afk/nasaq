//! Parser for Nasaq source files.

mod component;
mod parser;
mod recovery;
mod view;

pub use parser::{ParseResult, Parser, parse_program};
