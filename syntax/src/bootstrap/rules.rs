#[cfg(test)]
use crate::{parsers::Parser, Context};
#[cfg(test)]
use pretty_assertions::assert_eq;
#[cfg(test)]
use serde_json::json;

use crate::{
    action::{merge, reference, ret},
    alts,
    patterns::{self, separated, transparent, Sequence},
    rule_ref, seq, Rule,
};

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

rule!(Identifier, transparent(r"/[a-z_][a-zA-Z0-9_]*/"));
#[test]
fn identifier() {
    let mut context = Context::default();
    let r = Identifier::rule();
    assert_eq!(r.parse("name", &mut context).ast, json!("name"));
}

rule!(Typename, transparent("/[A-Z][a-zA-Z_0-9]*/"));
#[test]
fn typename() {
    let mut context = Context::default();
    let r = Typename::rule();
    assert_eq!(r.parse("Type", &mut context).ast, json!("Type"));
}

rule!(Variable, rule_ref!(Identifier));
#[test]
fn variable() {
    let mut context = Context::default();
    let r = Variable::rule();
    assert_eq!(r.parse("var", &mut context).ast, json!({"Variable": "var"}));
}

rule!(
    Type,
    transparent(alts!(rule_ref!(Typename), rule_ref!(Variable)))
);
#[test]
fn ty() {
    let mut context = Context::default();
    let r = Type::rule();
    assert_eq!(r.parse("Type", &mut context).ast, json!("Type"));
    assert_eq!(r.parse("var", &mut context).ast, json!({"Variable": "var"}));
}

rule!(
    Value,
    transparent(alts!(
        rule_ref!("Distinct"),
        rule_ref!(Char),
        rule_ref!(String),
        rule_ref!(Integer),
        rule_ref!(Object)
    ))
);
#[test]
fn value() {
    let mut context = Context::default();
    let r = Value::rule();
    assert_eq!(r.parse("'c'", &mut context).ast, json!('c'));
    assert_eq!(r.parse("\"str\"", &mut context).ast, json!("str"));
    assert_eq!(r.parse("123", &mut context).ast, json!(123));
    assert_eq!(r.parse("{}", &mut context).ast, json!({}));
}

rule!(
    Object,
    transparent(alts!(
        vec!['{'.into(), '}'.into()],
        rule_ref!(NonEmptyObject)
    ))
);
#[test]
fn object() {
    let mut context = Context::default();
    let r = Object::rule();
    assert_eq!(r.parse("{}", &mut context).ast, json!({}));
    assert_eq!(
        r.parse("{a: 1, b: 2}", &mut context).ast,
        json!({"a": 1, "b": 2})
    );
}

rule!(
    NonEmptyObject,
    Sequence::new(
        vec![
            '{'.into(),
            ("initializers", separated(rule_ref!(Initializer), ',')).into(),
            patterns::Repeat::at_most_once(',').into(),
            '}'.into(),
        ],
        ret(merge(reference("initializers")))
    )
);
#[test]
fn non_empty_object() {
    let mut context = Context::default();
    let r = NonEmptyObject::rule();
    assert_eq!(r.parse("{a: 1}", &mut context).ast, json!({"a": 1}));
    assert_eq!(r.parse("{a: 1,}", &mut context).ast, json!({"a": 1}));
    assert_eq!(
        r.parse("{a: 1, b: 2}", &mut context).ast,
        json!({"a": 1, "b": 2})
    );
    assert_eq!(
        r.parse("{a: 1, b: 2,}", &mut context).ast,
        json!({"a": 1, "b": 2})
    );
}

rule!(
    Initializer,
    seq!(
        ("name", rule_ref!(Identifier)),
        ':',
        ("value", rule_ref!("Expression"))
        =>
        ret(reference("value").cast_to(reference("name")))
    )
);
#[test]
fn initializer() {
    let mut context = Context::default();
    let r = Initializer::rule();
    assert_eq!(r.parse("a: 1", &mut context).ast, json!({"a": 1}));
}

rule!(
    Cast,
    seq!(
        ("expr", alts!(rule_ref!(Variable), rule_ref!(Value))),
        "as",
        ("ty", rule_ref!(Type))
    )
);
#[test]
fn cast_() {
    let mut context = Context::default();
    let r = Cast::rule();
    assert_eq!(
        r.parse("1 as Integer", &mut context).ast,
        json!({
            "Cast": {
                "ty": "Integer",
                "expr": 1
            }
        })
    );
    assert_eq!(
        r.parse("1 as ty", &mut context).ast,
        json!({
            "Cast": {
                "ty": { "Variable": "ty" },
                "expr": 1
            }
        })
    );
}

rule!(
    Expression,
    transparent(alts!(
        rule_ref!(Cast),
        rule_ref!(Value),
        rule_ref!(Variable)
    ))
);
#[test]
fn expression() {
    let mut context = Context::default();
    let r = Expression::rule();
    assert_eq!(r.parse("1", &mut context).ast, json!(1));
    assert_eq!(r.parse("var", &mut context).ast, json!({"Variable": "var"}));
    assert_eq!(
        r.parse("1 as Integer", &mut context).ast,
        json!({
            "Cast": {
                "ty": "Integer",
                "expr": 1
            }
        })
    );
}

