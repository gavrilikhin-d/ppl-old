// use super::{hir, Typed};
// use crate::semantics;

// pub fn create_runnable_module(name: &str) -> inkwell::module::Module {
// 	let context = inkwell::context::Context::create();
// 	let module = context.create_module(name);

// 	module.add_function("run", context.void_type().fn_type(&mut vec![], false));
// 	module
// }

// pub struct Context {
// 	/// LLVM context
// 	context: inkwell::context::Context
// }

// impl Context {
// 	pub fn convert_type(&self, ty: &semantics::Type) {
// 		match ty {
// 			semantics::Type::None => self.context.void_type()
// 		}
// 	}
// }

// pub fn create_function_for_expression(expr: hir::Expression) -> inkwell::values::FunctionValue {
// 	let return_type = convert_type(&expr.get_type());
// }
