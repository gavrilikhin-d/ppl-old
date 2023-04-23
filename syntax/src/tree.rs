use derive_more::From;

/// Parse tree consist from leaf tokens an subtrees
#[derive(Debug, PartialEq, Clone, From)]
pub enum ParseTree<'s> {
    /// Token
    Token(&'s str),
    /// Tree with children
    Tree(Vec<ParseTree<'s>>),
}

impl<'s> From<Vec<&'s str>> for ParseTree<'s> {
    fn from(v: Vec<&'s str>) -> Self {
        Self::Tree(v.into_iter().map(|s| s.into()).collect())
    }
}

impl ParseTree<'_> {
    /// Append another tree to this tree
    pub fn append(&mut self, tree: impl Into<Self>) -> &mut Self {
        let tree = tree.into();
        match self {
            Self::Token(_) => *self = Self::Tree(vec![self.clone(), tree]),
            Self::Tree(v) => v.push(tree),
        };
        self
    }
}
