use inkwell::module::Linkage;
use inkwell::types::BasicMetadataTypeEnum;

use inkwell::values::BasicMetadataValueEnum;
use inkwell::values::CallSiteValue;
use log::trace;

use super::inkwell::*;
use crate::compilation;
use crate::hir::*;
use crate::ir::Initializer;
use crate::mutability::Mutable;
use crate::named::Named;
use crate::syntax::Ranged;
use crate::DataHolder;

use super::Context;
use super::FunctionContext;
use super::ModuleContext;

/// Trait for lowering to IR within some context
pub trait ToIR<'llvm, C: Context<'llvm>> {
    type IR;

    /// Lower HIR to IR within some context
    fn to_ir(&self, context: &mut C) -> Self::IR;
}

impl<'llvm, C: Context<'llvm>> ToIR<'llvm, C> for Type {
    type IR = inkwell::types::AnyTypeEnum<'llvm>;

    fn to_ir(&self, context: &mut C) -> Self::IR {
        trace!(target: "to_ir", "{self}");

        match self {
            Type::Class(ty) => ty.read().unwrap().to_ir(context).into(),
            Type::SelfType(_) => unreachable!("Self must not be lowered to IR"),
            Type::Trait(_) => unreachable!("Trait must not be lowered to IR"),
            Type::Generic(_) => unreachable!("Generic must not be lowered to IR"),
            Type::Function { .. } => unimplemented!("Function type lowering"),
            Type::Unknown => unreachable!("Lowering not-inferred type"),
        }
    }
}

impl<'llvm> ToIR<'llvm, ModuleContext<'llvm, '_>> for Declaration {
    type IR = ();

    /// Lower global [`Declaration`] to LLVM IR
    fn to_ir(&self, context: &mut ModuleContext<'llvm, '_>) -> Self::IR {
        trace!(target: "to_ir", "{self}");

        match self {
            Declaration::Variable(var) => {
                var.to_ir(context);
            }
            Declaration::Type(ty) => {
                let ty = ty.read().unwrap();
                if !ty.is_generic() {
                    ty.to_ir(context);
                }
            }
            Declaration::Function(f) => {
                let f = f.read().unwrap();
                if !f.is_generic() {
                    f.to_ir(context);
                }
            }
            // Traits have no effect on ir
            Declaration::Trait(_) => (),
        }
    }
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm, '_>> for Declaration {
    type IR = ();

    /// Lower local [`Declaration`] to LLVM IR
    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm, '_>) -> Self::IR {
        trace!(target: "to_ir", "{self}");

