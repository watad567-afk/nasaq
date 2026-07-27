//! Basic Nasaq formatter — normalizes indentation and spacing.

pub fn format_source(source: &str) -> String {
    let mut out = String::new();
    let mut indent = 0usize;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            out.push('\n');
            continue;
        }
        if trimmed.starts_with('}') {
            indent = indent.saturating_sub(1);
        }
        out.push_str(&"    ".repeat(indent));
        out.push_str(trimmed);
        out.push('\n');
        if trimmed.ends_with('{') {
            indent += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indents_blocks() {
        let formatted = format_source("fn main() {\nlet x = 1\n}\n");
        assert!(formatted.contains("    let x = 1"));
    }
}
