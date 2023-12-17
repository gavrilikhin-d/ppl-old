use std::collections::{HashMap, VecDeque};

use crate::mir::{basic_block::Terminator, body::Edge, operand::Operand, statement::Statement};

use super::{
    basic_block::{BasicBlock, BasicBlockWithID},
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

/// The direction of this analysis. Either `Forward` or `Backward`.
pub trait Direction {
    const IS_FORWARD: bool = !Self::IS_BACKWARD;

    const IS_BACKWARD: bool = !Self::IS_FORWARD;
}

pub struct Forward;

impl Direction for Forward {
    const IS_FORWARD: bool = true;
}

pub struct Backward;

impl Direction for Backward {
    const IS_BACKWARD: bool = true;
}

/// Defines the domain of a dataflow problem.
///
/// This trait specifies the lattice on which this analysis operates (the domain) as well as its
/// initial value at the entry point of each basic block.
pub trait AnalysisDomain {
    /// The type that holds the dataflow state at any given point in the program.
    type Domain: Clone + JoinSemiLattice;

    /// The direction of this analysis. Either `Forward` or `Backward`.
    type Direction: Direction = Forward;

    /// A descriptive name for this analysis. Used only for debugging.
    ///
    /// This name should be brief and contain no spaces, periods or other characters that are not
    /// suitable as part of a filename.
    const NAME: &'static str;

    /// Returns the initial value of the dataflow state upon entry to each basic block.
    fn bottom_value(&self, body: &Body) -> Self::Domain;

    /// Mutates the initial value of the dataflow state upon entry to the `START_BLOCK`.
    fn initialize_start_block(&self, body: &Body, state: &mut Self::Domain);
}
