use pest::Parser;

#[salsa::jar(db = Db)]
pub struct Jar(
    SourceProgram,
    Module,
    Function,
    FunctionId,
    parse_module,
    Diagnostics,
);

pub trait Db: salsa::DbWithJar<Jar> {}
impl<DB> Db for DB where DB: ?Sized + salsa::DbWithJar<Jar> {}

#[derive(Default)]
#[salsa::db(Jar)]
pub struct Database {
    storage: salsa::Storage<Self>,
}
impl salsa::Database for Database {}
impl salsa::ParallelDatabase for Database {
    fn snapshot(&self) -> salsa::Snapshot<Self> {
        salsa::Snapshot::new(Database {
            storage: self.storage.snapshot(),
        })
    }
}

#[salsa::input]
pub struct SourceProgram {
    #[return_ref]
    pub text: String,
}

#[salsa::tracked]
pub struct Module {
    #[return_ref]
    pub statements: Vec<Function>,
}

#[salsa::tracked]
pub struct Function {
    #[id]
    pub name: FunctionId,
}

#[salsa::interned]
pub struct FunctionId {
    #[return_ref]
    pub text: String,
}

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

#[salsa::accumulator]
pub struct Diagnostics(Diagnostic);

#[derive(Clone)]
pub struct Diagnostic {
    pub message: String,
}

#[cfg(test)]
mod tests {
    use insta::assert_debug_snapshot;
    use salsa::DebugWithDb;

    use super::*;

    #[test]
    fn test_parse_module() {
        let db = &Database::default();
        let source = SourceProgram::new(db, "fn main".to_string());

        let module = parse_module(db, source);
        assert_debug_snapshot!(module.statements(db).debug(db), @r###"
        [
            Function {
                [salsa id]: 0,
                name: FunctionId {
                    [salsa id]: 0,
                    text: "main",
                },
            },
        ]
        "###);
    }
}
