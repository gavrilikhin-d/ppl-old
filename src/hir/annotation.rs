/// HIR for annotations
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Annotation {
    /// Set mangled name of function
    MangleAs(String),
}
