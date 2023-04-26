#![feature(anonymous_lifetime_in_impl_trait)]
#![feature(assert_matches)]
#![feature(is_some_and)]
#![feature(const_trait_impl)]

mod tree;
pub use tree::*;

pub mod patterns;
pub use patterns::Pattern;

mod rule;
pub use rule::*;

pub mod parsers;

pub mod context;
pub use context::Context;

pub mod errors;

#[cfg(test)]
mod test {
    use std::any::Any;

    use nom::Parser;

    use crate::{
        parsers::{alternatives, basic_pattern, regex, repeat, rule, rule_name},
        patterns::Repeat,
        ParseTree, Pattern, Rule,
    };

    #[test]
    fn test_rule_name() {
        assert_eq!(rule_name("ValidRuleName"), Ok(("", "ValidRuleName")));
        assert!(rule_name("invalidRuleName").is_err());
    }

    #[test]
    fn test_group() {
        assert_eq!(
            basic_pattern("(Rule)"),
            Ok(("", ("(Rule)", Pattern::RuleReference("Rule".to_string()))))
        );
        assert_eq!(
            basic_pattern("(Rule | [a-z])"),
            Ok((
                "",
                (
                    "(Rule | [a-z])",
                    Pattern::Alternatives(vec![
                        Pattern::RuleReference("Rule".to_string()),
                        Pattern::Regex("[a-z]".to_string())
                    ])
                )
            ))
        );
        assert_eq!(
            basic_pattern("(Rule [a-z])"),
            Ok((
                "",
                (
                    "(Rule [a-z])",
                    Pattern::Group(vec![
                        Pattern::RuleReference("Rule".to_string()),
                        Pattern::Regex("[a-z]".to_string())
                    ])
                )
            ))
        );
    }

    #[test]
    fn test_regex() {
        assert_eq!(regex("ValidRegex"), Ok(("", "ValidRegex")));
        assert_eq!(
            regex("Vali1324dRegex rest"),
            Ok((" rest", "Vali1324dRegex"))
        );

        assert_eq!(regex("x+"), Ok(("", "x+")));
        assert_eq!(regex("x*"), Ok(("", "x*")));
        assert_eq!(regex("x?"), Ok(("", "x?")));
    }

    #[test]
    fn test_basic_pattern() {
        assert_eq!(
            basic_pattern("ValidRuleName"),
            Ok((
                "",
                (
                    "ValidRuleName",
                    Pattern::RuleReference("ValidRuleName".to_string())
                )
            ))
        );
        assert_eq!(
            basic_pattern("validRegex"),
            Ok(("", ("validRegex", Pattern::Regex("validRegex".to_string()))))
        );
        assert_eq!(
            basic_pattern("(x y)"),
            Ok((
                "",
                (
                    "(x y)",
                    Pattern::Group(vec![
                        Pattern::Regex("x".to_string()),
                        Pattern::Regex("y".to_string())
                    ])
                )
            ))
        );
    }

    #[test]
    fn test_repeat() {
        let p = Pattern::Regex("x".to_string());
        assert_eq!(
            repeat("(x)+"),
            Ok(("", ("(x)+", Repeat::once_or_more(p.clone()))))
        );
        assert_eq!(
            repeat("(x)*"),
            Ok(("", ("(x)*", Repeat::zero_or_more(p.clone()))))
        );
        assert_eq!(
            repeat("(x)?"),
            Ok(("", ("(x)?", Repeat::at_most_once(p.clone()))))
        )
    }