        match self {
            Declaration::Variable(var) => {
                var.to_ir(context);
            }
            Declaration::Type(ty) => {
                let ty = ty.read().unwrap();
                if !ty.is_generic() {
                    ty.to_ir(context);
                }
            }
            Declaration::Function(f) => {
                let f = f.read().unwrap();
                if !f.is_generic() {
                    f.to_ir(context);
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
    fn declare_global(&self, context: &mut ModuleContext<'llvm, '_>) -> Self::IR;
}

impl<'llvm> DeclareGlobal<'llvm> for VariableData {
    type IR = Option<inkwell::values::GlobalValue<'llvm>>;

    /// Declare global variable without defining it
    fn declare_global(&self, context: &mut ModuleContext<'llvm, '_>) -> Self::IR {
        trace!(target: "declare_global", "{self}");

        if self.ty().is_none() {
            return None;
        }

        let ty = self.ty().to_ir(context);
        let global = context
            .module
            .add_global(ty.try_into_basic_type().unwrap(), None, &self.name);

        // Type variables should be local to module
        if self.name.starts_with("Type") {
            global.set_linkage(Linkage::Private)
        }

        if self.is_immutable() {
            global.set_constant(true);
        }

        Some(global)
    }
}

impl<'llvm> ToIR<'llvm, ModuleContext<'llvm, '_>> for Variable {
    type IR = Option<inkwell::values::GlobalValue<'llvm>>;

    /// Lower global [`VariableDeclaration`] to LLVM IR
    fn to_ir(&self, context: &mut ModuleContext<'llvm, '_>) -> Self::IR {
        trace!(target: "to_ir", "{self}");

        let global = self.read().unwrap().declare_global(context);

        let initialize = context.module.add_function(
            "initialize",
            context.llvm().void_type().fn_type(&[], false),
            Some(Linkage::Private),
        );
        let at = self.read().unwrap().initializer.as_ref().unwrap().start();
        context.initializers.push(Initializer {
            function: initialize,
            at,
        });
        let mut f_context = FunctionContext::new(context, initialize, at);

        let value = self
            .read()
            .unwrap()
            .initializer
            .as_ref()
            .expect("Currently all variables have initializers")
            .to_ir(&mut f_context);

        if global.is_none() {
            return None;
        }
        let global = global.unwrap();

        // TODO: check that we can initialize without function
        global.set_constant(false);

        global.set_initializer(
            &self
                .ty()
                .to_ir(f_context.module_context)
                .try_into_basic_type()
                .expect("non-basic type global initializer")
                .const_zero(),
        );

        f_context
            .builder
            .build_store(
                global.as_pointer_value(),
                value.expect("initializer return None or Void"),
            )
            .unwrap();

        Some(global)
    }
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm, '_>> for Variable {
    type IR = Option<inkwell::values::PointerValue<'llvm>>;

    /// Lower local [`VariableDeclaration`] to LLVM IR
    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm, '_>) -> Self::IR {
        trace!(target: "to_ir", "{self}");

        let value = self
            .read()
            .unwrap()
            .initializer
            .as_ref()
            .expect("Currently all variables have initializers")
            .to_ir(context);

        if self.ty().is_none() {
            return None;
        }

        let ty = self
            .ty()
            .to_ir(context)
            .try_into_basic_type()
            .expect("non-basic type local variable");

        let alloca = context.builder.build_alloca(ty, &self.name()).unwrap();
        context.builder.build_store(alloca, value.unwrap()).unwrap();
        context
            .variables
            .insert(self.name().to_string(), alloca.clone());
        Some(alloca)
    }
}

impl<'llvm, C: Context<'llvm>> ToIR<'llvm, C> for ClassData {
    type IR = inkwell::types::AnyTypeEnum<'llvm>;

    /// Lower [`TypeDeclaration`] to LLVM IR
    fn to_ir(&self, context: &mut C) -> Self::IR {
        trace!(target: "to_ir", "{self}");

        if self.is_none() || self.is_any_reference() && self.generics()[0].is_none() {
            return context.types().none().into();
        } else if self.is_bool() {
            return context.types().bool().into();
        } else if self.is_i32() {
            return context.types().i32().into();
        } else if self.is_f64() {
            return context.types().f64().into();
        }

        if self.members.is_empty() {
            return context.types().opaque(&self.basename).into();
        }

        if let Some(ty) = context.llvm().get_struct_type(&self.name()) {
            return ty.into();
        }

        let ty = context.llvm().opaque_struct_type(&self.name());
        ty.set_body(
            self.members
                .iter()
                .filter_map(|m| m.ty().to_ir(context).try_into_basic_type().ok())
                .collect::<Vec<_>>()
                .as_slice(),
            false,
        );
        ty.into()
    }
}

impl<'llvm> DeclareGlobal<'llvm> for FunctionData {
    type IR = inkwell::values::FunctionValue<'llvm>;

    /// Declare global function without defining it
    fn declare_global(&self, context: &mut ModuleContext<'llvm, '_>) -> Self::IR {
        trace!(target: "declare_global", "{self}");

        let ty = match self.ty() {
            Type::Function(f) => {
                let parameters = f
                    .parameters
                    .iter()
                    .filter_map(|p| p.to_ir(context).try_into().ok())
                    .collect::<Vec<BasicMetadataTypeEnum>>();
                let return_type = f.return_type.to_ir(context);
                return_type.fn_type(&parameters, false)
            }
            _ => unreachable!("FunctionDeclaration::ty() returned non-function type"),
        };
        context.module.add_function(
            &self.mangled_name(),
            ty,
            // Private linkage for monomorphized generic functions or functions from traits
            if self.mangled_name.is_none() && (!self.generic_types.is_empty() || self.tr.is_some())
            {
                Some(Linkage::Private)
            } else {
                None
            },
        )
    }
}

impl<'llvm> ToIR<'llvm, ModuleContext<'llvm, '_>> for FunctionData {
    type IR = inkwell::values::FunctionValue<'llvm>;

