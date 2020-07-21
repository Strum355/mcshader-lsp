use petgraph::stable_graph::NodeIndex;

use crate::graph::CachedStableGraph;

pub struct Dfs<'a> {
    stack: Vec<NodeIndex>,
    graph: &'a CachedStableGraph
}

impl <'a> Dfs<'a> {
    pub fn new(graph: &'a CachedStableGraph, start: NodeIndex) -> Self {
        Dfs {
            stack: vec![start],
            graph
        }
    }
}