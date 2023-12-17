use std::collections::{HashMap, VecDeque};

use crate::mir::{basic_block::Terminator, body::Edge, operand::Operand, statement::Statement};

use super::{
    basic_block::{BasicBlockID, BasicBlockWithID},
    body::Body,
    local::LocalID,
};

/// A [partially ordered set][poset] that has a [least upper bound][lub] for any pair of elements
/// in the set.
///
/// [lub]: https://en.wikipedia.org/wiki/Infimum_and_supremum
/// [poset]: https://en.wikipedia.org/wiki/Partially_ordered_set
pub trait JoinSemiLattice: Eq {
    /// Computes the least upper bound of two elements, storing the result in `self` and returning
    /// `true` if `self` has changed.
    ///
    /// The lattice join operator is abbreviated as `∨`.
    fn join(&mut self, other: &Self) -> bool;
}

/// Direction of an analysis
#[derive(PartialEq, Eq)]
pub enum Direction {
    /// From first basic block to last one
    FORWARD,
    /// From last basic block to first one
    BACKWARD,
}
