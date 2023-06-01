#[cfg(test)]
use crate::{parsers::Parser, Context};
#[cfg(test)]
use pretty_assertions::assert_eq;
#[cfg(test)]
use serde_json::json;

use crate::{alts, patterns::transparent, rule_ref, Rule};

macro_rules! rule {
    ($name:ident, $pattern:expr) => {
        pub struct $name;

        impl $name {
            /// Get rule with this name
            pub fn rule() -> Rule {
                Rule::new(stringify!($name), $pattern)
            }
        }
    };
}

rule!(Char, transparent(r"/'.'/"));
#[test]
fn char() {
    let mut context = Context::default();
    let r = Char::rule();
    assert_eq!(r.parse("'c'", &mut context).ast, json!('c'));
}

rule!(Integer, transparent(r"/[0-9]+/"));
#[test]
fn integer() {
    let mut context = Context::default();
    let r = Integer::rule();
    assert_eq!(r.parse("123", &mut context).ast, json!(123));

    let big_integer = "99999999999999999999999999999999";
    assert_eq!(
        r.parse(big_integer, &mut context).ast,
        json!({ "Integer": big_integer })
    );
}

rule!(String, transparent("/\"([^\"\\\\]|\\.)*\"/"));
#[test]
fn string() {
    let mut context = Context::default();
    let r = String::rule();
    assert_eq!(r.parse("\"str\"", &mut context).ast, json!("str"));
}

rule!(
    Text,
    transparent(alts!(
        rule_ref!(Char),
        rule_ref!(String),
        r"/[^\s*+?()|<:>{}=]+/"
    ))
);
#[test]
fn text() {
    let mut context = Context::default();
    let r = Text::rule();
    assert_eq!(r.parse("'c'", &mut context).ast, json!('c'));
    assert_eq!(r.parse("\"str\"", &mut context).ast, json!("str"));
    assert_eq!(r.parse("text", &mut context).ast, json!("text"));
}

rule!(Regex, transparent(r"//[^/]+//"));
#[test]
fn regex() {
    let mut context = Context::default();
    let r = Regex::rule();
    assert_eq!(r.parse("/ax?/", &mut context).ast, json!("/ax?/"));
}

rule!(RuleName, transparent(r"/[A-Z][a-zA-Z0-9]*/"));
#[test]
fn rule_name() {
    let mut context = Context::default();
    let r = RuleName::rule();
    assert_eq!(r.parse("Rule", &mut context).ast, json!("Rule"));
}

rule!(RuleReference, rule_ref!(RuleName));
#[test]
fn rule_reference() {
    let mut context = Context::default();
    let r = RuleReference::rule();
    assert_eq!(
        r.parse("RuleName", &mut context).ast,
        json!({"RuleReference": "RuleName"})
    );
}

rule!(Identifier, transparent(r"/[a-zA-Z_][a-zA-Z0-9_]*/"));
#[test]
fn identifier() {
    let mut context = Context::default();
    let r = Identifier::rule();
    assert_eq!(r.parse("name", &mut context).ast, json!("name"));
    assert_eq!(r.parse("Name", &mut context).ast, json!("Name"));
}

rule!(Typename, transparent("/[A-Z][a-zA-Z_0-9]*/"));
#[test]
fn typename() {
    let mut context = Context::default();
    let r = Typename::rule();
    assert_eq!(r.parse("Type", &mut context).ast, json!("Type"));
}

// rule!(
//     Type,
//     transparent(alts!(rule_ref!(Typename), rule_ref!(Variable)))
// );
