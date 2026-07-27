//! Official Nasaq linter — Phase 1 rules.

pub struct LintIssue {
    pub line: u32,
    pub message: String,
}

pub fn lint_source(source: &str) -> Vec<LintIssue> {
    let mut issues = Vec::new();
    for (i, line) in source.lines().enumerate() {
        let line_no = (i + 1) as u32;
        let trimmed = line.trim();
        if trimmed.contains("unsafeHtml") {
            issues.push(LintIssue {
                line: line_no,
                message: "avoid unsafeHtml unless absolutely required".into(),
            });
        }
        if trimmed.starts_with("let ") && trimmed.contains("= null") {
            issues.push(LintIssue {
                line: line_no,
                message: "prefer Option<T> over null — use None instead".into(),
            });
        }
    }
    issues
}
