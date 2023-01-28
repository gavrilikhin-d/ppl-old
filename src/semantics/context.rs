use std::{sync::{Arc, Weak}, ops::Range, collections::HashMap};

use crate::{hir::{ParameterOrVariable, Module, TraitDeclaration, Type, FunctionNamePart, Expression, Typed, Function, FunctionDefinition, SelfType, TypeDeclaration, VariableDeclaration, CallKind, Name, FunctionDeclaration, Parameter}, named::Named, ast::CallNamePart, syntax::Ranged};

use super::error::{CandidateNotViable, ArgumentTypeMismatch, NoFunction, Error};

/// Trait for various AST lowering contexts
pub trait Context {
	/// Is this a context for builtin module?
	fn is_for_builtin_module(&self) -> bool;

	/// Get module context of builtin module
	fn builtin(&self) -> BuiltinContext;

	/// Get current function
	fn function(&self) -> Option<&FunctionDefinition>;

	/// Find type by name
	fn find_type(&self, name: &str) -> Option<Type>;

	/// Find variable by name
	fn find_variable(&self, name: &str) -> Option<ParameterOrVariable>;

	/// Add type to context
	fn add_type(&mut self, ty: Arc<TypeDeclaration>);

	/// Add trait to context
	fn add_trait(&mut self, tr: Arc<TraitDeclaration>);

	/// Add function to context
	fn add_function(&mut self, f: Function);

	/// Add variable to context
	fn add_variable(&mut self, v: Arc<VariableDeclaration>);

	/// Get all visible functions
	fn functions_with_n_name_parts(&self, n: usize) -> Vec<Function>;

	/// Get candidates for function call
	fn candidates(
		&self,
		name_parts: &[CallNamePart],
		args_cache: &[Option<Expression>]
	) -> Vec<Function>
	{
		let mut functions = self.functions_with_n_name_parts(name_parts.len());
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
					|tr| tr.functions_with_n_name_parts(name_parts.len())
							.cloned()
							.collect::<Vec<_>>()
				)
		);

		// Filter functions by name parts
		functions.iter().filter(
			|f| f.name_parts()
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

	/// Get all functions with same name format
	fn functions_with_format(&self, format: &str) -> HashMap<Name, Function>;

	/// Recursively find function with same name format and arguments
	fn get_function(
		&self,
		range: Range<usize>,
		format: &str,
		args: &[Expression],
		kind: CallKind,
	) -> Result<Function, Error>
	{
		let functions = self.functions_with_format(format);
		// TODO: Add functions from traits
		let mut name = format.to_string();
		for arg in args {
			name = name.replacen("<>", format!("<:{}>", arg.ty()).as_str(), 1);
		}
		let arguments = args
			.iter()
			.map(|arg| (arg.ty(), arg.range().into()))
			.collect::<Vec<_>>();

		let f = functions.get(name.as_str());
		if f.is_none() {
			let mut candidates: Vec<CandidateNotViable> = Vec::new();
			for candidate in functions.values() {
				for (param, arg) in candidate.parameters().zip(args) {
					if param.ty() != arg.ty() {
						candidates.push(CandidateNotViable {
							reason: ArgumentTypeMismatch {
								expected: param.ty(),
								expected_span: param.name.range().into(),
								got: arg.ty(),
								got_span: arg.range().into(),
							}
							.into(),
						});
						break;
					}
				}
			}

			return Err(
				NoFunction {
					kind,
					name,
					at: range.into(),
					arguments,
					candidates,
				}.into()
			);
		}

		Ok(f.unwrap().clone())
	}

	/// Find concrete function for trait function
	fn find_implementation(&self, trait_fn: &Function, self_type: &Type) -> Option<Function> {
		let funcs =
		self.functions_with_n_name_parts(trait_fn.name_parts().len());
		funcs.iter().find(
			|f| trait_fn.name_parts().iter().zip(f.name_parts()).all(
				|(a, b)| {
					match (a, b) {
						(FunctionNamePart::Text(a), FunctionNamePart::Text(b))
							=> a.as_str() == b.as_str(),
						(FunctionNamePart::Parameter(a), FunctionNamePart::Parameter(b))
							=> a.ty().map_self(self_type) == &b.ty(),
						_ => false
					}
				}
			) && trait_fn.return_type().map_self(self_type) == &f.return_type()
		).cloned()
	}

	/// Monomorphize generic function
	fn monomorphize(&mut self, f: &Function, args: &[Expression])
		-> Arc<FunctionDeclaration> {
		// Get mapping of generic types to concrete types
		let mut mapping = HashMap::new();
		for (param, arg) in f.parameters().zip(args) {
			match param.ty() {
				Type::Trait(tr) => {
					mapping.insert(Arc::as_ptr(&tr), arg.ty());
				},
				_ => {}
			}
		}

		let mut arg = args.into_iter().map(|arg| arg.ty());
		let name_parts = f.name_parts().iter().map(
			|part| match part {
				FunctionNamePart::Text(text) => text.clone().into(),
				FunctionNamePart::Parameter(param) =>
					Arc::new(
						Parameter {
							name: param.name.clone(),
							ty: {
								let arg_ty = arg.next().unwrap();
								match param.ty() {
									Type::Trait(_) => arg_ty,
									_ => param.ty()
								}
							}
						}
					).into()
			}
		).collect::<Vec<_>>();

		let declaration = Arc::new(
			FunctionDeclaration::build()
				.with_name(name_parts)
				.with_return_type(f.return_type())
		);

		if let Function::Definition(def) = f {
			todo!("Monomorphize function definition")
		}

		// self.add_function(declaration.clone().into());

		declaration
	}
}

