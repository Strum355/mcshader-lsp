mod graph;
pub mod dfs;
pub use graph::*;

pub use petgraph::stable_graph::NodeIndex;
pub use petgraph::dot::Config;
pub use petgraph::dot;

/// FilialTuple represents a tuple (not really) of a child and any legitimate
/// parent. Parent can be nullable in the case of the child being a top level
/// node in the tree.
#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub struct FilialTuple<T> {
    // pub child: NodeIndex,
    // pub parent: Option<NodeIndex>,
    pub child: T,
    pub parent: Option<T>,
}
