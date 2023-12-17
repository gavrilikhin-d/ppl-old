use std::collections::{HashMap, VecDeque};

use crate::mir::{basic_block::Terminator, body::Edge, operand::Operand, statement::Statement};

use super::{
    basic_block::{BasicBlockID, BasicBlockWithID},
    body::Body,
    local::LocalID,
};

/// Direction of an analysis
#[derive(PartialEq, Eq)]
pub enum Direction {
    /// From first basic block to last one
    FORWARD,
    /// From last basic block to first one
    BACKWARD,
}