/// Helper struct to get builtin things
pub struct BuiltinContext<'m> {
	/// Builtin module
	module: &'m Module
}

impl<'m> BuiltinContext<'m> {
	/// Get builtin types
	pub fn types(&self) -> BuiltinTypes<'m> {
		BuiltinTypes { module: self.module }
	}
}

/// Helper struct to get builtin types
pub struct BuiltinTypes<'m> {
	/// Builtin module
	module: &'m Module
}

impl BuiltinTypes<'_> {
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
    pub module: Module
}

impl Default for ModuleContext {
	fn default() -> Self {
		Self {
			module: Module::default(),
		}
	}
}

impl Context for ModuleContext {
	fn is_for_builtin_module(&self) -> bool {
		self.module.is_builtin
	}

	fn builtin(&self) -> BuiltinContext {
		if self.module.is_builtin {
			BuiltinContext {
				module: &self.module
			}
		}
		else {
			BuiltinContext {
				module: Module::builtin()
			}
		}
	}

	fn function(&self) -> Option<&FunctionDefinition> { None }

	fn find_type(&self, name: &str) -> Option<Type> {
		let ty = self.module.types.get(name).cloned().map(|t| t.into());
		if ty.is_none() && !self.module.is_builtin {
			return Module::builtin().types.get(name).cloned().map(|t| t.into());
		}
		ty
	}

	fn find_variable(&self, name: &str) -> Option<ParameterOrVariable> {
		let var = self.module.variables.get(name).cloned().map(|v| v.into());
		if var.is_none() && !self.module.is_builtin {
			return Module::builtin().variables.get(name).cloned().map(|v| v.into());
		}
		var
	}

	/// Add type to context
	fn add_type(&mut self, ty: Arc<TypeDeclaration>) {
		self.module.types.insert(ty.name().to_string(), ty.into());
	}

	fn add_trait(&mut self, tr: Arc<TraitDeclaration>) {
		self.module.types.insert(tr.name().to_string(), tr.into());
	}

	fn add_function(&mut self, f: Function) {
		self.module.insert_function(f);
	}

	fn add_variable(&mut self, v: Arc<VariableDeclaration>) {
		self.module.variables.insert(v.name().to_string(), v);
	}

