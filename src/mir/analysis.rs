use std::{collections::VecDeque, ops::AddAssign};

use crate::mir::body::Edge;

use super::{
    basic_block::{BasicBlock, BasicBlockID},
    body::Body,
};

/// Direction of an analysis
#[derive(PartialEq, Eq)]
pub enum Direction {
    /// From first basic block to last one
    FORWARD,
    /// From last basic block to first one
    BACKWARD,
}

/// Represent an analysis over CFG (MIR)
pub trait Analysis: Sized + Default {
    /// Facts that we receive during the analysis
    type Facts: PartialOrd + AddAssign;

    /// Direction of an analysis
    const DIRECTION: Direction = Direction::FORWARD;

    /// Get facts about basic block
    fn facts(&mut self, block: BasicBlockID) -> &mut Self::Facts;

    /// Transfer facts from one basic block to another
    fn transfer(facts: &Self::Facts, block: &BasicBlock) -> Self::Facts;

    /// Run analysis on CFG
    fn run_on(body: &Body) -> Self {
        let mut analysis = Self::default();

        let is_forward: bool = Self::DIRECTION == Direction::FORWARD;

        let edges = body.edges();
        let mut edges = if is_forward {
            VecDeque::from_iter(edges)
        } else {
            VecDeque::from_iter(edges.map(|e| e.reversed()))
        };
        while !edges.is_empty() {
            let Edge { from, to } = edges.pop_front().unwrap();
            let block = &body.basic_blocks[from.0];
            let input_facts: &Self::Facts = analysis.facts(from);
            let output_facts = Self::transfer(input_facts, block);

            let to_facts = analysis.facts(to);
            if output_facts <= *to_facts {
                continue;
            }

            *to_facts += output_facts;
            let block = &body.basic_blocks[to.0];
            if is_forward {
                edges.extend(body.edges_from(to))
            } else {
                edges.extend(body.edges_to(to).map(|e| e.reversed()))
            }
        }
        analysis
    }
}
