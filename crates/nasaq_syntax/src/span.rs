use std::ops::Range;

/// Byte offset span into a [`SourceFile`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub const EMPTY: Self = Self { start: 0, end: 0 };

    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub fn merge(self, other: Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    pub fn range(self) -> Range<usize> {
        self.start as usize..self.end as usize
    }

    pub fn slice<'a>(self, source: &'a str) -> &'a str {
        let len = source.len();
        let mut start = self.start as usize;
        let mut end = self.end as usize;
        start = start.min(len);
        end = end.min(len);
        while start > 0 && !source.is_char_boundary(start) {
            start -= 1;
        }
        while end < len && !source.is_char_boundary(end) {
            end += 1;
        }
        if start > end {
            return "";
        }
        &source[start..end]
    }

    pub fn contains(self, offset: u32) -> bool {
        offset >= self.start && offset < self.end
    }
}

/// A value annotated with its source span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Spanned<U> {
        Spanned {
            node: f(self.node),
            span: self.span,
        }
    }
}
