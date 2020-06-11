use petgraph::stable_graph::StableDiGraph;
use petgraph::stable_graph::NodeIndex;
use petgraph::stable_graph::EdgeIndex;
use std::collections::HashMap;
use super::IncludePosition;

/// Wraps a `StableDiGraph` with caching behaviour for node search by maintaining
/// an index for node value to node index and a reverse index.
/// This allows for O(1) lookup for a value after the initial lookup.
pub struct CachedStableGraph {
    // StableDiGraph is used as it allows for String node values, essential for
    // generating the GraphViz DOT render.
    pub graph: StableDiGraph<String, IncludePosition>,
    cache: HashMap<String, NodeIndex>,
    // Maps a node index to its abstracted string representation.
    // Mainly used as the graph is based on NodeIndex and 
    reverse_index: HashMap<NodeIndex, String>,
}

impl CachedStableGraph {
    pub fn new() -> CachedStableGraph {
        CachedStableGraph{
            graph: StableDiGraph::new(),
            cache: HashMap::new(),
            reverse_index: HashMap::new(),
        }
    }

    /// Returns the `NodeIndex` for a given graph node with the value of `name`
    /// and caches the result in the `HashMap`. Complexity is O(1) if the value
    /// is cached, else O(n) as an exhaustive search must be done.
    pub fn find_node(&mut self, name: impl Into<String>) -> Option<NodeIndex> {
        let name_str = name.into();
        match self.cache.get(&name_str) {
            Some(n) => Some(*n),
            None => {
                // If the string is not in cache, O(n) search the graph (i know...) and then cache the NodeIndex
                // for later
                let n = self.graph.node_indices().find(|n| self.graph[*n] == name_str);
                if n.is_some() {
                    self.cache.insert(name_str, n.unwrap());
                }
                n
            }
        }
    }

    pub fn remove_node(&mut self, name: impl Into<String>) {
        let idx = self.cache.remove(&name.into());
        if idx.is_some() {
            self.graph.remove_node(idx.unwrap());
        }
    }

    pub fn add_node(&mut self, name: impl Into<String>) -> NodeIndex {
        let name_str = name.into();
        let idx = self.graph.add_node(name_str.clone());
        self.cache.insert(name_str.clone(), idx);
        self.reverse_index.insert(idx, name_str);
        idx
    }

    pub fn add_edge(&mut self, a: NodeIndex, b: NodeIndex, line: u64, start: u64, end: u64) -> EdgeIndex {
        let child_path = self.reverse_index.get(&b).unwrap().clone();
        self.graph.add_edge(a, b, IncludePosition{filepath: child_path, line, start, end})
    }

    /// Returns the filepaths of the neighbors for the given `NodeIndex`
    pub fn neighbors(&self, node: NodeIndex) -> Vec<String> {
        self.graph.neighbors(node).map(|n| self.reverse_index.get(&n).unwrap().clone()).collect()
    }

    pub fn get_includes(&self, node: NodeIndex) -> Vec<IncludePosition> {
        self.graph.edges(node).into_iter().map(|e| e.weight().clone()).collect()
    }
}