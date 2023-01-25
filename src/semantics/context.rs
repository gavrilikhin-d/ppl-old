use std::{sync::Arc, collections::HashMap};

use crate::{hir::{ClassOrTrait, ParameterOrVariable, FunctionDeclaration, Module, TraitDeclaration, Type, Format, Name, FunctionNamePart, Expression, Typed}, named::Named, ast::CallNamePart};

/// Trait for various AST lowering contexts
pub trait Context {
	/// Get module context of builtin module
	fn builtin(&self) -> BuiltinContext;

	/// Find type by name
	fn find_type(&self, name: &str) -> Option<ClassOrTrait>;

	/// Find variable by name
	fn find_variable(&self, name: &str) -> Option<ParameterOrVariable>;

	/// Get visible functions
	fn functions(&self) -> HashMap<
		Format,
		HashMap<
			Name,
			Arc<FunctionDeclaration>
		>
	>;

	/// Get candidates for function call
	fn get_candidates(
		&self,
		name_parts: &[CallNamePart],
		args_cache: &[Option<Expression>]
	)
		-> Vec<Arc<FunctionDeclaration>>
	{
		// Add functions from current module
		let mut functions: Vec<_> = self.functions().values().flat_map(
			|f| f.values().filter(|f| f.name_parts.len() == name_parts.len())
		).cloned().collect();

		// Add functions from traits
		functions.extend(
			args_cache
				.iter()
				.filter_map(|a| a.as_ref())
				.filter_map(
					|a| if let Type::Trait(tr) = a.ty() {
						return Some(tr)
					}
					else {
						None
					}
				)
				.flat_map(
					|tr| tr.functions_with_n_name_parts(name_parts.len()).collect::<Vec<_>>()
				)
		);

		// Filter functions by name parts
		functions.iter().filter(
			|f| f.name_parts
			    .iter()
				.zip(name_parts)
				.enumerate()
				.all(
					|(i, (f_part, c_part))| match (f_part, c_part) {
					(FunctionNamePart::Text(text1), CallNamePart::Text(text2)) => text1.as_str() == text2.as_str(),
					(FunctionNamePart::Parameter(_), CallNamePart::Argument(_)) => true,
					(FunctionNamePart::Parameter(_), CallNamePart::Text(_)) => args_cache[i].is_some(),
					_ => false,
				})
		).cloned().collect()
	}
}

/// Helper struct to get builtin things
pub struct BuiltinContext {
	/// Builtin module
	module: Arc<Module>
}

impl BuiltinContext {
	/// Get builtin types
	pub fn types(&self) -> BuiltinTypes {
		BuiltinTypes { module: self.module.clone() }
	}
}

/// Helper struct to get builtin types
pub struct BuiltinTypes {
	/// Builtin module
	module: Arc<Module>
}

impl BuiltinTypes {
	/// Get builtin type by name
	fn get_type(&self, name: &str) -> Type {
		self.module.types.get(name).unwrap().clone().into()
	}

	/// Get builtin "None" type
	pub fn none(&self) -> Type {
		self.get_type("None")
	}

	/// Get builtin "Bool" type
	pub fn bool(&self) -> Type {
		self.get_type("Bool")
	}

	/// Get builtin "Integer" type
	pub fn integer(&self) -> Type {
		self.get_type("Integer")
	}

	/// Get builtin "String" type
	pub fn string(&self) -> Type {
		self.get_type("String")
	}
}

/// Context for lowering content of module
pub struct ModuleContext {
    /// Module, which is being lowered
    pub module: Arc<Module>,
	/// Builtin module context
	pub builtin: Option<Box<ModuleContext>>
}

impl Context for ModuleContext {
	fn builtin(&self) -> BuiltinContext {
		BuiltinContext {
			module:
				self.builtin.as_ref()
					.map(|c| c.module.clone())
					.unwrap_or_else(|| self.module.clone())
		}
	}

	fn find_type(&self, name: &str) -> Option<ClassOrTrait> {
		self.module
			.types.get(name).cloned()
			.or_else(|| self.builtin.as_ref().and_then(|b| b.find_type(name)))
	}

	fn find_variable(&self, name: &str) -> Option<ParameterOrVariable> {
		self.module
			.variables.get(name).cloned().map(|v| v.into())
			.or_else(
				|| self.builtin.as_ref().and_then(|b| b.find_variable(name))
			)
	}

	fn functions(&self) -> HashMap<
			Format,
			HashMap<
				Name,
				Arc<FunctionDeclaration>
			>
		> {
		let mut functions = self.module.functions.clone();
		functions.extend(
			self.builtin.as_ref()
				.map(|b| b.functions())
				.unwrap_or_else(HashMap::new)
		);
		functions
	}
}

/// Context for lowering body of function
pub struct FunctionContext<Parent: Context> {
	/// Function, which is being lowered
	pub function: Arc<FunctionDeclaration>,

	/// Parent context for this function
	pub parent: Parent
}

impl<T: Context> Context for FunctionContext<T> {
	fn builtin(&self) -> BuiltinContext {
		self.parent.builtin()
	}

	fn find_type(&self, name: &str) -> Option<ClassOrTrait> {
		self.parent.find_type(name)
	}

	fn find_variable(&self, name: &str) -> Option<ParameterOrVariable> {
		self.function
			.parameters()
				.find(|p| p.name() == name).map(|p| p.into())
				.or_else(|| self.parent.find_variable(name))
	}

	fn functions(&self) -> HashMap<
		Format,
		HashMap<
			Name,
			Arc<FunctionDeclaration>
		>
	> {
		// First insert this function
		let mut functions: HashMap<
			Format,
			HashMap<
				Name,
				Arc<FunctionDeclaration>
			>
		> = HashMap::new();
		functions.insert(
			self.function.name_format().to_string(),
			vec![(self.function.name().to_string(), self.function.clone())].into_iter().collect()
		);
		functions.extend(self.parent.functions());
		functions
	}
}

/// Context for lowering body of trait
pub struct TraitContext<Parent: Context> {
	/// Trait, which is being lowered
	pub tr: Arc<TraitDeclaration>,

	/// Parent context for this function
	pub parent: Parent
}

impl<T: Context> Context for TraitContext<T> {
	fn builtin(&self) -> BuiltinContext {
		self.parent.builtin()
	}

	fn find_type(&self, name: &str) -> Option<ClassOrTrait> {
		self.parent.find_type(name)
	}

	fn find_variable(&self, name: &str) -> Option<ParameterOrVariable> {
		self.parent.find_variable(name)
	}

	fn functions(&self) -> HashMap<
			Format,
			HashMap<
				Name,
				Arc<FunctionDeclaration>
			>
		> {
		// TODO: insert functions from trait
		self.parent.functions()
	}
}