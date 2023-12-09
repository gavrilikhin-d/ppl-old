use crate::ir::{Context, FunctionContext, ToIR};

use super::local::Local;

pub struct Body {
    pub locals: Vec<Local>,
}

impl<'llvm, 'm> ToIR<'llvm, FunctionContext<'llvm, 'm>> for Body {
    type IR = inkwell::values::FunctionValue<'llvm>;

    fn to_ir(&self, context: &mut FunctionContext<'llvm, 'm>) -> Self::IR {
        for (i, _) in self.locals.iter().enumerate() {
            let ty = context.types().i64();
            let name = format!("_{i}");
            context.builder.build_alloca(ty, &name);
        }

        context.builder.build_return(None);

        context.function.clone()
    }
}

#[cfg(test)]
mod test {
    use inkwell::values::AnyValue;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::{ir::ModuleContext, mir::local::Local};

    #[test]
    fn test_body() {
        let llvm = inkwell::context::Context::create();
        let module = llvm.create_module("test");
        let mut context = ModuleContext::new(module);

        let test = context.module.add_function(
            "test",
            context.llvm().void_type().fn_type(&[], false),
            None,
        );
        let mut context = FunctionContext::new(&mut context, test);

        let body = Body {
            locals: vec![Local {}, Local {}, Local {}],
        };

        let f = body.to_ir(&mut context);
        let ir = f.print_to_string().to_string();
        let expected = r#"
define void @test() {
  %_0 = alloca i64, align 8
  %_1 = alloca i64, align 8
  %_2 = alloca i64, align 8
  ret void
}
"#;
        assert_eq!(ir, expected.trim_start());
    }
}
