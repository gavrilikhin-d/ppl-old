use std::{sync::{Arc, Mutex}, collections::HashMap};

use crate::{hir::{ClassOrTrait, ParameterOrVariable, FunctionDeclaration, Module, TraitDeclaration, Type, Expression}, ast::CallNamePart};

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
		FunctionFormat,
		HashMap<
			FunctionName,
			Arc<Mutex<FunctionDeclaration>>
		>
	>
}

/// Helper struct to get builtin things
pub struct BuiltinContext {
	/// Builtin module
	module: Arc<Mutex<Module>>
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
	module: Arc<Mutex<Module>>
}

impl BuiltinTypes {
	/// Get builtin type by name
	fn get_type(&self, name: &str) -> Type {
		self.module.lock().unwrap().types.get(name).unwrap().value.into()
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
    pub module: Arc<Mutex<Module>>,
	/// Builtin module context
	pub builtin: Option<ModuleContext>
}

impl Context for ModuleContext {
	fn builtin(&self) -> BuiltinContext {
		BuiltinContext {
			module:
				self.builtin
					.map(|c| c.module.clone())
					.unwrap_or_else(|| self.module.clone())
		}
	}

	fn find_type(&self, name: &str) -> Option<ClassOrTrait> {
		self.module.lock().unwrap()
			.types.get(name).map(|t| t.value.clone())
			.or_else(|| self.builtin.as_ref().and_then(|b| b.find_type(name)))
	}

	fn find_variable(&self, name: &str) -> Option<ParameterOrVariable> {
		self.module.lock().unwrap()
			.variables.get(name).map(|v| v.value.clone())
			.or_else(
				|| self.builtin.as_ref().and_then(|b| b.find_variable(name))
			)
	}
}

/// Context for lowering body of function
pub struct FunctionContext<Parent: Context> {
	/// Function, which is being lowered
	pub function: Arc<Mutex<FunctionDeclaration>>,

	/// Parent context for this function
	pub parent: Parent
}

impl Context for FunctionContext<_> {
	fn builtin(&self) -> BuiltinContext {
		self.parent.builtin()
	}

	fn find_type(&self, name: &str) -> Option<ClassOrTrait> {
		self.parent.find_type(name)
	}

	fn find_variable(&self, name: &str) -> Option<ParameterOrVariable> {
		self.function.lock().unwrap()
			.parameters()
				.find(|p| p.name() == name)
				.or_else(|| self.parent.find_variable(name))
	}
}

/// Context for lowering body of trait
pub struct TraitContext<Parent: Context> {
	/// Trait, which is being lowered
	pub tr: Arc<Mutex<TraitDeclaration>>,

	/// Parent context for this function
	pub parent: Parent
}

impl Context for TraitContext<_> {
	fn builtin(&self) -> BuiltinContext {
		self.parent.builtin()
	}

	fn find_type(&self, name: &str) -> Option<ClassOrTrait> {
		self.parent.find_type(name)
	}

	fn find_variable(&self, name: &str) -> Option<ParameterOrVariable> {
		self.parent.find_variable(name)
	}
}