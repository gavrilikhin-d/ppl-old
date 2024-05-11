use crate::{
    hir::{self, Function, Trait, Type},
    syntax::Ranged,
    DataHolder,
};

use super::{error::NotImplemented, Context};

/// Trait to check if type implements trait
pub trait Implements
where
    Self: Sized,
{
    /// Does this class implement given trait?
    fn implements(&self, tr: Trait) -> ImplementsCheck<Self> {
        ImplementsCheck { ty: self, tr }
    }
}

impl Implements for hir::Class {}

/// Helper struct to do check within context
pub struct ImplementsCheck<'s, S> {
    ty: &'s S,
    tr: Trait,
}

impl ImplementsCheck<'_, hir::Class> {
    pub fn within(self, context: &mut impl Context) -> Result<Vec<Function>, NotImplemented> {
        let mut implemented = vec![];
        for supertrait in &self.tr.read().unwrap().supertraits {
            implemented.extend(
                self.ty
                    .implements(supertrait.clone())
                    .within(context)?
                    .into_iter(),
            );
        }

        let mut unimplemented = vec![];
        for f in self.tr.read().unwrap().functions.values().cloned() {
            if f.read().unwrap().is_definition() {
                implemented.push(f);
                continue;
            }

            if let Some(imp) =
                context.find_implementation(&f.read().unwrap(), &Type::from(self.ty.clone()))
            {
                implemented.push(imp);
                continue;
            }

            unimplemented.push(f);
        }

        if !unimplemented.is_empty() {
            let source_file = self
                .tr
                .read()
                .unwrap()
                .module
                .data(context.compiler())
                .source_file()
                .clone();
            return Err(NotImplemented {
                ty: self.ty.clone().into(),
                tr: self.tr,
                unimplemented: unimplemented
                    .into_iter()
                    .map(|f| f.range().into())
                    .collect(),
                source_file,
            });
        }

        Ok(implemented)
    }
}
