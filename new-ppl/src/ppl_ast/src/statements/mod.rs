use derive_more::From;
use salsa::DebugWithDb;

use crate::{annotation::Annotation, declarations::Declaration, expressions::Expression, Db};

#[derive(Debug, PartialEq, Eq, Clone, From)]
pub enum Statement {
    Declaration(Declaration),
    Expression(Expression),
}

impl<DB: Sized + Db> DebugWithDb<DB> for Statement {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        db: &DB,
        include_all_fields: bool,
    ) -> std::fmt::Result {
        use Statement::*;
        match self {
            Declaration(d) => d.fmt(f, db, include_all_fields),
            Expression(e) => e.fmt(f, db, include_all_fields),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, From)]
pub struct AnnotatedStatement {
    pub annotations: Vec<Annotation>,
    pub statement: Statement,
}

impl<DB: Sized + Db> DebugWithDb<DB> for AnnotatedStatement {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        db: &DB,
        include_all_fields: bool,
    ) -> std::fmt::Result {
        if self.annotations.is_empty() {
            return self.statement.fmt(f, db, include_all_fields);
        }

        f.debug_struct("AnnotatedStatement")
            .field(
                "annotations",
                &self.annotations.debug_with(db, include_all_fields),
            )
            .field(
                "statement",
                &self.statement.debug_with(db, include_all_fields),
            )
            .finish()
    }
}
