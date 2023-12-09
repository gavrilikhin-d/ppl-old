use super::{constant::Constant, local::LocalID};

pub enum Operand {
    Copy(LocalID),
    Move(LocalID),
    Constant(Constant),
}
