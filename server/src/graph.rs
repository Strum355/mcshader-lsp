use petgraph::stable_graph::StableDiGraph;
use petgraph::stable_graph::NodeIndex;
use petgraph::Direction;
use petgraph::stable_graph::EdgeIndex;

use std::{collections::{HashMap, HashSet}, path::PathBuf, str::FromStr};

use super::IncludePosition;

/// Wraps a `StableDiGraph` with caching behaviour for node search by maintaining
/// an index for node value to node index and a reverse index.
/// This allows for **O(1)** lookup for a value if it exists, else **O(n)**.
pub struct CachedStableGraph {
    // StableDiGraph is used as it allows for String node values, essential for
    // generating the GraphViz DOT render.
    pub graph: StableDiGraph<String, IncludePosition>,
    cache: HashMap<PathBuf, NodeIndex>,
    // Maps a node index to its abstracted string representation.
    // Mainly used as the graph is based on NodeIndex and 
    reverse_index: HashMap<NodeIndex, PathBuf>,
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
    pub fn find_node(&mut self, name: &PathBuf) -> Option<NodeIndex> {
        match self.cache.get(name) {
            Some(n) => Some(*n),
            None => {
                // If the string is not in cache, O(n) search the graph (i know...) and then cache the NodeIndex
                // for later
                let n = self.graph.node_indices().find(|n| self.graph[*n] == name.to_str().unwrap().to_string());
                if let Some(n) = n {
                    self.cache.insert(name.into(), n);
                }
                n
            }
        }
    }

    pub fn get_node(&self, node: NodeIndex) -> PathBuf {
        PathBuf::from_str(&self.graph[node]).unwrap()
    }

    pub fn get_edge_meta(&self, parent: NodeIndex, child: NodeIndex) -> &IncludePosition {
        self.graph.edge_weight(self.graph.find_edge(parent, child).unwrap()).unwrap()
    }

    #[allow(dead_code)]
    pub fn remove_node(&mut self, name: &PathBuf) {
        let idx = self.cache.remove(name);
        if let Some(idx) = idx {
            self.graph.remove_node(idx);
        }
    }

    pub fn add_node(&mut self, name: &PathBuf) -> NodeIndex {
        if let Some(idx) = self.cache.get(name) {
            return *idx;
        }
        let idx = self.graph.add_node(name.to_str().unwrap().to_string());
        self.cache.insert(name.clone(), idx);
        self.reverse_index.insert(idx, name.clone());
        idx
    }

    pub fn add_edge(&mut self, parent: NodeIndex, child: NodeIndex, meta: IncludePosition) -> EdgeIndex {
        self.graph.add_edge(parent, child, meta)
    }

    pub fn remove_edge(&mut self, parent: NodeIndex, child: NodeIndex) {
        let edge = self.graph.find_edge(parent, child).unwrap();
        self.graph.remove_edge(edge);
    }

    #[allow(dead_code)]
    pub fn edge_weights(&self, node: NodeIndex) -> Vec<IncludePosition> {
        self.graph.edges(node).map(|e| e.weight().clone()).collect()
    }

    #[allow(dead_code)]
    pub fn child_node_names(&self, node: NodeIndex) -> Vec<PathBuf> {
        self.graph.neighbors(node).map(|n| self.reverse_index.get(&n).unwrap().clone()).collect()
    }

    pub fn child_node_meta(&self, node: NodeIndex) -> Vec<(PathBuf, IncludePosition)> {
        self.graph.neighbors(node).map(|n| {
            let edge = self.graph.find_edge(node, n).unwrap();
            let edge_meta = self.graph.edge_weight(edge).unwrap();
            return (self.reverse_index.get(&n).unwrap().clone(), edge_meta.clone())
        }).collect()
    }

    pub fn child_node_indexes(&self, node: NodeIndex) -> Vec<NodeIndex> {
        self.graph.neighbors(node).collect()
    }

    #[allow(dead_code)]
    pub fn parent_node_names(&self, node: NodeIndex) -> Vec<PathBuf> {
        self.graph.neighbors_directed(node, Direction::Incoming).map(|n| self.reverse_index.get(&n).unwrap().clone()).collect()
    }

    pub fn parent_node_indexes(&self, node: NodeIndex) -> Vec<NodeIndex> {
        self.graph.neighbors_directed(node, Direction::Incoming).collect()
    }

    #[allow(dead_code)]
    pub fn get_include_meta(&self, node: NodeIndex) -> Vec<IncludePosition> {
        self.graph.edges(node).map(|e| e.weight().clone()).collect()
    }

    pub fn collect_root_ancestors(&self, node: NodeIndex) -> Vec<NodeIndex> {
        let mut visited = HashSet::new();
        self.get_root_ancestors(node, node, &mut visited)
    }

    fn get_root_ancestors(&self, initial: NodeIndex, node: NodeIndex, visited: &mut HashSet<NodeIndex>) -> Vec<NodeIndex> {
        if node == initial && !visited.is_empty() {
            return vec![];
        }
        
        let parents = self.parent_node_indexes(node);
        let mut collection = Vec::with_capacity(parents.len());

        for ancestor in &parents {
            visited.insert(*ancestor);
        }

        for ancestor in &parents {
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