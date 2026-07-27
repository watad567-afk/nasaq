use smol_str::SmolStr;

/// A single Nasaq source file loaded into memory.
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: SmolStr,
    pub contents: SmolStr,
}

impl SourceFile {
    pub fn new(path: impl Into<SmolStr>, contents: impl Into<SmolStr>) -> Self {
        Self {
            path: path.into(),
            contents: contents.into(),
        }
    }

    pub fn snippet(&self, start: u32, end: u32) -> &str {
        &self.contents[start as usize..end as usize]
    }

    pub fn line_col(&self, offset: u32) -> (u32, u32) {
        let mut line = 1u32;
        let mut col = 1u32;
        for (i, ch) in self.contents.char_indices() {
            if i as u32 >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }
}
