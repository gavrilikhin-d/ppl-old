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

/// Represent an analysis over CFG (MIR)
pub trait Analysis: Sized + Default {
    /// Facts that we receive during the analysis
    type Facts;

    /// Direction of an analysis
    const DIRECTION: Direction = Direction::FORWARD;

    /// Get facts about basic block
    fn facts(&mut self, block: BasicBlockID) -> &mut Self::Facts;

    /// Transfer facts from one basic block to another
    fn transfer(facts: &Self::Facts, block: &BasicBlockWithID) -> Self::Facts;

    /// Join facts together
    fn join(accumulator: &mut Self::Facts, facts: Self::Facts);

    /// `a` <= `b`?
    fn le(a: &Self::Facts, b: &Self::Facts) -> bool;

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
            let output_facts = Self::transfer(
                input_facts,
                &BasicBlockWithID {
                    id: from,
                    data: block,
                },
            );

            let to_facts = analysis.facts(to);
            if Self::le(&output_facts, to_facts) {
                continue;
            }

            Self::join(to_facts, output_facts);
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

/// Initialization state of variable
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Initialized {
    No,
    Yes,
    Maybe,
}

impl Initialized {
    pub fn join(self, other: Initialized) -> Initialized {
        use Initialized::*;
        match (self, other) {
            (No, No) => No,
            (Yes, Yes) => Yes,
            (_, _) => Maybe,
        }
    }
}

#[derive(PartialEq, Eq)]
pub struct InitializedVariables(HashMap<LocalID, Initialized>);

#[derive(Default)]
pub struct InitializedVariablesAnalysis {
    pub facts: Vec<Vec<HashMap<LocalID, Initialized>>>,
}

impl Analysis for InitializedVariablesAnalysis {
    type Facts = Vec<HashMap<LocalID, Initialized>>;

    fn join(acc: &mut Self::Facts, facts: Self::Facts) {
        let initialized = facts.last().unwrap();
        for vars in acc {
            for (var, init) in initialized {
                vars.entry(*var)
                    .and_modify(|entry| *entry = entry.join(*init))
                    .or_insert(*init);
            }
        }
    }

    fn le(a: &Self::Facts, b: &Self::Facts) -> bool {
        assert!(a.len() == b.len());
        a == b
    }

    fn facts(&mut self, block: BasicBlockID) -> &mut Self::Facts {
        while block.0 >= self.facts.len() {
            self.facts.push(Self::Facts::new())
        }
        &mut self.facts[block.0]
    }

    fn transfer(input: &Self::Facts, block: &BasicBlockWithID) -> Self::Facts {
        let mut facts = Vec::new();
        let mut state = HashMap::new();

        use Terminator::*;
        match &block.data.terminator {
            Switch {
                operand: Operand::Copy(local) | Operand::Move(local),
                ..
            } => {
                state.insert(
                    *local,
                    input
                        .last()
                        .unwrap()
                        .get(local)
                        .cloned()
                        .unwrap_or(Initialized::No),
                );
            }
            _ => {}
        }

        for stmt in block.data.statements.iter() {
            use Statement::*;
            match stmt {
                Assign { lhs, .. } => {
                    state.insert(*lhs, Initialized::Yes);
                }
                _ => {}
            }
            facts.push(state.clone());
        }
        facts
    }
}