    /// Lower global [`FunctionDeclaration`] to LLVM IR
    fn to_ir(&self, context: &mut ModuleContext<'llvm, '_>) -> Self::IR {
        trace!(target: "to_ir", "{self}");

        let f = self.declare_global(context);

        if self.module == context.compilation_module
            || !self.generic_types.is_empty()
            || self.tr.is_some()
        {
            self.emit_body(context);
        }

        f
    }
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm, '_>> for FunctionData {
    type IR = inkwell::values::FunctionValue<'llvm>;

    /// Lower local [`FunctionDeclaration`] to LLVM IR
    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm, '_>) -> Self::IR {
        trace!(target: "to_ir", "{self}");

        // TODO: limit function visibility, capture variables, etc.
        let f = self.declare_global(context.module_context);

        if self.module == context.module_context.compilation_module
            || !self.generic_types.is_empty()
            || self.tr.is_some()
        {
            self.emit_body(context.module_context);
        }

        f
    }
}

/// Trait for emitting body of function
trait EmitBody<'llvm> {
    /// Emit body of function
    fn emit_body(&self, context: &mut ModuleContext<'llvm, '_>);
}

impl<'llvm> EmitBody<'llvm> for FunctionData {
    fn emit_body(&self, context: &mut ModuleContext<'llvm, '_>) {
        trace!(target: "emit_body", "{self}");

        let f = context
            .functions()
            .get(&self.mangled_name())
            .expect("Function was not declared before emitting body");
        if !self.body.is_empty() {
            let mut f_context = FunctionContext::new(context, f, self.start());
            for (i, p) in self
                .parameters()
                .filter(|p| !p.name().is_empty() && !p.ty().is_none())
                .enumerate()
            {
                let ty = p.ty().to_ir(&mut f_context).try_into_basic_type().unwrap();
                let alloca = f_context.builder.build_alloca(ty, &p.name()).unwrap();
                f_context
                    .parameters
                    .insert(p.name().to_string(), alloca.clone());
                f_context
                    .builder
                    .build_store(alloca, f.get_nth_param(i as u32).unwrap())
                    .unwrap();
            }
            for stmt in &self.body {
                stmt.to_ir(&mut f_context);
            }
        }
    }
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm, '_>> for Literal {
    type IR = Option<inkwell::values::BasicValueEnum<'llvm>>;

