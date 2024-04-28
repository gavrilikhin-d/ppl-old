use derive_visitor::VisitorMut;

use crate::hir::Type;

#[derive(VisitorMut)]
#[visitor(Type(enter))]
pub struct ReplaceSelf {
    ty: Type,
}

impl ReplaceSelf {
    pub fn with(ty: Type) -> Self {
        Self { ty }
    }

    fn enter_type(&mut self, ty: &mut Type) {
        if let Type::SelfType(_) = ty {
            *ty = self.ty.clone();
        }
    }
}
