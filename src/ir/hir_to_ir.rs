use std::sync::Arc;

use inkwell::types::BasicMetadataTypeEnum;

use inkwell::values::BasicMetadataValueEnum;

use super::inkwell::*;
use crate::hir::*;
use crate::mutability::Mutable;
use crate::named::Named;

use super::Context;
use super::FunctionContext;
use super::ModuleContext;

/// Trait for lowering HIR for global declarations to IR within module context
pub trait GlobalHIRLowering<'llvm> {
    type IR;

    /// Lower HIR for global declaration to IR within module context
    fn lower_global_to_ir(&self, context: &mut ModuleContext<'llvm>) -> Self::IR;
}

/// Trait for lowering HIR for local declarations to IR within function context
pub trait LocalHIRLowering<'llvm, 'm> {
    type IR;

    /// Lower HIR for local declaration to IR within function context
    fn lower_local_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR;
}

/// Trait for lowering HIR to IR within function context
pub trait HIRLoweringWithinFunctionContext<'llvm, 'm> {
    type IR;

    /// Lower HIR to IR within function context
    fn lower_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR;
}

/// Trait for convenient lowering of PPL's [`Type`](Type) to LLVM IR
pub trait HIRTypesLowering<'llvm> {
    type IR;

    /// Lower PPL's [`Type`](Type) to LLVM IR
    fn lower_to_ir(&self, context: &impl Context<'llvm>) -> Self::IR;
}

impl<'llvm> HIRTypesLowering<'llvm> for Type {
    type IR = inkwell::types::AnyTypeEnum<'llvm>;

    fn lower_to_ir(&self, context: &impl Context<'llvm>) -> Self::IR {
        match self {
            Type::Class(ty) => ty.lower_to_ir(context).into(),
            Type::SelfType(_) => unreachable!("Self must not be lowered to IR"),
            Type::Trait(_) => unreachable!("Trait must not be lowered to IR"),
            Type::Function { .. } => unimplemented!("Function type lowering"),
        }
    }
}

impl<'llvm> GlobalHIRLowering<'llvm> for Declaration {
    type IR = ();

    /// Lower global [`Declaration`] to LLVM IR
    fn lower_global_to_ir(&self, context: &mut ModuleContext<'llvm>) -> Self::IR {
        match self {
            Declaration::Variable(var) => {
                var.lower_global_to_ir(context);
            }
            Declaration::Type(ty) => {
                ty.lower_to_ir(context);
            }
            Declaration::Function(f) => {
                if !f.is_generic() {
                    f.lower_global_to_ir(context);
                }
            }
            // Traits have no effect on ir
            Declaration::Trait(_) => (),
        }
    }
}

impl<'llvm, 'm> LocalHIRLowering<'llvm, 'm> for Declaration {
    type IR = ();

    /// Lower local [`Declaration`] to LLVM IR
    fn lower_local_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        match self {
            Declaration::Variable(var) => {
                var.lower_local_to_ir(context);
            }
            Declaration::Type(ty) => {
                ty.lower_to_ir(context);
            }
            Declaration::Function(f) => {
                if !f.is_generic() {
                    f.lower_local_to_ir(context);
                }
            }
            // Traits have no effect on ir
            Declaration::Trait(_) => (),
        }
    }
}

/// Trait for declaring global entries in LLVM IR
trait DeclareGlobal<'llvm> {
    type IR;

    /// Declare global value without defining it
    fn declare_global(&self, context: &mut ModuleContext<'llvm>) -> Self::IR;
}

impl<'llvm> DeclareGlobal<'llvm> for VariableDeclaration {
    type IR = Option<inkwell::values::GlobalValue<'llvm>>;

    /// Declare global variable without defining it
    fn declare_global(&self, context: &mut ModuleContext<'llvm>) -> Self::IR {
        if self.ty().is_none() {
            return None;
        }

        let ty = self.ty().lower_to_ir(context);
        let global = context
            .module
            .add_global(ty.try_into_basic_type().unwrap(), None, &self.name);

        if self.is_immutable() {
            global.set_constant(true);
        }

        Some(global)
    }
}

impl<'llvm> GlobalHIRLowering<'llvm> for Arc<VariableDeclaration> {
    type IR = Option<inkwell::values::GlobalValue<'llvm>>;

