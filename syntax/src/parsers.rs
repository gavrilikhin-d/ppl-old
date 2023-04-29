use std::{any::Any, error::Error};

use nom::{IResult, Parser};

use crate::{ParseTree, Pattern};

/// Parse multiple patterns as group
pub fn grouped_patterns<'i, 'p>(
    patterns: &'p mut [Pattern],
    input: &'i str,
) -> IResult<&'i str, (ParseTree<'i>, Vec<Box<dyn Any>>), Box<dyn Error>> {
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
