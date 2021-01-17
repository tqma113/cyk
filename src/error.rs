use super::tree::Span;

use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Diagnostic {
    c: char,
    span: Span,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.c, f)
    }
}

impl Diagnostic {
    pub fn new(c: char, span: Span) -> Self {
        Diagnostic { c, span }
    }
}
