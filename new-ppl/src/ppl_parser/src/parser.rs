use pest::Parser;
use ppl_ast::{
    declarations::{Function, FunctionId, Identifier, Parameter, Text, Typename},
    module::Module,
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
pub fn parse_module(db: &dyn Db, source: SourceProgram) -> Module {
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

    let mut functions: Vec<Function> = Vec::new();
    for f in module.into_inner() {
        match f.as_rule() {
            Rule::function => {
                let name_parts = f
                    .into_inner()
                    .map(|part| match part.as_rule() {
                        Rule::text => Text::new(db, part.as_str().to_string()).into(),
                        Rule::parameter => {
                            let parts = part.into_inner();
                            Parameter::new(
                                db,
                                Identifier::new(db, "lol".to_string()),
                                Typename::new(db, "kek".to_string()),
                            )
                            .into()
                        }
                        _ => {
                            todo!("Handle invalid parsing for rule {}", part.to_string())
                        }
                    })
                    .collect();
                functions.push(Function::new(
                    db,
                    FunctionId::new(db, "mock".to_string()),
                    name_parts,
                ))
            }
            _ => {}
        }
    }

    Module::new(db, functions)
}
