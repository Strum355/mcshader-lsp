use petgraph::stable_graph::StableDiGraph;
use petgraph::stable_graph::NodeIndex;
use petgraph::Direction;
use petgraph::stable_graph::EdgeIndex;

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use super::IncludePosition;

/// Wraps a `StableDiGraph` with caching behaviour for node search by maintaining
/// an index for node value to node index and a reverse index.
/// This allows for **O(1)** lookup for a value after the initial lookup.
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
    /// and caches the result in the `HashMap`. Complexity is **O(1)** if the value
    /// is cached (which should always be the case), else **O(n)** where **n** is
    /// the number of node indices, as an exhaustive search must be done.
    pub fn find_node(&mut self, name: impl Into<String>) -> Option<NodeIndex> {
        let name_str = name.into();
        match self.cache.get(&name_str) {
            Some(n) => Some(*n),
            None => {
                // If the string is not in cache, O(n) search the graph (i know...) and then cache the NodeIndex
                // for later
                let n = self.graph.node_indices().find(|n| self.graph[*n] == name_str);
                if let Some(n) = n {
                    self.cache.insert(name_str, n);
                }
                n
            }
        }
    }

    /* pub fn get_node(&self, node: NodeIndex) -> &IncludePosition {
        self.graph.node_weight(node).expect("node index not found in graph")
    } */

    pub fn remove_node(&mut self, name: impl Into<String>) {
        let idx = self.cache.remove(&name.into());
        if let Some(idx) = idx {
            self.graph.remove_node(idx);
        }
    }

    pub fn add_node(&mut self, name: impl Into<String>) -> NodeIndex {
        let name_str = name.into();
        let idx = self.graph.add_node(name_str.clone());
        self.cache.insert(name_str.clone(), idx);
        self.reverse_index.insert(idx, name_str);
        idx
    }

    pub fn add_edge(&mut self, parent: NodeIndex, child: NodeIndex, line: u64, start: u64, end: u64) -> EdgeIndex {
        let child_path = self.reverse_index.get(&child).unwrap().clone();
        self.graph.add_edge(parent, child, IncludePosition{filepath: child_path, line, start, end})
    }

    pub fn edge_weights(&self, node: NodeIndex) -> Vec<IncludePosition> {
        self.graph.edges(node).map(|e| e.weight().clone()).collect()
    }

    pub fn child_node_names(&self, node: NodeIndex) -> Vec<String> {
        self.graph.neighbors(node).map(|n| self.reverse_index.get(&n).unwrap().clone()).collect()
    }

    pub fn child_node_indexes(&self, node: NodeIndex) -> Vec<NodeIndex> {
        self.graph.neighbors(node).collect()
    }

    pub fn parent_node_names(&self, node: NodeIndex) -> Vec<String> {
        self.graph.neighbors_directed(node, Direction::Incoming).map(|n| self.reverse_index.get(&n).unwrap().clone()).collect()
    }

    pub fn parent_node_indexes(&self, node: NodeIndex) -> Vec<NodeIndex> {
        self.graph.neighbors_directed(node, Direction::Incoming).collect()
    }

    pub fn get_include_meta(&self, node: NodeIndex) -> Vec<IncludePosition> {
        self.graph.edges(node).map(|e| e.weight().clone()).collect()
    }

    pub fn collect_root_ancestors(&self, node: NodeIndex) -> Vec<NodeIndex> {
        let mut visited = HashSet::new();
        //visited.insert(node);
        self.get_root_ancestors(node, node, &mut visited)
    }

    fn get_root_ancestors(&self, initial: NodeIndex, node: NodeIndex, visited: &mut HashSet<NodeIndex>) -> Vec<NodeIndex> {
        if visited.contains(&node) {
            return vec![node];
        } else if node == initial && !visited.is_empty() {
            return vec![];
        }
        
        let parents = Rc::new(self.parent_node_indexes(node));
        let mut collection = Vec::with_capacity(parents.len());

        for ancestor in parents.as_ref() {
            visited.insert(*ancestor);
        }

        for ancestor in parents.as_ref() {
            let ancestors = self.parent_node_indexes(*ancestor);
            if !ancestors.is_empty() {
                collection.extend(self.get_root_ancestors(initial, *ancestor, visited));
            } else {
                collection.push(*ancestor);
            }
        }

        collection
    }
}