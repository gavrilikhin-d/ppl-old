use super::{local::LocalID, operand::Operand};

pub enum Statement {
    Assign { lhs: LocalID, rhs: Operand },
}
