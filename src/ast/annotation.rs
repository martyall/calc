use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Copy, Clone, Eq, Hash)]
pub struct SourePos {
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Copy, Clone, Eq, Hash)]
pub struct Span {
    pub start: SourePos,
    pub end: SourePos,
}

// useful for tests but otherwise meaningless
impl Default for Span {
    fn default() -> Self {
        Span {
            start: SourePos { line: 0, column: 0 },
            end: SourePos { line: 0, column: 0 },
        }
    }
}

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
