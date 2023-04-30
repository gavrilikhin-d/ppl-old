use std::{any::Any, error::Error};

use nom::{IResult, Parser};

use crate::{ParseTree, Pattern, Rule};

/// Parse multiple patterns as group
pub fn grouped_patterns<'i, 'p>(
    patterns: &'p mut [Pattern],
    input: &'i str,
) -> IResult<&'i str, (ParseTree<'i>, Vec<Box<dyn Any>>), Box<dyn Error + 'i>> {
    let mut input = input;
    let mut trees = Vec::new();
    let mut asts = Vec::new();
    for p in patterns {
        let (rest, (t, ast)) = p.parse(input)?;
        input = rest;
        trees.push(t);
        asts.push(ast);
    }
    Ok((input, (trees.into(), asts)))
}

/// Create default parsing rules
pub fn create_default_rules() -> Vec<Rule> {
    vec![Rule {
        name: "Regex".to_string(),
        patterns: vec![Pattern::Regex(r"[^\s]+".to_string())],
    }]
}
