use std::borrow::Cow;
use std::fmt::Display;
use std::ops::Range;
use std::sync::{Arc, LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard};

use derive_more::From;
use derive_visitor::DriveMut;

use crate::compilation::Module;
use crate::hir::{FunctionType, Generic, Statement, Type, TypeReference, Typed};
use crate::mutability::Mutable;
use crate::named::Named;
use crate::syntax::{Identifier, Keyword, Ranged};

use super::Trait;

/// Parameter data holder
#[derive(Debug, Clone)]
pub struct Parameter {
    inner: Arc<RwLock<ParameterData>>,
}

impl Parameter {
    /// Create a new parameter from its data
    pub fn new(data: ParameterData) -> Self {
        Self {
            inner: Arc::new(RwLock::new(data)),
        }
    }

    /// Lock variable for reading
    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, ParameterData>> {
        self.inner.read()
    }

    /// Lock variable for writing
    pub fn write(&self) -> LockResult<RwLockWriteGuard<'_, ParameterData>> {
        self.inner.write()
    }
}

impl PartialEq for Parameter {
    fn eq(&self, other: &Self) -> bool {
        *self.read().unwrap() == *other.read().unwrap()
    }
}

impl Eq for Parameter {}

impl Named for Parameter {
    fn name(&self) -> Cow<'_, str> {
        self.read().unwrap().name().to_string().into()
    }
}

impl Generic for Parameter {
    fn is_generic(&self) -> bool {
        self.read().unwrap().is_generic()
    }
}

impl Typed for Parameter {
    fn ty(&self) -> Type {
        self.read().unwrap().ty()
    }
}

impl Mutable for Parameter {
    fn is_mutable(&self) -> bool {
        self.read().unwrap().is_mutable()
    }
}

impl Ranged for Parameter {
    fn range(&self) -> std::ops::Range<usize> {
        self.read().unwrap().range()
    }
}

impl DriveMut for Parameter {
    fn drive_mut<V: derive_visitor::VisitorMut>(&mut self, visitor: &mut V) {
        derive_visitor::VisitorMut::visit(visitor, self, ::derive_visitor::Event::Enter);
        self.write().unwrap().drive_mut(visitor);
        derive_visitor::VisitorMut::visit(visitor, self, ::derive_visitor::Event::Exit);
    }
}

/// Declaration of a function parameter
#[derive(Debug, PartialEq, Eq, Clone, DriveMut)]
pub struct ParameterData {
    /// Type's name
    #[drive(skip)]
    pub name: Identifier,
    /// Type of parameter
    #[drive(skip)]
    pub ty: TypeReference,
    /// Range of the whole parameter
    #[drive(skip)]
    pub range: Range<usize>,
}

impl Ranged for ParameterData {
    fn range(&self) -> std::ops::Range<usize> {
        self.range.clone()
    }
}

impl Generic for ParameterData {
    /// Is this a generic parameter?
    fn is_generic(&self) -> bool {
        self.ty.referenced_type.is_generic()
    }
}

impl Named for ParameterData {
    /// Get name of parameter
    fn name(&self) -> Cow<'_, str> {
        self.name.as_str().into()
    }
}

impl Typed for ParameterData {
    /// Get type of parameter
    fn ty(&self) -> Type {
        self.ty.referenced_type.clone()
    }
}

impl Mutable for ParameterData {
    fn is_mutable(&self) -> bool {
        self.ty.is_mutable()
    }
}

/// Part of a function name
#[derive(Debug, PartialEq, Eq, Clone, From, DriveMut)]
pub enum FunctionNamePart {
    #[drive(skip)]
    Text(Identifier),
    Parameter(Parameter),
}

impl Display for FunctionNamePart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionNamePart::Text(text) => write!(f, "{}", text),
            FunctionNamePart::Parameter(parameter) => {
                if parameter.name().is_empty() {
                    write!(f, "<:{}>", parameter.ty())
                } else {
                    write!(f, "<{}: {}>", parameter.name(), parameter.ty())
                }
            }
        }
    }
}

impl From<ParameterData> for FunctionNamePart {
    fn from(parameter: ParameterData) -> Self {
        Parameter::new(parameter).into()
    }
}

impl Ranged for FunctionNamePart {
    fn range(&self) -> std::ops::Range<usize> {
        match self {
            FunctionNamePart::Text(text) => text.range(),
            FunctionNamePart::Parameter(parameter) => parameter.range(),
        }
    }
}

/// Function data holder
#[derive(Debug, Clone)]
pub struct Function {
    inner: Arc<RwLock<FunctionData>>,
}

impl Function {
    /// Create a new function from its data
    pub fn new(data: FunctionData) -> Self {
        Function {
            inner: Arc::new(RwLock::new(data)),
        }
    }

    /// Lock function for reading
    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, FunctionData>> {
        self.inner.read()
    }