    /// Lower [`Literal`] to LLVM IR
    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm, '_>) -> Self::IR {
        trace!(target: "to_ir", "{self}");

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
                            .unwrap()
                            .try_as_basic_value()
                            .left()
                            .unwrap(),
                    );
                }

                let str = context
                    .builder
                    .build_global_string_ptr(&format!("{}", value), "")
                    .unwrap();
                context
                    .builder
                    .build_call(
                        context.functions().integer_from_c_string(),
                        &[str.as_pointer_value().into()],
                        "",
                    )
                    .unwrap()
                    .try_as_basic_value()
                    .left()
                    .unwrap()
            }
            Literal::Rational { value, .. } => {
                let str = context
                    .builder
                    .build_global_string_ptr(&format!("{}", value), "")
                    .unwrap();
                context
                    .builder
                    .build_call(
                        context.functions().rational_from_c_string(),
                        &[str.as_pointer_value().into()],
                        "",
                    )
                    .unwrap()
                    .try_as_basic_value()
                    .left()
                    .unwrap()
            }
            Literal::String { value, .. } => {
                let value = unescaper::unescape(&value).unwrap_or_else(|_| value.clone());
                let str = context.builder.build_global_string_ptr(&value, "").unwrap();
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
                    .unwrap()
                    .try_as_basic_value()
                    .left()
                    .unwrap()
            }
        })
    }
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm, '_>> for VariableReference {
    type IR = Option<inkwell::values::PointerValue<'llvm>>;

    /// Lower [`VariableReference`] to LLVM IR
    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm, '_>) -> Self::IR {
        trace!(target: "to_ir", "{self}");

        if self.variable.ty().is_none() {
            return None;
        }

        if let Some(var) = context.get_variable(&self.variable) {
            return Some(var);
        }

        match &self.variable {
            ParameterOrVariable::Parameter(p) => panic!("Parameter {:?} not found", p.name()),
            ParameterOrVariable::Variable(var) => Some(
                var.read()
                    .unwrap()
                    .declare_global(context.module_context)
                    .unwrap()
                    .as_pointer_value(),
            ),
        }
    }
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm, '_>> for Call {
    type IR = inkwell::values::CallSiteValue<'llvm>;

    /// Lower [`Call`] to LLVM IR
    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm, '_>) -> Self::IR {
        trace!(target: "to_ir", "{self}");

        let function = context
            .functions()
            .get(&self.function.read().unwrap().mangled_name())
            .unwrap_or_else(|| {
                if self.generic.is_none() {
                    self.function
                        .read()
                        .unwrap()
                        .declare_global(context.module_context)
                } else {
                    debug_assert!(
                        self.function.read().unwrap().mangled_name.is_some()
                            || self.function.read().unwrap().is_definition(),
                        "Generic function {} {} has no definition, inside this call: {}",
                        self.function.read().unwrap(),
                        self.function
                            .read()
                            .unwrap()
                            .tr
                            .as_ref()
                            .map(|tr| format!("from trait {}", tr.name()))
                            .unwrap_or_default(),
                        self
                    );
                    self.function.read().unwrap().to_ir(context)
                }
            });

        let arguments = self
            .args
            .iter()
            .zip(self.function.read().unwrap().parameters().map(|p| p.ty()))
            .filter_map(|(arg, p)| {
                if p.is_any_reference() {
                    arg.lower_to_ir_without_load(context)
                } else {
                    arg.to_ir(context)
                }
                .map(|x| x.into())
            })
            .collect::<Vec<BasicMetadataValueEnum>>();

        context
            .builder
            .build_call(function, &arguments, "")
            .unwrap()
    }
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm, '_>> for Constructor {
    type IR = inkwell::values::PointerValue<'llvm>;

    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm, '_>) -> Self::IR {
        trace!(target: "to_ir", "{self}");

        let ty = self
            .ty
            .referenced_type
            .to_ir(context)
            .try_into_basic_type()
            .expect("non-basic type constructor");
        let alloca = context.builder.build_alloca(ty, "").unwrap();

        for init in self.initializers.iter().filter(|i| !i.value.ty().is_none()) {
            let field = context
                .builder
                .build_struct_gep(
                    ty,
                    alloca,
                    init.index as u32,
                    format!("{}.{}", self.ty.referenced_type.name(), init.member.name()).as_str(),
                )
                .unwrap();
            let value = init.value.to_ir(context);
            context.builder.build_store(field, value.unwrap()).unwrap();
        }

        alloca
    }
}

/// Trait for [`Expression`] to lower HIR to LLVM IR without loading references
trait HIRExpressionLoweringWithoutLoad<'llvm, 'm> {
    /// Lower [`Expression`] to LLVM IR without loading variables
    fn lower_to_ir_without_load(
        &self,
        context: &mut FunctionContext<'llvm, 'm, '_>,
    ) -> Option<inkwell::values::BasicValueEnum<'llvm>>;
}

impl<'llvm, 'm> HIRExpressionLoweringWithoutLoad<'llvm, 'm> for MemberReference {
    fn lower_to_ir_without_load(
        &self,
        context: &mut FunctionContext<'llvm, 'm, '_>,
    ) -> Option<inkwell::values::BasicValueEnum<'llvm>> {
        trace!(target: "lower_to_ir_without_load", "{self}");

        let base = self.base.lower_to_ir_without_load(context);
        if base.is_none() {
            return None;
        }

        let base = base.unwrap().into_pointer_value();
        let ty = self.base.ty().to_ir(context).try_into_basic_type().unwrap();
        Some(
            context
                .builder
                .build_struct_gep(ty, base, self.index as u32, &self.member.name())
                .unwrap()
                .into(),
        )
    }
}

