use std::{any::Any, error::Error};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{alpha0, one_of, satisfy, space0},
    combinator::map,
    error::VerboseError,
    multi::{many1, separated_list1},
    sequence::delimited,
    Parser,
};

use crate::{context, patterns::Repeat, ParseTree, Pattern, Rule};

type IResult<I, O> = nom::IResult<I, O, VerboseError<I>>;

/// [A-Z]
pub fn uppercase_alpha(input: &str) -> IResult<&str, char> {
    satisfy(|c| c.is_ascii_uppercase())(input)
}

/// RuleName: [A-Z][a-zA-Z]*
pub fn rule_name(input: &str) -> IResult<&str, &str> {
    let (new_input, _) = uppercase_alpha(input)?;
    let (new_input, tail) = alpha0(new_input)?;
    Ok((new_input, &input[..1 + tail.len()]))
}

/// Parse multiple patterns as group
pub(crate) fn grouped_patterns<'i, 'p>(
    patterns: &'p mut [Pattern],
    input: &'i str,
) -> nom::IResult<&'i str, (ParseTree<'i>, Vec<Box<dyn Any>>), Box<dyn Error>> {
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

/// Rule: RuleName: Pattern+
pub fn rule(input: &str) -> IResult<&str, (ParseTree, Rule)> {
    let (rest, name) = rule_name(input)?;
    let mut tree: ParseTree = name.into();

    let (rest, colon) = delimited(space0, tag(":"), space0)(rest)?;
    tree.append(colon);

    let (rest, v) = many1(delimited(space0, pattern, space0))(rest)?;
    v.iter().for_each(|(s, _)| {
        tree.append(*s);
    });

    let rule = Rule {
        name: name.to_string(),
        patterns: v.into_iter().map(|(_, p)| p).collect(),
    };
    context::add_rule(rule.clone());

    Ok((rest, (tree, rule)))
}

/// Pattern: Repeat | Alternatives
pub fn pattern(input: &str) -> IResult<&str, (&str, Pattern)> {
    alt((map(repeat, |(s, r)| (s, Pattern::from(r))), alternatives))(input)
}

/// Repeat: BasicPattern ('+' | '*' | '?')
pub fn repeat(input: &str) -> IResult<&str, (&str, Repeat)> {
    let (rest, (_, p)) = basic_pattern(input)?;
    let (rest, c) = one_of("+*?")(rest)?;
    Ok((
        rest,
        (
            &input[..input.len() - rest.len()],
            match c {
                '*' => Repeat::zero_or_more(p),
                '+' => Repeat::once_or_more(p),
                '?' => Repeat::at_most_once(p),
                _ => unreachable!(),
            },
        ),
    ))
}

/// Alternatives: BasicPattern ( '|' BasicPattern )*
pub fn alternatives(input: &str) -> IResult<&str, (&str, Pattern)> {
    let (rest, v) = separated_list1(delimited(space0, tag("|"), space0), basic_pattern)(input)?;
    Ok((
        rest,
        if v.len() == 1 {
            v.into_iter().next().unwrap()
        } else {
            (
                &input[..input.len() - rest.len()],
                Pattern::Alternatives(v.into_iter().map(|(_, p)| p).collect()),
            )
        },
    ))
}

/// BasicPattern: RuleReference | Group | Regex
pub fn basic_pattern(input: &str) -> IResult<&str, (&str, Pattern)> {
    alt((
        map(rule_reference, |s| {
            (s, Pattern::RuleReference(s.to_string()))
        }),
        map(group, |(s, v)| {
            (
                s,
                if v.len() == 1 {
                    v.into_iter().next().unwrap()
                } else {
                    Pattern::Group(v)
                },
            )
        }),
        map(regex, |s| (s, Pattern::Regex(s.to_string()))),
    ))(input)
}

/// RuleReference: RuleName
pub fn rule_reference(input: &str) -> IResult<&str, &str> {
    rule_name(input)
}

/// Group: '(' Pattern+ ')'
pub fn group(input: &str) -> IResult<&str, (&str, Vec<Pattern>)> {
    let (rest, v) = delimited(
        tag("("),
        many1(delimited(space0, pattern, space0)),
        tag(")"),
    )(input)?;
    Ok((
        rest,
        (
            &input[..input.len() - rest.len()],
            v.into_iter().map(|(_, p)| p).collect(),
        ),
    ))
}

/// Regex: [^ \t\r\n()|]+
pub fn regex(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| !(c.is_whitespace() || ['(', ')', '|'].contains(&c)))(input)
}