    /// Lock function for writing
    pub fn write(&self) -> LockResult<RwLockWriteGuard<'_, FunctionData>> {
        self.inner.write()
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read().unwrap().fmt(f)
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        *self.read().unwrap() == *other.read().unwrap()
    }
}

impl Eq for Function {}

impl Named for Function {
    fn name(&self) -> Cow<'_, str> {
        self.read().unwrap().name().to_string().into()
    }
}

impl Ranged for Function {
    fn range(&self) -> Range<usize> {
        self.read().unwrap().range()
    }
}

impl DriveMut for Function {
    fn drive_mut<V: derive_visitor::VisitorMut>(&mut self, visitor: &mut V) {
        derive_visitor::VisitorMut::visit(visitor, self, ::derive_visitor::Event::Enter);
        self.write().unwrap().drive_mut(visitor);
        derive_visitor::VisitorMut::visit(visitor, self, ::derive_visitor::Event::Exit);
    }
}

/// Declaration (or definition) of a function
#[derive(Debug, PartialEq, Eq, Clone, DriveMut)]
pub struct FunctionData {
    /// Keyword `fn`
    #[drive(skip)]
    pub keyword: Keyword<"fn">,
    /// Generic parameters of a function
    #[drive(skip)]
    pub generic_types: Vec<Type>,
    /// Type's name
    pub name_parts: Vec<FunctionNamePart>,
    /// Type of returned value
    #[drive(skip)]
    pub return_type: Type,

    /// Optional body of a function
    pub body: Vec<Statement>,

    /// Module where function is defined
    #[drive(skip)]
    pub module: Module,

    /// Trait this function defined in
    #[drive(skip)]
    pub tr: Option<Trait>,

    /// Mangled name to use instead of default
    #[drive(skip)]
    pub(crate) mangled_name: Option<String>,
    /// Cached format for name of function
    #[drive(skip)]
    pub(crate) name_format: String,
    /// Cached name of function
    #[drive(skip)]
    pub(crate) name: String,
}

impl FunctionData {
    /// Create a new builder for a function declaration
    pub fn build(module: Module, keyword: Keyword<"fn">) -> FunctionBuilder {
        FunctionBuilder::new(module, keyword)
    }

    /// Get name parts of function
    pub fn name_parts(&self) -> &[FunctionNamePart] {
        &self.name_parts
    }

    /// Format for the function's name
    ///
    /// # Example
    ///
    /// The following functions:
    /// ```ppl
    /// fn print <x: None> -> None
    /// fn print <x: Integer> -> None
    /// ```
    /// both have `print <>` name format
    pub fn name_format(&self) -> &str {
        &self.name_format
    }

    /// Get iterator over function's parameters
    pub fn parameters(&self) -> impl Iterator<Item = Parameter> + '_ {
        self.name_parts.iter().filter_map(|part| match part {
            FunctionNamePart::Parameter(p) => Some(p.clone()),
            _ => None,
        })
    }

    /// Get mangled name of function
    pub fn mangled_name(&self) -> Cow<'_, str> {
        self.mangled_name
            .as_deref()
            .map(|n| n.into())
            .unwrap_or(self.name())
    }

    /// Build function name from name parts
    pub fn build_name(name_parts: &[FunctionNamePart]) -> String {
        let mut name = String::new();
        for (i, part) in name_parts.iter().enumerate() {
            if i > 0 {
                name.push_str(" ");
            }

            match part {
                FunctionNamePart::Text(text) => name.push_str(&text),
                FunctionNamePart::Parameter(p) => {
                    name.push_str(format!("<:{}>", p.ty().name()).as_str())
                }
            }
        }
        name
    }

    /// Is this a definition of a function?
    pub fn is_definition(&self) -> bool {
        !self.body.is_empty()
    }
}

impl Ranged for FunctionData {
    fn start(&self) -> usize {
        self.keyword.start()
    }

    fn end(&self) -> usize {
        // FIXME: return end of return type annotation for fallback, if any
        self.body.last().map_or_else(
            || {
                self.name_parts
                    .last()
                    .map(|p| p.end())
                    .unwrap_or(self.keyword.end())
            },
            |s| s.end(),
        )
    }
}

impl Generic for FunctionData {
    fn is_generic(&self) -> bool {
        self.parameters().any(|p| p.is_generic()) || self.return_type.is_generic()
    }
}

impl Named for FunctionData {
    /// Get name of function
    fn name(&self) -> Cow<'_, str> {
        self.name.as_str().into()
    }
}

impl Typed for FunctionData {
    fn ty(&self) -> Type {
        FunctionType::build()
            .with_parameters(
                self.name_parts
                    .iter()
                    .filter_map(|part| match part {
                        FunctionNamePart::Parameter(p) => Some(p.ty()),
                        _ => None,
                    })
                    .collect(),
            )
            .with_return_type(self.return_type.clone())
            .into()
    }
}