impl<'llvm, 'm> HIRExpressionLoweringWithoutLoad<'llvm, 'm> for ImplicitConversion {
    fn lower_to_ir_without_load(
        &self,
        context: &mut FunctionContext<'llvm, 'm, '_>,
    ) -> Option<inkwell::values::BasicValueEnum<'llvm>> {
        trace!(target: "lower_to_ir_without_load", "{self}");

        use ImplicitConversionKind::*;
        match self.kind {
            Reference => self.expression.lower_to_ir_without_load(context),
            Dereference => self.expression.to_ir(context),
            Copy => self.expression.to_ir(context),
        }
    }
}

impl<'llvm, 'm> HIRExpressionLoweringWithoutLoad<'llvm, 'm> for Expression {
    /// Lower [`Expression`] to LLVM IR without loading variables
    fn lower_to_ir_without_load(
        &self,
        context: &mut FunctionContext<'llvm, 'm, '_>,
    ) -> Option<inkwell::values::BasicValueEnum<'llvm>> {
        trace!(target: "lower_to_ir_without_load", "{self}");

        context.set_debug_location(self.start());

        match self {
            Expression::VariableReference(var) => var.to_ir(context).map(Into::into),
            Expression::Literal(l) => l.to_ir(context),
            Expression::Call(call) => call.to_ir(context).try_as_basic_value().left(),
            Expression::TypeReference(_) => {
                unreachable!("TypeReference should be converted to constructors")
            }
            Expression::MemberReference(m) => m.lower_to_ir_without_load(context),
            Expression::Constructor(c) => Some(c.to_ir(context).into()),
            Expression::ImplicitConversion(i) => i.lower_to_ir_without_load(context),
        }
    }
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm, '_>> for Expression {
    type IR = Option<inkwell::values::BasicValueEnum<'llvm>>;

    /// Lower [`Expression`] to LLVM IR with loading references
    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm, '_>) -> Self::IR {
        trace!(target: "to_ir", "{self}");

        let value = self.lower_to_ir_without_load(context);
        if value.is_none() {
            return None;
        }

        let value = value.unwrap();
        if !value.is_pointer_value() {
            return Some(value);
        }

        let ptr = value.into_pointer_value();
        match self.ty() {
            Type::Class(cl) => {
                let cl = cl.read().unwrap();
                // FIXME: this is very error prone
                if cl.is_opaque()
                    && !(cl.is_none()
                        || cl.is_bool()
                        || cl.is_i32()
                        || cl.is_f64()
                        || self.is_reference())
                {
                    return Some(ptr.into());
                }
                let ty = cl.to_ir(context).try_into_basic_type().unwrap();
                return Some(context.builder.build_load(ty, ptr, "").unwrap());
            }
            ty if ty.is_generic() => unreachable!("Loading reference of generic type `{ty}`"),
            ty => unimplemented!("Load reference of type `{ty}`"),
        };
    }
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm, '_>> for Assignment {
    type IR = Option<inkwell::values::InstructionValue<'llvm>>;

    /// Lower [`Assignment`] to LLVM IR
    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm, '_>) -> Self::IR {
        trace!(target: "to_ir", "{self}");

        let target = if self.target.ty().is_any_reference() {
            self.target.to_ir(context)
        } else {
            self.target.lower_to_ir_without_load(context)
        };
        let value = self.value.to_ir(context);

        if target.is_none() {
            return None;
        }

        Some(
            context
                .builder
                .build_store(
                    target.unwrap().into_pointer_value(),
                    value.expect("Assigning none"),
                )
                .unwrap(),
        )
    }
}

impl<'llvm> ToIR<'llvm, ModuleContext<'llvm, '_>> for Statement {
    type IR = ();

