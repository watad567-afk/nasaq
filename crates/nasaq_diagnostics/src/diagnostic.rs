use nasaq_syntax::{SourceFile, Span};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: Option<String>,
    pub message: String,
    pub span: Span,
    pub labels: Vec<(Span, String)>,
    pub help: Option<String>,
    pub suggestion: Option<String>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: Severity::Error,
            code: None,
            message: message.into(),
            span,
            labels: Vec::new(),
            help: None,
            suggestion: None,
        }
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    pub fn render(&self, file: &SourceFile) -> String {
        let (line, col) = file.line_col(self.span.start);
        let mut out = format!(
            "{}[{code}] {message}\n  --> {file}:{line}:{col}\n",
            match self.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => "info",
                Severity::Hint => "hint",
            },
            code = self.code.as_deref().unwrap_or("nasaq"),
            message = self.message,
            file = file.path,
            line = line,
            col = col,
        );
        if let Some(help) = &self.help {
            out.push_str(&format!("   = help: {help}\n"));
        }
        if let Some(suggestion) = &self.suggestion {
            out.push_str(&format!("   = suggestion: {suggestion}\n"));
        }
        out
    }
}

#[derive(Debug, Default)]
pub struct DiagnosticBag {
    pub diagnostics: Vec<Diagnostic>,
}

impl DiagnosticBag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }

    pub fn merge(&mut self, other: DiagnosticBag) {
        self.diagnostics.extend(other.diagnostics);
    }
}

pub fn suggestion(message: impl Into<String>, span: Span) -> Diagnostic {
    Diagnostic {
        severity: Severity::Hint,
        code: Some("suggestion".into()),
        message: message.into(),
        span,
        labels: Vec::new(),
        help: None,
        suggestion: None,
    }
}

#[derive(Debug, Error)]
pub enum EmitError {
    #[error("compilation failed with {0} error(s)")]
    Failed(usize),
}
