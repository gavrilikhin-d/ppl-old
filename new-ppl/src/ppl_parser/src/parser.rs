use pest::Parser;
use ppl_ast::{
    declarations::{Function, FunctionId},
    module::Module,
};

use crate::{source::SourceProgram, Db};

#[derive(pest_derive::Parser)]
#[grammar = "ppl.pest"]
pub struct PPLParser;

#[salsa::tracked]
pub fn parse_module(db: &dyn Db, source: SourceProgram) -> Module {
    let source_text = source.text(db);

    let module = PPLParser::parse(Rule::module, &source_text)
        .unwrap()
        .next()
        .unwrap();

    let mut functions: Vec<Function> = Vec::new();
    for f in module.into_inner() {
        match f.as_rule() {
            Rule::function => functions.push(Function::new(
                db,
                FunctionId::new(db, f.into_inner().nth(1).unwrap().as_str().to_string()),
            )),
            _ => {}
        }
    }

    Module::new(db, functions)
}