    /// Lower global [`Statement`] to LLVM IR
    fn to_ir(&self, context: &mut ModuleContext<'llvm, '_>) -> Self::IR {
        trace!(target: "to_ir", "{self}");

        match self {
            Statement::Block(b) => {
                for stmt in &b.statements {
                    stmt.to_ir(context);
                }
            }
            Statement::Declaration(d) => d.to_ir(context),

            Statement::Assignment(_)
            | Statement::If(_)
            | Statement::Loop(_)
            | Statement::While(_) => {
                let function = context.module.add_function(
                    "execute",
                    context.types().none().fn_type(&[], false),
                    None,
                );

                let mut context = FunctionContext::new(context, function, self.start());
                self.to_ir(&mut context);
            }

            Statement::Expression(expr) => {
                let ty = expr.ty().to_ir(context).fn_type(&[], false);
                let function = context.module.add_function("execute", ty, None);

                let mut context = FunctionContext::new(context, function, self.start());

                let value = expr.to_ir(&mut context);
                context.load_return_value_and_branch(value);
            }
            Statement::Return(_) => unreachable!("Return statement is not allowed in global scope"),
            Statement::Use(_) => {
                // Use statements are skipped
            }
        };
    }
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm, '_>> for Statement {
    type IR = ();

    /// Lower local [`Statement`] to LLVM IR
    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm, '_>) -> Self::IR {
        trace!(target: "to_ir", "{self}");

        match self {
            Statement::Block(b) => {
                for stmt in &b.statements {
                    stmt.to_ir(context);
                }
            }
            Statement::Declaration(d) => d.to_ir(context),
            Statement::Assignment(a) => {
                a.to_ir(context);
            }
            Statement::Expression(expr) => {
                expr.to_ir(context);
            }
            Statement::Return(ret) => {
                ret.to_ir(context);
            }
            Statement::If(if_stmt) => {
                if_stmt.to_ir(context);
            }
            Statement::Loop(loop_stmt) => loop_stmt.to_ir(context),
            Statement::While(while_stmt) => while_stmt.to_ir(context),
            Statement::Use(_) => {
                // Use statements are skipped
            }
        };
    }
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm, '_>> for Return {
    type IR = ();

    /// Lower [`Return`] to LLVM IR
    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm, '_>) -> Self::IR {
        trace!(target: "to_ir", "{self}");

        let value = self.value().map(|expr| expr.to_ir(context)).flatten();
        context.load_return_value_and_branch(value);
    }
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm, '_>> for If {
    type IR = ();

    /// Lower [`If`] to LLVM IR
    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm, '_>) -> Self::IR {
        trace!(target: "to_ir", "{self}");

        let entry_block = context.builder.get_insert_block().unwrap();

        let merge_block = context.llvm().append_basic_block(context.function, "");

        let if_true = context.build_block("if.body", &self.body, Some(merge_block));
        if_true.move_after(entry_block).unwrap();

        let last_block = if self.else_block.is_none() {
            merge_block.clone()
        } else {
            let else_block = context.build_block(
                "else",
                &self.else_block.as_ref().unwrap().body,
                Some(merge_block),
            );
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
            let condition = else_if.condition.to_ir(context).unwrap().into_int_value();
            if i + 1 < else_if_conditions.len() {
                context
                    .builder
                    .build_conditional_branch(
                        condition,
                        else_if_bodies[i],
                        else_if_conditions[i + 1],
                    )
                    .unwrap();
            } else {
                context
                    .builder
                    .build_conditional_branch(condition, else_if_bodies[i], last_block)
                    .unwrap();
            }
            else_if_bodies[i].move_after(else_if_conditions[i]).unwrap()
        }

        let condition_block = context
            .llvm()
            .append_basic_block(context.function, "if.condition");
        condition_block.move_after(entry_block).unwrap();

        context.builder.position_at_end(entry_block);
        context
            .builder
            .build_unconditional_branch(condition_block)
            .unwrap();

        context.builder.position_at_end(condition_block);
        let condition = self.condition.to_ir(context).unwrap().into_int_value();
        if let Some(else_if) = else_if_conditions.first() {
            context
                .builder
                .build_conditional_branch(condition, if_true, *else_if)
                .unwrap();
        } else {
            context
                .builder
                .build_conditional_branch(condition, if_true, last_block)
                .unwrap();
        }

        context.builder.position_at_end(merge_block);
    }
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm, '_>> for Loop {
    type IR = ();

    /// Lower [`Loop`] to LLVM IR
    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm, '_>) -> Self::IR {
        trace!(target: "to_ir", "{self}");

        let loop_block = context.build_block("loop", &self.body, None);

        context
            .builder
            .build_unconditional_branch(loop_block)
            .unwrap();

        if loop_block.get_terminator().is_none() {
            context.builder.position_at_end(loop_block);
            context
                .builder
                .build_unconditional_branch(loop_block)
                .unwrap();
        }
    }
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm, '_>> for While {
    type IR = ();

    /// Lower [`While`] to LLVM IR
    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm, '_>) -> Self::IR {
        trace!(target: "to_ir", "{self}");

        let condition_block = context
            .llvm()
            .append_basic_block(context.function, "while.condition");

        context
            .builder
            .build_unconditional_branch(condition_block)
            .unwrap();

        let loop_block = context.build_block("while.body", &self.body, Some(condition_block));

        let merge_block = context.llvm().append_basic_block(context.function, "");

        context.builder.position_at_end(condition_block);
        let condition = self.condition.to_ir(context).unwrap().into_int_value();
        context
            .builder
            .build_conditional_branch(condition, loop_block, merge_block)
            .unwrap();

        context.builder.position_at_end(merge_block);
    }
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm, '_>> for Initializer<'llvm> {
    type IR = CallSiteValue<'llvm>;

    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm, '_>) -> Self::IR {
        context.set_debug_location(self.at);
        context.builder.build_call(self.function, &[], "").unwrap()
    }
}

