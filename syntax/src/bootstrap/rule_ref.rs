#[macro_export]
macro_rules! rule_ref {
    ($name: ident) => {
        crate::Pattern::RuleReference(crate::bootstrap::rules::$name::rule().name.clone())
    };
}
