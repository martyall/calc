use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Copy, Clone, Eq, Hash)]
pub struct SourePos {
    pub line: u32,
    pub column: u32,
}

impl Display for SourePos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(line {}, column {})", self.line, self.column)
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Copy, Clone, Eq, Hash)]
pub struct Span {
    pub start: SourePos,
    pub end: SourePos,
}

impl Default for Span {
    fn default() -> Self {
        Span {
            start: SourePos { line: 0, column: 0 },
            end: SourePos { line: 0, column: 0 },
        }
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.start.line == self.end.line {
            write!(
                f,
                "line {}, columns {}-{}",
                self.start.line, self.start.column, self.end.column
            )
        } else {
            write!(f, "{}-{}", self.start, self.end)
        }
    }
}

pub trait HasSourceLoc {
    fn source_loc(&self) -> Span;
}

impl HasSourceLoc for () {
    fn source_loc(&self) -> Span {
        Span::default()
    }
}

impl HasSourceLoc for Span {
    fn source_loc(&self) -> Span {
        *self
    }
}

// useful for tests but otherwise meaningless
pub fn from_pest_span(span: pest::Span) -> Span {
    Span {
        start: SourePos {
            line: span.start_pos().line_col().0 as u32,
            column: span.start_pos().line_col().1 as u32,
        },
        end: SourePos {
            line: span.end_pos().line_col().0 as u32,
            column: span.end_pos().line_col().1 as u32,
        },
    }
}