/// Trait for lowering HIR Module to LLVM IR
pub trait HIRModuleLowering<'llvm> {
    /// Lower [`Module`] to LLVM IR
    fn to_ir(
        &self,
        llvm: &'llvm inkwell::context::Context,
        with_main: bool,
        compilation_module: compilation::Module,
    ) -> inkwell::module::Module<'llvm>;
}

impl<'llvm> HIRModuleLowering<'llvm> for ModuleData {
    /// Lower [`Module`] to LLVM IR
    fn to_ir(
        &self,
        llvm: &'llvm inkwell::context::Context,
        with_main: bool,
        compilation_module: compilation::Module,
    ) -> inkwell::module::Module<'llvm> {
        trace!(target: "lower_to_ir", "{self}");

        let name = self.name();

        let module = llvm.create_module(&name);
        module.set_source_file_name(&self.source_file.path().to_string_lossy());

        let mut context = ModuleContext::new(compilation_module, module, self.source_file());

        // First emit special variables with type info
        for variable in self
            .variables
            .values()
            .filter(|v| v.name().starts_with("Type<"))
        {
            variable.to_ir(&mut context);
        }

        let execute = context.module.add_function(
            &format!("{name}.execute"),
            context.types().none().fn_type(&[], false),
            None,
        );
        let at = self.statements.first().map(|s| s.start()).unwrap_or(0);

        FunctionContext::new(&mut context, execute, at).run(|context| {
            for init in context.module_context.initializers.clone() {
                init.to_ir(context);
            }

            for statement in &self.statements {
                if matches!(statement, Statement::Declaration(_)) {
                    statement.to_ir(context.module_context);
                    if matches!(statement, Statement::Declaration(Declaration::Variable(_))) {
                        let init = context.module_context.initializers.last().unwrap().clone();
                        init.to_ir(context);
                    }
                } else {
                    statement.to_ir(context);
                }
            }
        });

        if with_main {
            let main = context.module.add_function(
                "main",
                context.types().i32().fn_type(&[], false),
                None,
            );
            FunctionContext::new(&mut context, main, at).run(|context| {
                // Load 0 to return value
                context
                    .builder
                    .build_store(
                        context.return_value.unwrap(),
                        context.llvm().i32_type().const_zero(),
                    )
                    .unwrap();

                // Call execute
                context.set_debug_location(at);
                context.builder.build_call(execute, &[], "").unwrap();

                context.branch_to_return_block();
            });
        }

        let module = context.take_module();

        module
            .verify()
            .expect("Should never produce invalid modules");

        module
    }
}