	/// Get all visible functions
	fn functions_with_n_name_parts(&self, n: usize) -> Vec<Function> {
		let mut functions: Vec<_> =
			self.module.functions_with_n_name_parts(n).cloned().collect();
		if !self.module.is_builtin {
			functions.extend(
				Module::builtin().functions_with_n_name_parts(n).cloned()
			);
		}
		functions
	}

	fn functions_with_format(&self, format: &str) -> HashMap<Name, Function>
	{
		let mut funcs = self.module.functions.get(format).cloned().unwrap_or_default();
		if !self.module.is_builtin {
			funcs.extend(
				Module::builtin().functions.get(format).cloned().unwrap_or_default()
			)
		}
		funcs
	}
}

/// Context for lowering body of function
pub struct FunctionContext<'p> {
	/// Function, which is being lowered
	pub function: FunctionDefinition,

	/// Parent context for this function
	pub parent: &'p mut dyn Context
}

impl Context for FunctionContext<'_> {
	fn is_for_builtin_module(&self) -> bool {
		self.parent.is_for_builtin_module()
	}

	fn builtin(&self) -> BuiltinContext {
		self.parent.builtin()
	}

	fn function(&self) -> Option<&FunctionDefinition> { Some(&self.function) }

	fn find_type(&self, name: &str) -> Option<Type> {
		self.parent.find_type(name)
	}

	fn find_variable(&self, name: &str) -> Option<ParameterOrVariable> {
		self.function
			.parameters()
				.find(|p| p.name() == name).map(|p| p.into())
				.or_else(|| self.parent.find_variable(name))
	}

	fn add_type(&mut self, ty: Arc<TypeDeclaration>) {
		todo!("local types")
	}

	fn add_trait(&mut self, tr: Arc<TraitDeclaration>) {
		todo!("local traits")
	}

	fn add_function(&mut self, f: Function) {
		todo!("local functions")
	}

	fn add_variable(&mut self, v: Arc<VariableDeclaration>) {
		todo!("local variables")
	}

	fn functions_with_n_name_parts(&self, n: usize) -> Vec<Function> {
		self.parent.functions_with_n_name_parts(n)
	}

	fn functions_with_format(&self, format: &str) -> HashMap<Name, Function> {
		self.parent.functions_with_format(format)
	}
}

/// Context for lowering body of trait
pub struct TraitContext<'p> {
	/// Trait, which is being lowered
	pub tr: TraitDeclaration,

	/// Uninitialized weak pointer to trait
	pub trait_weak: Weak<TraitDeclaration>,

	/// Parent context for this function
	pub parent: &'p mut dyn Context
}

impl Context for TraitContext<'_> {
	fn is_for_builtin_module(&self) -> bool {
		self.parent.is_for_builtin_module()
	}

	fn builtin(&self) -> BuiltinContext {
		self.parent.builtin()
	}

	fn function(&self) -> Option<&FunctionDefinition> { self.parent.function() }

	fn find_type(&self, name: &str) -> Option<Type> {
		if name == "Self" {
			return Some(SelfType {
				associated_trait: self.trait_weak.clone()
			}.into())
		}
		self.parent.find_type(name)
	}

	fn find_variable(&self, name: &str) -> Option<ParameterOrVariable> {
		self.parent.find_variable(name)
	}

	fn add_type(&mut self, ty: Arc<TypeDeclaration>) {
		todo!("types in traits")
	}

	fn add_trait(&mut self, tr: Arc<TraitDeclaration>) {
		todo!("traits in traits?")
	}

	fn add_function(&mut self, f: Function) {
		self.tr.functions.push(f)
	}

	fn add_variable(&mut self, v: Arc<VariableDeclaration>) {
		todo!("variables in traits")
	}

	fn functions_with_n_name_parts(&self, n: usize) -> Vec<Function> {
		let mut functions = self.parent.functions_with_n_name_parts(n);
		functions.extend(
			self.tr.functions.iter()
				.filter(move |f| f.name_parts().len() == n)
				.cloned()
		);
		functions
	}

	fn functions_with_format(&self, format: &str) -> HashMap<Name, Function> {
		todo!("functions with format in traits")
	}
}