    /// Lower global [`VariableDeclaration`] to LLVM IR
    fn lower_global_to_ir(&self, context: &mut ModuleContext<'llvm>) -> Self::IR {
        let global = self.declare_global(context);
        if global.is_none() {
            return None;
        }
        let global = global.unwrap();

        global.set_initializer(
            &self
                .ty()
                .lower_to_ir(context)
                .try_into_basic_type()
                .expect("non-basic type global initializer")
                .const_zero(),
        );

        let initialize = context.module.add_function(
            "initialize",
            context.llvm().void_type().fn_type(&[], false),
            None,
        );
        let mut f_context = FunctionContext::new(context, initialize);
        let value = self.initializer.lower_to_ir(&mut f_context);
        f_context.builder.build_store(
            global.as_pointer_value(),
            value.expect("initializer return None or Void"),
        );

        Some(global)
    }
}

impl<'llvm, 'm> LocalHIRLowering<'llvm, 'm> for Arc<VariableDeclaration> {
    type IR = inkwell::values::PointerValue<'llvm>;

    /// Lower local [`VariableDeclaration`] to LLVM IR
    fn lower_local_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        let ty = self
            .ty()
            .lower_to_ir(context)
            .try_into_basic_type()
            .expect("non-basic type local variable");
        let value = self
            .initializer
            .lower_to_ir(context)
            .expect("initializer return None or Void");
        let alloca = context.builder.build_alloca(ty, &self.name);
        context.builder.build_store(alloca, value);
        context
            .variables
            .insert(self.name.to_string(), alloca.clone());
        alloca
    }
}

impl<'llvm> HIRTypesLowering<'llvm> for TypeDeclaration {
    type IR = inkwell::types::AnyTypeEnum<'llvm>;

    /// Lower [`TypeDeclaration`] to LLVM IR
    fn lower_to_ir(&self, context: &impl Context<'llvm>) -> Self::IR {
        if self.is_none() {
            return context.types().none().into();
        } else if self.is_bool() {
            return context.types().bool().into();
        }

        if self.members.is_empty() {
            return context.types().opaque(&self.name).into();
        }

        if let Some(ty) = context.llvm().get_struct_type(self.name()) {
            return ty.into();
        }

        let ty = context.llvm().opaque_struct_type(self.name());
        ty.set_body(
            self.members
                .iter()
                .filter_map(|m| m.ty.lower_to_ir(context).try_into_basic_type().ok())
                .collect::<Vec<_>>()
                .as_slice(),
            false,
        );
        ty.into()
    }
}

impl<'llvm> DeclareGlobal<'llvm> for FunctionDeclaration {
    type IR = inkwell::values::FunctionValue<'llvm>;

    /// Declare global function without defining it
    fn declare_global(&self, context: &mut ModuleContext<'llvm>) -> Self::IR {
        let ty = match self.ty() {
            Type::Function(f) => {
                let parameters = f
                    .parameters
                    .iter()
                    .filter_map(|p| p.lower_to_ir(context).try_into().ok())
                    .collect::<Vec<BasicMetadataTypeEnum>>();
                let return_type = f.return_type.lower_to_ir(context);
                return_type.fn_type(&parameters, false)
            }
            _ => unreachable!("FunctionDeclaration::ty() returned non-function type"),
        };
        context.module.add_function(self.mangled_name(), ty, None)
    }
}

impl<'llvm> GlobalHIRLowering<'llvm> for FunctionDeclaration {
    type IR = inkwell::values::FunctionValue<'llvm>;

    /// Lower global [`FunctionDeclaration`] to LLVM IR
    fn lower_global_to_ir(&self, context: &mut ModuleContext<'llvm>) -> Self::IR {
        self.declare_global(context)
    }
}

impl<'llvm, 'm> LocalHIRLowering<'llvm, 'm> for FunctionDeclaration {
    type IR = inkwell::values::FunctionValue<'llvm>;

    /// Lower local [`FunctionDeclaration`] to LLVM IR
    fn lower_local_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        // TODO: limit function visibility, capture variables, etc.
        self.declare_global(context.module_context)
    }
}

impl<'llvm> DeclareGlobal<'llvm> for FunctionDefinition {
    type IR = inkwell::values::FunctionValue<'llvm>;

    /// Declare global function without defining it
    fn declare_global(&self, context: &mut ModuleContext<'llvm>) -> Self::IR {
        self.declaration.declare_global(context)
    }
}

impl<'llvm> GlobalHIRLowering<'llvm> for FunctionDefinition {
    type IR = inkwell::values::FunctionValue<'llvm>;

