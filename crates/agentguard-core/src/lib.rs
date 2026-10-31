//! Shared types used by AgentGuard's verification pipeline.

/// Describes how much evidence supports a diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Evidence {
    /// A property violation was established under stated assumptions.
    Proven,

    /// A test or execution reproduced the failure.
    Reproduced,

    /// Static analysis found a potentially defective path.
    StaticFinding,

    /// A pattern looks suspicious and needs human review.
    Heuristic,
}

/// A location in a source file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceLocation {
    pub path: String,
    pub line: u32,
    pub column: u32,
}

/// A finding produced by an analyzer or verification step.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub evidence: Evidence,
    pub location: Option<SourceLocation>,
}

impl Diagnostic {
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        evidence: Evidence,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            evidence,
            location: None,
        }
    }

    pub fn at(mut self, location: SourceLocation) -> Self {
        self.location = Some(location);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{Diagnostic, Evidence, SourceLocation};

    #[test]
    fn a_diagnostic_can_include_a_source_location() {
        let diagnostic = Diagnostic::new(
            "AG1001",
            "A possible defect was found.",
            Evidence::StaticFinding,
        )
        .at(SourceLocation {
            path: "src/main.rs".to_owned(),
            line: 12,
            column: 4,
        });

        assert_eq!(diagnostic.location.unwrap().line, 12);
    }
}