rule!(
    Return,
    seq!(
        ("value", rule_ref!(Expression))
        =>
        ret(reference("value").cast_to("Return"))
    )
);
#[test]
fn return_() {
    let mut context = Context::default();
    let r = Return::rule();
    assert_eq!(
        r.parse("1", &mut context).ast,
        json!({
            "Return": 1
        })
    );
}

rule!(
    Throw,
    seq!(
        "throw",
        ("error", rule_ref!(Expression)) =>
        ret(reference("error").cast_to("Throw"))
    )
);
#[test]
fn throw() {
    let mut context = Context::default();
    let r = Throw::rule();
    assert_eq!(
        r.parse("throw 1", &mut context).ast,
        json!({
            "Throw": 1
        })
    );
}

rule!(
    Named,
    seq!(
        "<",
        ("name", rule_ref!(Identifier)),
        ":",
        ("pattern", rule_ref!("Pattern")),
        ">"
    )
);
#[test]
fn named() {
    let mut context = Context::default();
    let r = Named::rule();
    assert_eq!(
        r.parse("<name: x>", &mut context).ast,
        json!({
            "Named": {
                "name": "name",
                "pattern": "x"
            }
        })
    );
}

rule!(
    AtomicPattern,
    transparent(alts!(
        seq!(
            "(",
            ("pattern", rule_ref!("Pattern")),
            ")"
            => ret(reference("pattern"))
        ),
        rule_ref!(Named),
        rule_ref!(RuleReference),
        rule_ref!(Regex),
        rule_ref!(Text)
    ))
);
#[test]
fn atomic_pattern() {
    let mut context = Context::default();
    let r = AtomicPattern::rule();
    assert_eq!(r.parse("(x)", &mut context).ast, json!("x"));
    assert_eq!(
        r.parse("<name: x>", &mut context).ast,
        json!({
            "Named": {
                "name": "name",
                "pattern": "x"
            }
        })
    );
    assert_eq!(
        r.parse("RuleName", &mut context).ast,
        json!({
            "RuleReference": "RuleName"
        })
    );
    assert_eq!(r.parse("/x/", &mut context).ast, json!("/x/"));
    assert_eq!(r.parse("x", &mut context).ast, json!("x"));
}

rule!(
    Alternatives,
    transparent(separated(("x", rule_ref!(Sequence)), "|"))
);
#[test]
fn alternatives() {
    let mut context = Context::default();
    let r = Alternatives::rule();
    assert_eq!(r.parse("x", &mut context).ast, json!("x"));
    assert_eq!(
        r.parse("x | y", &mut context).ast,
        json!({"Alternatives": ["x", "y"]})
    );
    assert_eq!(
        r.parse("x y | z | f g", &mut context).ast,
        json!({"Alternatives": [["x", "y"], "z", ["f", "g"]]})
    );
}

rule!(
    DistinctObject,
    transparent(seq!(
        ("ty", rule_ref!(Typename)),
        ("obj", rule_ref!(Object))
        =>
        ret(reference("obj").cast_to(reference("ty")))
    ))
);
#[test]
fn distinct_object() {
    let mut context = Context::default();
    let r = DistinctObject::rule();
    assert_eq!(
        r.parse("Type {}", &mut context).ast,
        json!({
            "Type": {}
        })
    );
}

rule!(
    DistinctValue,
    transparent(seq!(
        ("ty", rule_ref!(Typename)),
        '(',
        ("value", rule_ref!(Value)),
        ')'
        =>
        ret(reference("value").cast_to(reference("ty")))
    ))
);
#[test]
fn distinct_value() {
    let mut context = Context::default();
    let r = DistinctValue::rule();
    assert_eq!(
        r.parse("Type(1)", &mut context).ast,
        json!({
            "Type": 1
        })
    );
    assert_eq!(
        r.parse("Type('c')", &mut context).ast,
        json!({
            "Type": 'c'
        })
    );
    assert_eq!(
        r.parse("Type(\"str\")", &mut context).ast,
        json!({
            "Type": "str"
        })
    );
    assert_eq!(
        r.parse("Type({})", &mut context).ast,
        json!({
            "Type": {}
        })
    );
    assert_eq!(
        r.parse("Type({a: 1})", &mut context).ast,
        json!({
            "Type": {"a": 1}
        })
    );
    assert_eq!(
        r.parse("Foo( Bar { a: 1 } )", &mut context).ast,
        json!({
            "Foo": {
                "Bar": {
                    "a": 1
                }
            }
        })
    );
}

rule!(
    Distinct,
    transparent(alts!(rule_ref!(DistinctObject), rule_ref!(DistinctValue)))
);
#[test]
fn distinct() {
    let mut context = Context::default();
    let r = Distinct::rule();
    assert_eq!(
        r.parse("Type {}", &mut context).ast,
        json!({
            "Type": {}
        })
    );
    assert_eq!(
        r.parse("Type(1)", &mut context).ast,
        json!({
            "Type": 1
        })
    );
}
