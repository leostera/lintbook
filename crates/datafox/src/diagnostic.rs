use serde::{Deserialize, Serialize};

/// One source span in an input query string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// One contextual parser or analyzer diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Diagnostic {
    pub message: String,
    pub span: Option<Span>,
    pub found: Option<String>,
}

impl Diagnostic {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
            found: None,
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_found(mut self, found: impl Into<String>) -> Self {
        self.found = Some(found.into());
        self
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(span) = self.span {
            write!(f, " at {}..{}", span.start, span.end)?;
        }
        if let Some(found) = &self.found {
            write!(f, " (found `{found}`)")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Diagnostic, Span};

    #[test]
    fn diagnostic_formats_message_span_and_found_context() {
        let diagnostic = Diagnostic::new("expected `)`")
            .with_span(Span::new(12, 13))
            .with_found(",");

        assert_eq!(diagnostic.to_string(), "expected `)` at 12..13 (found `,`)");
    }
}