    /// Lower global [`FunctionDefinition`] to LLVM IR
    fn lower_global_to_ir(&self, context: &mut ModuleContext<'llvm>) -> Self::IR {
        let f = self.declaration.lower_global_to_ir(context);

        self.emit_body(context);

        f
    }
}

impl<'llvm, 'm> LocalHIRLowering<'llvm, 'm> for FunctionDefinition {
    type IR = inkwell::values::FunctionValue<'llvm>;

    /// Lower local [`FunctionDefinition`] to LLVM IR
    fn lower_local_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        let f = self.declaration.lower_local_to_ir(context);

        self.emit_body(context.module_context);

        f
    }
}

/// Trait for emitting body of function
trait EmitBody<'llvm> {
    /// Emit body of function
    fn emit_body(&self, context: &mut ModuleContext<'llvm>);
}

impl<'llvm> EmitBody<'llvm> for FunctionDefinition {
    fn emit_body(&self, context: &mut ModuleContext<'llvm>) {
        let f = context
            .functions()
            .get(self.mangled_name())
            .expect("Function was not declared before emitting body");
        if !self.body.is_empty() {
            let mut f_context = FunctionContext::new(context, f);
            for (i, p) in self
                .parameters()
                .filter(|p| !p.name().is_empty() && !p.ty().is_none())
                .enumerate()
            {
                let alloca = f_context.builder.build_alloca(
                    p.ty()
                        .lower_to_ir(&f_context)
                        .try_into_basic_type()
                        .unwrap(),
                    &p.name(),
                );
                f_context
                    .parameters
                    .insert(p.name().to_string(), alloca.clone());
                f_context
                    .builder
                    .build_store(alloca, f.get_nth_param(i as u32).unwrap());
            }
            for stmt in &self.body {
                stmt.lower_local_to_ir(&mut f_context);
            }
        }
    }
}

impl<'llvm> GlobalHIRLowering<'llvm> for Function {
    type IR = inkwell::values::FunctionValue<'llvm>;

    /// Lower global [`Function`] to LLVM IR
    fn lower_global_to_ir(&self, context: &mut ModuleContext<'llvm>) -> Self::IR {
        match self {
            Function::Declaration(decl) => decl.lower_global_to_ir(context),
            Function::Definition(def) => def.lower_global_to_ir(context),
        }
    }
}

impl<'llvm, 'm> LocalHIRLowering<'llvm, 'm> for Function {
    type IR = inkwell::values::FunctionValue<'llvm>;

    /// Lower local [`Function`] to LLVM IR
    fn lower_local_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        match self {
            Function::Declaration(decl) => decl.lower_local_to_ir(context),
            Function::Definition(def) => def.lower_local_to_ir(context),
        }
    }
}

impl<'llvm, 'm> HIRLoweringWithinFunctionContext<'llvm, 'm> for Literal {
    type IR = Option<inkwell::values::BasicValueEnum<'llvm>>;

    /// Lower [`Literal`] to LLVM IR
    fn lower_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        Some(match self {
            Literal::None { .. } => return None,
            Literal::Bool { value, .. } => context
                .types()
                .bool()
                .const_int(*value as u64, false)
                .into(),
            Literal::Integer { value, .. } => {
                if let Some(value) = value.to_i64() {
                    return Some(
                        context
                            .builder
                            .build_call(
                                context.functions().integer_from_i64(),
                                &[context.types().i(64).const_int(value as u64, false).into()],
                                "",
                            )
                            .try_as_basic_value()
                            .left()
                            .unwrap(),
                    );
                }

                let str = context
                    .builder
                    .build_global_string_ptr(&format!("{}", value), "");
                context
                    .builder
                    .build_call(
                        context.functions().integer_from_c_string(),
                        &[str.as_pointer_value().into()],
                        "",
                    )
                    .try_as_basic_value()
                    .left()
                    .unwrap()
            }
            Literal::Rational { value, .. } => {
                let str = context
                    .builder
                    .build_global_string_ptr(&format!("{}", value), "");
                context
                    .builder
                    .build_call(
                        context.functions().rational_from_c_string(),
                        &[str.as_pointer_value().into()],
                        "",
                    )
                    .try_as_basic_value()
                    .left()
                    .unwrap()
            }
            Literal::String { value, .. } => {
                let str = context.builder.build_global_string_ptr(&value, "");
                context
                    .builder
                    .build_call(
                        context.functions().string_from_c_string_and_length(),
                        &[
                            str.as_pointer_value().into(),
                            context
                                .types()
                                .u(64)
                                .const_int(value.len() as u64, false)
                                .into(),
                        ],
                        "",
                    )
                    .try_as_basic_value()
                    .left()
                    .unwrap()
            }
        })
    }
}

