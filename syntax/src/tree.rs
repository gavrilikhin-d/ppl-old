use std::ops::Index;

use derive_more::From;
use miette::Diagnostic;

use crate::errors::Error;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseTree<'s> {
    /// Name of the tree. Empty string for anonymous trees
    pub name: String,
    /// Children of the subtree
    pub children: Vec<ParseTreeNode<'s>>,
}

impl<'s> ParseTree<'s> {
    /// Create empty tree
    pub fn empty() -> Self {
        Self {
            name: "".into(),
            children: vec![],
        }
    }

    /// Create empty tree with a name
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            children: vec![],
        }
    }

    /// Return this tree with another name
    pub fn with_name(self, name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            children: self.children,
        }
    }

    /// Push a node to the end of tree
    pub fn push(&mut self, node: impl Into<ParseTreeNode<'s>>) -> &mut Self {
        self.children.push(node.into());
        self
    }

    /// Return tree with element append to it
    pub fn with(mut self, node: impl Into<ParseTreeNode<'s>>) -> Self {
        self.push(node);
        self
    }

    /// Check if tree has errors
    pub fn has_errors(&self) -> bool {
        self.children.iter().any(|c| c.has_errors())
    }

    /// Check if tree has no errors
    pub fn is_ok(&self) -> bool {
        !self.has_errors()
    }

    /// Flatten one level of the tree,
    /// moving all children of subtrees without name to the root
    pub fn flatten(mut self) -> Self {
        let mut children = Vec::new();
        for child in self.children.drain(..) {
            match child {
                ParseTreeNode::Tree(tree) if tree.name.is_empty() => children.extend(tree.children),
                _ => children.push(child),
            }
        }
        self.children = children;
        self
    }

    /// Get subtree by name
    pub fn get(&self, name: &str) -> Option<&ParseTree<'s>> {
        self.children.iter().find_map(|c| match c {
            ParseTreeNode::Tree(tree) if tree.name == name => Some(tree),
            _ => None,
        })
    }

    /// Iterate over errors
    pub fn errors(&self) -> Box<dyn Iterator<Item = &dyn Error> + '_> {
        Box::new(self.children.iter().flat_map(|c| c.errors()))
    }

    /// Iterate over tokens
    pub fn tokens(&self) -> Box<dyn Iterator<Item = &'s str> + '_> {
        Box::new(self.children.iter().flat_map(|c| c.tokens()))
    }
}

impl<'s> Index<&str> for ParseTree<'s> {
    type Output = ParseTree<'s>;

    fn index(&self, name: &str) -> &Self::Output {
        self.get(name)
            .expect(format!("No subtree with name '{name}'").as_str())
    }
}

impl<'s> From<&'s str> for ParseTree<'s> {
    fn from(child: &'s str) -> Self {
        Self {
            name: "".into(),
            children: vec![child.into()],
        }
    }
}

impl<'s> From<ParseTreeNode<'s>> for ParseTree<'s> {
    fn from(child: ParseTreeNode<'s>) -> Self {
        Self {
            name: "".into(),
            children: vec![child.into()],
        }
    }
}

impl<'s, I: IntoParseTreeNode> From<I> for ParseTree<'s> {
    fn from(child: I) -> Self {
        Self {
            name: "".into(),
            children: vec![child.into()],
        }
    }
}

impl<'s> From<Vec<ParseTreeNode<'s>>> for ParseTree<'s> {
    fn from(children: Vec<ParseTreeNode<'s>>) -> Self {
        Self {
            name: "".into(),
            children,
        }
    }
}

impl<'s> From<Vec<ParseTree<'s>>> for ParseTree<'s> {
    fn from(children: Vec<ParseTree<'s>>) -> Self {
        Self {
            name: "".into(),
            children: children.into_iter().map(|c| c.into()).collect(),
        }
    }
}

impl<'s> From<Vec<&'s str>> for ParseTree<'s> {
    fn from(children: Vec<&'s str>) -> Self {
        Self {
            name: "".into(),
            children: children.into_iter().map(|c| c.into()).collect(),
        }
    }
}

/// Parse tree consist from leaf tokens an subtrees
#[derive(Debug, From)]
pub enum ParseTreeNode<'s> {
    /// Token
    Token(&'s str),
    /// Subtree
    Tree(ParseTree<'s>),
    /// Parsing error
    Error(Box<dyn Error>),
}

impl<'s> ParseTreeNode<'s> {
    /// Check if tree node has errors
    pub fn has_errors(&self) -> bool {
        match self {
            Self::Token(_) => false,
            Self::Tree(tree) => tree.has_errors(),
            Self::Error(_) => true,
        }
    }

    /// Check if tree node has no errors
    pub fn is_ok(&self) -> bool {
        !self.has_errors()
    }

    /// Iterate over errors
    pub fn errors(&self) -> Box<dyn Iterator<Item = &dyn Error> + '_> {
        match self {
            Self::Token(_) => Box::new(std::iter::empty()),
            Self::Tree(tree) => tree.errors(),
            Self::Error(err) => Box::new(std::iter::once(err.as_ref())),
        }
    }

    /// Iterate over tokens
    pub fn tokens(&self) -> Box<dyn Iterator<Item = &'s str> + '_> {
        match self {
            Self::Token(token) => Box::new(std::iter::once(token.clone())),
            Self::Tree(tree) => tree.tokens(),
            Self::Error(_) => Box::new(std::iter::empty()),
        }
    }
}

impl PartialEq for ParseTreeNode<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Token(a), Self::Token(b)) => a == b,
            (Self::Tree(a), Self::Tree(b)) => a == b,
            (Self::Error(_), Self::Error(_)) => true,
            _ => false,
        }
    }
}
impl Eq for ParseTreeNode<'_> {}

/// Helper trait to convert errors to parse tree
pub trait IntoParseTreeNode: Sized + Error {
    fn into_parse_tree_node(self) -> ParseTreeNode<'static> {
        ParseTreeNode::Error(Box::new(self))
    }
}

impl<'s, I: IntoParseTreeNode + Diagnostic + 'static> From<I> for ParseTreeNode<'s> {
    fn from(v: I) -> Self {
        v.into_parse_tree_node()
    }
}

#[cfg(test)]
mod test {
    use crate::{errors::Expected, ParseTree};

    #[test]
    fn create() {
        let tree = ParseTree::from("a").with("b");
        assert!(tree.is_ok());
        assert_eq!(tree, ParseTree::from(vec!["a", "b"]));

        let tree = ParseTree::from(vec!["a", "b"]).with("c");
        assert!(tree.is_ok());
        assert_eq!(tree, ParseTree::from(vec!["a", "b", "c"]));

        let tree = ParseTree::from(Expected {
            expected: "a".to_string(),
            at: 0.into(),
        })
        .with("b");
        assert!(tree.has_errors());
        assert_eq!(
            tree,
            ParseTree::from(Expected {
                expected: "a".to_string(),
                at: 0.into(),
            })
            .with("b")
        );
    }

    #[test]
    fn name() {
        let tree = ParseTree::from(vec![ParseTree::named("name").with("a")]);
        assert_eq!(tree["name"], ParseTree::named("name").with("a"));
        assert_eq!(tree.get("invalid"), None);
    }
}
