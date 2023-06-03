use derive_more::From;
use serde::{Deserialize, Serialize};

use crate::{
    parsers::{ParseResult, Parser},
    Rule,
};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, From)]
#[serde(untagged)]
pub enum RuleReference {
    Weak(String),
    Strong(Rule),
}

impl From<&str> for RuleReference {
    fn from(value: &str) -> Self {
        Self::Weak(value.to_string())
    }
}

#[macro_export]
macro_rules! rule_ref {
    ($rule: ident) => {
        crate::Pattern::RuleReference(Box::new($rule::rule().into()))
    };
    ($name: expr) => {
        crate::Pattern::RuleReference(Box::new($name.into()))
    };
}

impl Parser for RuleReference {
    fn parse_at<'s>(
        &self,
        source: &'s str,
        at: usize,
        context: &mut crate::Context,
    ) -> ParseResult<'s> {
        match self {
            Self::Weak(name) => {
                let rule = context.find_rule(name).expect("Rule not found");
                rule.parse_at(source, at, context)
            }
            Self::Strong(rule) => rule.parse_at(source, at, context),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bootstrap::rules::Text, Context, Pattern};
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn weak() {
        let mut context = Context::default();
        let r = rule_ref!("Text");
        assert_eq!(r, Pattern::RuleReference(Box::new("Text".into())));
        assert_eq!(r.parse("text", &mut context).ast, json!("text"));
    }

    #[test]
    fn strong() {
        let mut context = Context::default();
        let r = rule_ref!(Text);
        assert_eq!(
            r,
            Pattern::RuleReference(Box::new(RuleReference::Strong(Text::rule())))
        );
        assert_eq!(r.parse("text", &mut context).ast, json!("text"));
    }
}
