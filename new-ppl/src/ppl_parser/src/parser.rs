use pest::{iterators::Pair, Parser};
use ppl_ast::{
    annotation::Annotation,
    declarations::{
        function::{Function, FunctionId, Parameter, Text},
        ty::Type,
        Declaration,
    },
    expressions::{literal::Literal, Expression},
    identifier::Identifier,
    module::Module,
    statements::{AnnotatedStatement, Statement},
    typename::Typename,
};

use crate::{
    diagnostic::{Diagnostic, Diagnostics},
    source::SourceProgram,
    Db,
};

#[derive(pest_derive::Parser)]
#[grammar = "ppl.pest"]
struct PPLParser;

#[salsa::tracked]
pub fn module(db: &dyn Db, source: SourceProgram) -> Module {
    let source_text = source.text(db);

    let module = PPLParser::parse(Rule::module, &source_text);
    if let Err(err) = module {
        let err = if let Some(path) = source.path(db) {
            err.with_path(path.to_str().unwrap())
        } else {
            err
        };
        Diagnostics::push(
            db,
            Diagnostic {
                message: err.to_string(),
            },
        );
        return Module::new(db, vec![]);
    }
    let module = module.unwrap().next().unwrap();

    let mut stmts: Vec<AnnotatedStatement> = Vec::new();
    for stmt in module.into_inner() {
        match stmt.as_rule() {
            Rule::annotated_statement => {
                stmts.push(annotated_statement(db, stmt).into());
            }
            _ => {
                break;
            }
        }
    }

    Module::new(db, stmts)
}

fn annotated_statement(db: &dyn Db, tree: Pair<'_, Rule>) -> AnnotatedStatement {
    let mut annotations = Vec::new();
    let mut stmt = None;
    for tree in tree.into_inner() {
        match tree.as_rule() {
            Rule::annotation => {
                annotations.push(annotation(db, tree));
            }
            _ => {
                stmt = Some(statement(db, tree));
                break;
            }
        }
    }
    AnnotatedStatement {
        annotations,
        statement: stmt.unwrap(),
    }
}

fn annotation(db: &dyn Db, tree: Pair<'_, Rule>) -> Annotation {
    Annotation::new(db, identifier(db, tree.into_inner().next().unwrap()))
}

fn statement(db: &dyn Db, tree: Pair<'_, Rule>) -> Statement {
    match tree.as_rule() {
        Rule::expression_statement => expression(db, tree).into(),
        _ => declaration(db, tree).into(),
    }
}

fn expression(db: &dyn Db, tree: Pair<'_, Rule>) -> Expression {
    literal(db, tree).into()
}

fn literal(_db: &dyn Db, tree: Pair<'_, Rule>) -> Literal {
    let tree = tree.into_inner().next().unwrap();
    match tree.as_rule() {
        Rule::NONE => Literal::None,
        Rule::TRUE => Literal::Boolean(true),
        Rule::FALSE => Literal::Boolean(false),
        _ => {
            unreachable!("Unexpected rule {:?}", tree.as_rule())
        }
    }
}

fn declaration(db: &dyn Db, tree: Pair<'_, Rule>) -> Declaration {
    match tree.as_rule() {
        Rule::function => function(db, tree).into(),
        Rule::r#type => ty(db, tree).into(),
        _ => {
            unreachable!("Unexpected rule {:?}", tree.as_rule())
        }
    }
}

fn ty(db: &dyn Db, tree: Pair<'_, Rule>) -> Type {
    Type::new(db, typename(db, tree.into_inner().next().unwrap()))
}

fn typename(db: &dyn Db, tree: Pair<'_, Rule>) -> Typename {
    Typename::new(db, tree.as_str().to_string())
}

fn function(db: &dyn Db, tree: Pair<'_, Rule>) -> Function {
    let name_parts: Vec<_> = tree
        .into_inner()
        .filter_map(|part| {
            Some(match part.as_rule() {
                Rule::text => Text::new(db, part.as_str().to_string()).into(),
                Rule::parameter => parameter(db, part).into(),
                _ => return None,
            })
        })
        .collect();
    Function::new(db, FunctionId::from_parts(db, &name_parts), name_parts)
}

fn parameter(db: &dyn Db, tree: Pair<'_, Rule>) -> Parameter {
    let mut parts = tree.into_inner();
    Parameter {
        name: identifier(db, parts.next().unwrap()),
        ty: typename(db, parts.next().unwrap()),
    }
}

fn identifier(db: &dyn Db, tree: Pair<'_, Rule>) -> Identifier {
    Identifier::new(db, tree.as_str().to_string())
}
