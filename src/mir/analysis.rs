use super::{basic_block::BasicBlock, body::Body};

/// Represent an analysis over CFG (MIR)
pub trait Analysis {
    /// Facts that we receive during the analysis
    type Facts: PartialOrd;

    /// Join facts from two different paths
    fn join(a: Self::Facts, b: Self::Facts) -> Self::Facts;

    /// Transfer facts from one basic block to another
    fn transfer(facts: Self::Facts, block: &BasicBlock) -> Self::Facts;
}

/// Analysis that starts from entry block and goes forward
pub trait ForwardAnalysis: Analysis {
    fn run_on(body: &Body) -> Self::Facts {
        let edges = body.edges();
        todo!()
    }
}

/// Analysis that starts from entry block and goes forward
pub trait BackwardAnalysis: Analysis {
    fn run_on(body: &Body) -> Self::Facts {
        todo!()
    }
}