impl Display for FunctionData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.body.is_empty() {
            writeln!(f, "")?;
        }

        let indent = "\t".repeat(f.width().unwrap_or(0));
        write!(f, "{indent}")?;

        let generics = if self.generic_types.is_empty() {
            String::new()
        } else {
            format!(
                "<{}>",
                self.generic_types
                    .iter()
                    .map(|ty| format!("{ty:+}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        let name_parts = self
            .name_parts
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        let return_type = self.return_type.name();
        write!(f, "fn{generics} {name_parts} -> {return_type}")?;

        if self.body.is_empty() {
            return Ok(());
        }

        writeln!(f, ":")?;
        let new_indent = f.width().unwrap_or(0) + 1;
        for statement in &self.body {
            writeln!(f, "{statement:#new_indent$}")?;
        }
        Ok(())
    }
}

/// Builder for a function declaration
pub struct FunctionBuilder {
    /// Module where function is defined
    module: Module,
    /// Keyword `fn`
    keyword: Keyword<"fn">,
    /// Generic parameters of a function
    generic_types: Vec<Type>,
    /// Type's name
    name_parts: Vec<FunctionNamePart>,
    /// Mangled name of function
    mangled_name: Option<String>,
    /// Body of a function
    body: Vec<Statement>,
}

impl FunctionBuilder {
    /// Create a new builder for a function
    pub fn new(module: Module, keyword: Keyword<"fn">) -> Self {
        FunctionBuilder {
            module,
            keyword,
            generic_types: Vec::new(),
            name_parts: Vec::new(),
            mangled_name: None,
            body: vec![],
        }
    }

    /// Set generic parameters of a function
    pub fn with_generic_types(mut self, generic_types: Vec<Type>) -> Self {
        self.generic_types = generic_types;
        self
    }

    /// Set name parts of the function
    pub fn with_name(mut self, name_parts: Vec<FunctionNamePart>) -> Self {
        self.name_parts = name_parts;
        self
    }

    /// Set mangled name of function
    pub fn with_mangled_name(mut self, mangled_name: Option<String>) -> Self {
        self.mangled_name = mangled_name;
        self
    }

    /// Set body of function
    pub fn with_body(mut self, body: Vec<Statement>) -> Self {
        self.body = body;
        self
    }

    pub fn without_body(mut self) -> Self {
        self.body = vec![];
        self
    }

    /// Build function's name format
    fn build_name_format(&self) -> String {
        let mut name_format = String::new();
        for (i, part) in self.name_parts.iter().enumerate() {
            if i > 0 {
                name_format.push_str(" ");
            }
            name_format.push_str(match part {
                FunctionNamePart::Text(text) => &text,
                FunctionNamePart::Parameter(_) => "<>",
            })
        }
        name_format
    }

    /// Build function's name
    fn build_name(&self) -> String {
        FunctionData::build_name(&self.name_parts)
    }

    /// Set the return type of the function and return the declaration
    pub fn with_return_type(self, return_type: Type) -> FunctionData {
        let name_format = self.build_name_format();
        let name = self.build_name();
        FunctionData {
            module: self.module,
            tr: None,
            keyword: self.keyword,
            generic_types: self.generic_types,
            name_parts: self.name_parts,
            return_type,
            name_format,
            name,
            mangled_name: self.mangled_name,
            body: self.body,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast,
        compilation::Module,
        hir::{
            FunctionData, GenericType, ParameterData, Return, Statement, TypeReference,
            VariableReference,
        },
        semantics::ToHIR,
        syntax::{Identifier, Keyword},
    };

    use pretty_assertions::assert_eq;

    #[test]
    fn generic_parameters() {
        let ast = "fn<T> <x: T> -> T => x"
            .parse::<ast::FunctionDeclaration>()
            .unwrap();
        let hir = ast.to_hir_without_context().unwrap();

        let ty = GenericType {
            name: Identifier::from("T").at(3),
            generated: false,
            constraint: None,
        };
        let param = ParameterData {
            name: Identifier::from("x").at(7),
            ty: TypeReference {
                referenced_type: ty.clone().into(),
                span: 10..11,
                type_for_type: hir
                    .read()
                    .unwrap()
                    .parameters()
                    .next()
                    .unwrap()
                    .read()
                    .unwrap()
                    .ty
                    .type_for_type
                    .clone(),
            },
            range: 6..12,
        };
        assert_eq!(
            *hir.read().unwrap(),
            FunctionData::build(Module::with_index(0), Keyword::<"fn">::at(0))
                .with_generic_types(vec![ty.clone().into()])
                .with_name(vec![param.clone().into()])
                .with_body(vec![Statement::Return(Return::Implicit {
                    value: VariableReference {
                        span: 21..22,
                        variable: param.into()
                    }
                    .into()
                })])
                .with_return_type(ty.into())
        );
    }
}