impl<'llvm, 'm> HIRLoweringWithinFunctionContext<'llvm, 'm> for VariableReference {
    type IR = Option<inkwell::values::PointerValue<'llvm>>;

    /// Lower [`VariableReference`] to LLVM IR
    fn lower_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        if self.variable.ty().is_none() {
            return None;
        }

        if let Some(var) = context.get_variable(&self.variable) {
            return Some(var);
        }

        match &self.variable {
            ParameterOrVariable::Parameter(p) => panic!("Parameter {:?} not found", p.name()),
            ParameterOrVariable::Variable(var) => Some(
                var.declare_global(context.module_context)
                    .unwrap()
                    .as_pointer_value(),
            ),
        }
    }
}

impl<'llvm, 'm> HIRLoweringWithinFunctionContext<'llvm, 'm> for Call {
    type IR = inkwell::values::CallSiteValue<'llvm>;

    /// Lower [`Call`] to LLVM IR
    fn lower_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        let function = context
            .functions()
            .get(self.function.mangled_name())
            .unwrap_or_else(|| {
                if self.generic.is_none() {
                    self.function
                        .declaration()
                        .declare_global(context.module_context)
                } else {
                    self.function.lower_local_to_ir(context)
                }
            });

        let arguments = self
            .args
            .iter()
            .filter_map(|arg| arg.lower_to_ir(context).map(|x| x.into()))
            .collect::<Vec<BasicMetadataValueEnum>>();

        context.builder.build_call(function, &arguments, "")
    }
}

impl<'llvm, 'm> HIRLoweringWithinFunctionContext<'llvm, 'm> for Constructor {
    type IR = inkwell::values::PointerValue<'llvm>;

    fn lower_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        let ty = self
            .ty
            .referenced_type
            .lower_to_ir(context)
            .into_struct_type();
        let alloca = context.builder.build_alloca(ty, "");

        for init in self.initializers.iter().filter(|i| !i.value.ty().is_none()) {
            let field = context
                .builder
                .build_struct_gep(
                    alloca,
                    init.index as u32,
                    format!("{}.{}", self.ty.referenced_type.name(), init.member.name()).as_str(),
                )
                .unwrap();
            let value = init.value.lower_to_ir(context);
            context.builder.build_store(field, value.unwrap());
        }

        alloca
    }
}

/// Trait for [`Expression`] to lower HIR to LLVM IR without loading references
trait HIRExpressionLoweringWithoutLoad<'llvm, 'm> {
    /// Lower [`Expression`] to LLVM IR without loading variables
    fn lower_to_ir_without_load(
        &self,
        context: &mut FunctionContext<'llvm, 'm>,
    ) -> Option<inkwell::values::BasicValueEnum<'llvm>>;
}

impl<'llvm, 'm> HIRExpressionLoweringWithoutLoad<'llvm, 'm> for MemberReference {
    fn lower_to_ir_without_load(
        &self,
        context: &mut FunctionContext<'llvm, 'm>,
    ) -> Option<inkwell::values::BasicValueEnum<'llvm>> {
        let base = self.base.lower_to_ir_without_load(context);
        if base.is_none() {
            return None;
        }

        let base = base.unwrap().into_pointer_value();
        Some(
            context
                .builder
                .build_struct_gep(base, self.index as u32, self.member.name())
                .unwrap()
                .into(),
        )
    }
}

impl<'llvm, 'm> HIRExpressionLoweringWithoutLoad<'llvm, 'm> for Expression {
    /// Lower [`Expression`] to LLVM IR without loading variables
    fn lower_to_ir_without_load(
        &self,
        context: &mut FunctionContext<'llvm, 'm>,
    ) -> Option<inkwell::values::BasicValueEnum<'llvm>> {
        match self {
            Expression::VariableReference(var) => {
                let var = var.lower_to_ir(context);
                if var.is_none() {
                    return None;
                }

                Some(var.unwrap().into())
            }

            Expression::Literal(l) => l.lower_to_ir(context),
            Expression::Call(call) => call.lower_to_ir(context).try_as_basic_value().left(),
            Expression::TypeReference(ty) => unimplemented!("TypeReference as expresssion"),
            Expression::MemberReference(m) => m.lower_to_ir_without_load(context),
            Expression::Constructor(c) => Some(c.lower_to_ir(context).into()),
        }
    }
}

