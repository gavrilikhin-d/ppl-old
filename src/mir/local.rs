use super::ty::Type;

pub struct Local {
    pub ty: Type,
}

pub struct LocalID(pub usize);