    #[test]
    fn test_alternatives() {
        assert_eq!(
            alternatives("ValidRuleName | [a-z]"),
            Ok((
                "",
                (
                    "ValidRuleName | [a-z]",
                    Pattern::Alternatives(vec![
                        Pattern::RuleReference("ValidRuleName".to_string()),
                        Pattern::Regex("[a-z]".to_string()),
                    ])
                )
            ))
        );

        assert_eq!(
            alternatives("ValidRuleName| [a-z]"),
            Ok((
                "",
                (
                    "ValidRuleName| [a-z]",
                    Pattern::Alternatives(vec![
                        Pattern::RuleReference("ValidRuleName".to_string()),
                        Pattern::Regex("[a-z]".to_string()),
                    ])
                )
            ))
        );

        assert_eq!(
            alternatives("ValidRuleName |[a-z]"),
            Ok((
                "",
                (
                    "ValidRuleName |[a-z]",
                    Pattern::Alternatives(vec![
                        Pattern::RuleReference("ValidRuleName".to_string()),
                        Pattern::Regex("[a-z]".to_string()),
                    ])
                )
            ))
        );

        assert_eq!(
            alternatives("ValidRuleName|[a-z]"),
            Ok((
                "",
                (
                    "ValidRuleName|[a-z]",
                    Pattern::Alternatives(vec![
                        Pattern::RuleReference("ValidRuleName".to_string()),
                        Pattern::Regex("[a-z]".to_string()),
                    ])
                )
            ))
        );
    }

    #[test]
    fn test_rule() {
        assert_eq!(
            rule("Rule: x"),
            Ok((
                "",
                (
                    ParseTree::from(vec!["Rule", ":", "x"]),
                    Rule {
                        name: "Rule".to_string(),
                        patterns: vec![Pattern::Regex("x".to_string())]
                    }
                )
            ))
        )
    }

    #[test]
    fn test_pattern_as_parser() {
        let res = Pattern::Regex("x+".to_string()).parse("xxx");
        assert!(res.is_ok());
        let (rest, (tree, ast)) = res.unwrap();
        assert_eq!(rest, "");
        assert_eq!(tree, ParseTree::from("xxx"));
        assert_eq!(ast.downcast::<String>().ok().unwrap().as_str(), "xxx");

        let res = Pattern::Alternatives(vec![
            Pattern::Regex("x".to_string()),
            Pattern::Regex("y".to_string()),
        ])
        .parse("y");
        assert!(res.is_ok());
        let (rest, (tree, ast)) = res.unwrap();
        assert_eq!(rest, "");
        assert_eq!(tree, ParseTree::from("y"));
        assert_eq!(ast.downcast::<String>().ok().unwrap().as_str(), "y");

        let res = Pattern::Group(vec![
            Pattern::Regex("x".to_string()),
            Pattern::Regex("y".to_string()),
        ])
        .parse("xy");
        assert!(res.is_ok());
        let (rest, (tree, ast)) = res.unwrap();
        assert_eq!(rest, "");
        assert_eq!(tree, ParseTree::from(vec!["x", "y"]));
        assert_eq!(
            ast.downcast::<Vec<Box<dyn Any>>>()
                .ok()
                .unwrap()
                .into_iter()
                .map(|x| x.downcast::<String>().ok().unwrap())
                .collect::<Vec<_>>()
                .as_slice(),
            &vec![Box::new("x".to_string()), Box::new("y".to_string())]
        );
    }

    #[test]
    fn test_repeat_as_parser() {
        let res = Repeat::at_most_once(Pattern::Regex("x".to_string())).parse("");
        assert!(res.is_ok());
        let (rest, (tree, ast)) = res.unwrap();
        assert_eq!(rest, "");
        assert_eq!(tree, ParseTree::Tree(vec![]));
        assert!(ast.is_empty());
    }

    #[test]
    fn test_rule_as_parser() {
        let res = Rule {
            name: "Rule".to_string(),
            patterns: vec![Pattern::Regex("x".to_string())],
        }
        .parse("x");
        assert!(res.is_ok());
        let (rest, (tree, ast)) = res.unwrap();
        assert_eq!(rest, "");
        assert_eq!(tree, ParseTree::from(vec!["x"]));
        assert_eq!(
            ast.downcast::<Vec<Box<dyn Any>>>()
                .ok()
                .unwrap()
                .into_iter()
                .map(|x| x.downcast::<String>().ok().unwrap())
                .collect::<Vec<_>>()
                .as_slice(),
            &vec![Box::new("x".to_string())]
        );
    }
}
