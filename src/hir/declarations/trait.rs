use std::{
    borrow::Cow,
    fmt::Display,
    hash::{Hash, Hasher},
    ops::Range,
    sync::{Arc, LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use derive_visitor::DriveMut;
use indexmap::IndexMap;

use crate::{
    compilation::Module,
    named::Named,
    syntax::{Identifier, Keyword, Ranged},
    AddSourceLocation,
};

use super::Function;
use crate::DataHolder;

/// Trait data holder
#[derive(Debug, Clone)]
pub struct Trait {
    inner: Arc<RwLock<TraitData>>,
}

impl DataHolder for Trait {
    type Data = TraitData;

    fn new(data: Self::Data) -> Self {
        Self {
            inner: Arc::new(RwLock::new(data)),
        }
    }

    fn inner(&self) -> &Arc<RwLock<Self::Data>> {
        &self.inner
    }
}

impl Display for Trait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read().unwrap().fmt(f)
    }
}

impl PartialEq for Trait {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}
impl Eq for Trait {}

impl Named for Trait {
    fn name(&self) -> Cow<'_, str> {
        self.read().unwrap().name().to_string().into()
    }
}

impl Ranged for Trait {
    fn range(&self) -> Range<usize> {
        self.read().unwrap().range()
    }
}

impl Hash for Trait {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.read().unwrap().hash(state)
    }
}

impl DriveMut for Trait {
    fn drive_mut<V: derive_visitor::VisitorMut>(&mut self, visitor: &mut V) {
        derive_visitor::VisitorMut::visit(visitor, self, ::derive_visitor::Event::Enter);
        self.write().unwrap().drive_mut(visitor);
        derive_visitor::VisitorMut::visit(visitor, self, ::derive_visitor::Event::Exit);
    }
}

/// Declaration of a trait
#[derive(Debug, PartialEq, Eq, Clone, DriveMut)]
pub struct TraitData {
    /// Keyword `trait`
    #[drive(skip)]
    pub keyword: Keyword<"trait">,
    /// Trait's name
    #[drive(skip)]
    pub name: Identifier,
    /// Supertraits
    #[drive(skip)]
    pub supertraits: Vec<Trait>,
    /// Associated functions
    pub functions: IndexMap<String, Function>,
    /// Module this trait is located in
    #[drive(skip)]
    pub module: Module,
}

impl TraitData {
    /// Iterate over all functions
    pub fn all_functions(&self) -> impl Iterator<Item = Function> + '_ {
        let mut functions: Vec<_> = self.functions.values().cloned().collect();

        self.supertraits
            .iter()
            .for_each(|tr| functions.extend(tr.read().unwrap().all_functions()));

        functions.into_iter()
    }

    /// Iterate over all functions with `n` name parts
    pub fn functions_with_n_name_parts(&self, n: usize) -> impl Iterator<Item = Function> + '_ {
        self.all_functions()
            .filter(move |f| f.read().unwrap().name_parts().len() == n)
    }
}

impl Named for TraitData {
    fn name(&self) -> Cow<'_, str> {
        self.name.as_str().into()
    }
}

impl Display for TraitData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            let indent = f.width().unwrap_or(0);
            let new_indent = indent + 1;

            let indent = "\t".repeat(indent);
            write!(f, "{indent}")?;

            writeln!(f, "trait {}:", self.name())?;
            for function in self.functions.values() {
                let function = function.read().unwrap();
                writeln!(f, "{function:#new_indent$}")?;
            }
        } else {
            write!(f, "{}", self.name())?;
        }
        Ok(())
    }
}

impl AddSourceLocation for Trait {}

impl Hash for TraitData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Ranged for TraitData {
    fn start(&self) -> usize {
        self.keyword.start()
    }

    fn end(&self) -> usize {
        self.functions
            .values()
            .last()
            .map_or(self.name.end(), |f| f.end())
    }
}