impl<'llvm, 'm> HIRLoweringWithinFunctionContext<'llvm, 'm> for Expression {
    type IR = Option<inkwell::values::BasicValueEnum<'llvm>>;

    /// Lower [`Expression`] to LLVM IR with loading references
    fn lower_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        let value = self.lower_to_ir_without_load(context);
        if value.is_none() {
            return None;
        }

        let value = value.unwrap();
        if !value.is_pointer_value() {
            return Some(value);
        }

        let ptr = value.into_pointer_value();
        let elem = ptr.get_type().get_element_type();
        if elem.is_struct_type() && elem.into_struct_type().is_opaque() {
            return Some(ptr.into());
        }
        Some(context.builder.build_load(ptr, ""))
    }
}

impl<'llvm, 'm> HIRLoweringWithinFunctionContext<'llvm, 'm> for Assignment {
    type IR = Option<inkwell::values::InstructionValue<'llvm>>;

    /// Lower [`Assignment`] to LLVM IR
    fn lower_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        let target = self.target.lower_to_ir_without_load(context);
        let value = self.value.lower_to_ir(context);

        if target.is_none() {
            return None;
        }

        Some(context.builder.build_store(
            target.unwrap().into_pointer_value(),
            value.expect("Assigning none"),
        ))
    }
}

impl<'llvm> GlobalHIRLowering<'llvm> for Statement {
    type IR = ();

    /// Lower global [`Statement`] to LLVM IR
    fn lower_global_to_ir(&self, context: &mut ModuleContext<'llvm>) -> Self::IR {
        match self {
            Statement::Declaration(d) => d.lower_global_to_ir(context),

            Statement::Assignment(_)
            | Statement::If(_)
            | Statement::Loop(_)
            | Statement::While(_) => {
                let function = context.module.add_function(
                    "execute",
                    context.llvm().void_type().fn_type(&[], false),
                    None,
                );

                let mut context = FunctionContext::new(context, function);
                self.lower_local_to_ir(&mut context);
            }

            Statement::Expression(expr) => {
                let function = context.module.add_function(
                    "evaluate",
                    expr.ty().lower_to_ir(context).fn_type(&[], false),
                    None,
                );

                let mut context = FunctionContext::new(context, function);

                let value = expr.lower_to_ir(&mut context);
                if let Some(value) = value {
                    context.builder.build_return(Some(&value));
                } else {
                    context.builder.build_return(None);
                }

                function.verify(true);
            }
            Statement::Return(_) => unreachable!("Return statement is not allowed in global scope"),
            Statement::Use(_) => {
                // Use statements are skipped
            }
        };
    }
}

impl<'llvm, 'm> LocalHIRLowering<'llvm, 'm> for Statement {
    type IR = ();

    /// Lower local [`Statement`] to LLVM IR
    fn lower_local_to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        match self {
            Statement::Declaration(d) => d.lower_local_to_ir(context),
            Statement::Assignment(a) => {
                a.lower_to_ir(context);
            }
            Statement::Expression(expr) => {
                expr.lower_to_ir(context);
            }
            Statement::Return(ret) => {
                ret.lower_to_ir(context);
            }
            Statement::If(if_stmt) => {
                if_stmt.lower_to_ir(context);
            }
            Statement::Loop(loop_stmt) => loop_stmt.lower_to_ir(context),
            Statement::While(while_stmt) => while_stmt.lower_to_ir(context),
            Statement::Use(_) => {
                // Use statements are skipped
            }
        };
    }
}

impl HIRLoweringWithinFunctionContext<'_, '_> for Return {
    type IR = ();

    /// Lower [`Return`] to LLVM IR
    fn lower_to_ir(&self, context: &mut FunctionContext) -> Self::IR {
        let value = self.value.as_ref().map(|expr| expr.lower_to_ir(context));
        if let Some(Some(value)) = value {
            context.builder.build_return(Some(&value));
        } else {
            context.builder.build_return(None);
        }
    }
}

impl HIRLoweringWithinFunctionContext<'_, '_> for If {
    type IR = ();

