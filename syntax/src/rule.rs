use crate::Pattern;

/// Type of rule name
pub type RuleName = String;

/// Ast for rules
#[derive(Debug, PartialEq, Clone)]
pub struct Rule {
    /// Rule name
    pub name: RuleName,
    /// Rule patterns
    pub patterns: Vec<Pattern>,
}