    /// Lower [`If`] to LLVM IR
    fn lower_to_ir(&self, context: &mut FunctionContext) -> Self::IR {
        let entry_block = context.builder.get_insert_block().unwrap();

        let merge_block = context.llvm().append_basic_block(context.function, "");

        let if_true = context.build_block("if.body", &self.body, Some(merge_block));
        if_true.move_after(entry_block).unwrap();

        let last_block = if self.else_block.is_empty() {
            merge_block.clone()
        } else {
            let else_block = context.build_block("else", &self.else_block, Some(merge_block));
            else_block.move_after(if_true).unwrap();
            else_block
        };

        let else_if_conditions = self
            .else_ifs
            .iter()
            .map(|_| {
                context
                    .llvm()
                    .append_basic_block(context.function, "else_if.condition")
            })
            .collect::<Vec<_>>();
        let else_if_bodies = self
            .else_ifs
            .iter()
            .map(|else_if| context.build_block("else_if.body", &else_if.body, Some(merge_block)))
            .collect::<Vec<_>>();
        for (i, else_if) in self.else_ifs.iter().enumerate() {
            context.builder.position_at_end(else_if_conditions[i]);
            let condition = else_if
                .condition
                .lower_to_ir(context)
                .unwrap()
                .into_int_value();
            if i + 1 < else_if_conditions.len() {
                context.builder.build_conditional_branch(
                    condition,
                    else_if_bodies[i],
                    else_if_conditions[i + 1],
                );
            } else {
                context
                    .builder
                    .build_conditional_branch(condition, else_if_bodies[i], last_block);
            }
            else_if_bodies[i].move_after(else_if_conditions[i]).unwrap()
        }

        let condition_block = context
            .llvm()
            .append_basic_block(context.function, "if.condition");
        condition_block.move_after(entry_block).unwrap();

        context.builder.position_at_end(entry_block);
        context.builder.build_unconditional_branch(condition_block);

        context.builder.position_at_end(condition_block);
        let condition = self
            .condition
            .lower_to_ir(context)
            .unwrap()
            .into_int_value();
        if let Some(else_if) = else_if_conditions.first() {
            context
                .builder
                .build_conditional_branch(condition, if_true, *else_if);
        } else {
            context
                .builder
                .build_conditional_branch(condition, if_true, last_block);
        }

        context.builder.position_at_end(merge_block);
    }
}

impl HIRLoweringWithinFunctionContext<'_, '_> for Loop {
    type IR = ();

    /// Lower [`Loop`] to LLVM IR
    fn lower_to_ir(&self, context: &mut FunctionContext) -> Self::IR {
        let loop_block = context.build_block("loop", &self.body, None);

        context.builder.build_unconditional_branch(loop_block);

        if loop_block.get_terminator().is_none() {
            context.builder.position_at_end(loop_block);
            context.builder.build_unconditional_branch(loop_block);
        }
    }
}

impl HIRLoweringWithinFunctionContext<'_, '_> for While {
    type IR = ();

    /// Lower [`While`] to LLVM IR
    fn lower_to_ir(&self, context: &mut FunctionContext) -> Self::IR {
        let condition_block = context
            .llvm()
            .append_basic_block(context.function, "while.condition");

        context.builder.build_unconditional_branch(condition_block);

        let loop_block = context.build_block("while.body", &self.body, Some(condition_block));

        let merge_block = context.llvm().append_basic_block(context.function, "");

        context.builder.position_at_end(condition_block);
        let condition = self
            .condition
            .lower_to_ir(context)
            .unwrap()
            .into_int_value();
        context
            .builder
            .build_conditional_branch(condition, loop_block, merge_block);

        context.builder.position_at_end(merge_block);
    }
}

/// Trait for lowering HIR Module to LLVM IR
pub trait HIRModuleLowering<'llvm> {
    /// Lower [`Module`] to LLVM IR
    fn lower_to_ir(&self, llvm: &'llvm inkwell::context::Context)
        -> inkwell::module::Module<'llvm>;
}

impl<'llvm> HIRModuleLowering<'llvm> for Module {
    /// Lower [`Module`] to LLVM IR
    fn lower_to_ir(
        &self,
        llvm: &'llvm inkwell::context::Context,
    ) -> inkwell::module::Module<'llvm> {
        let module = llvm.create_module(self.name());
        module.set_source_file_name(&self.filename);

        let mut context = ModuleContext::new(module);

        for statement in &self.statements {
            statement.lower_global_to_ir(&mut context);
        }

        context.module
    }
}
