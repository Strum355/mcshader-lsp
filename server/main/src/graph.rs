use petgraph::stable_graph::EdgeIndex;
use petgraph::stable_graph::NodeIndex;
use petgraph::stable_graph::StableDiGraph;
use petgraph::visit::EdgeRef;
use petgraph::Direction;

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    str::FromStr,
};

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
    // Mainly used as the graph is based on NodeIndex.
    reverse_index: HashMap<NodeIndex, PathBuf>,
}

impl CachedStableGraph {
    #[allow(clippy::new_without_default)]
    pub fn new() -> CachedStableGraph {
        CachedStableGraph {
            graph: StableDiGraph::new(),
            cache: HashMap::new(),
            reverse_index: HashMap::new(),
        }
    }

    /// Returns the `NodeIndex` for a given graph node with the value of `name`
    /// and caches the result in the `HashMap`. Complexity is **O(1)** if the value
    /// is cached (which should always be the case), else **O(n)** where **n** is
    /// the number of node indices, as an exhaustive search must be done.
    pub fn find_node(&mut self, name: &Path) -> Option<NodeIndex> {
        match self.cache.get(name) {
            Some(n) => Some(*n),
            None => {
                // If the string is not in cache, O(n) search the graph (i know...) and then cache the NodeIndex
                // for later
                let n = self.graph.node_indices().find(|n| self.graph[*n] == name.to_str().unwrap());
                if let Some(n) = n {
                    self.cache.insert(name.into(), n);
                }
                n
            }
        }
    }

    // Returns the `PathBuf` for a given `NodeIndex`
    pub fn get_node(&self, node: NodeIndex) -> PathBuf {
        PathBuf::from_str(&self.graph[node]).unwrap()
    }

    /// returns an iterator over all the `IncludePosition`'s between a parent and its child for all the positions
    /// that the child may be imported into the parent, in order of import.
    pub fn get_edge_metas(&self, parent: NodeIndex, child: NodeIndex) -> impl Iterator<Item = IncludePosition> + '_ {
        let mut edges = self
            .graph
            .edges(parent)
            .filter_map(move |edge| {
                let target = self.graph.edge_endpoints(edge.id()).unwrap().1;
                if target != child {
                    return None;
                }
                Some(self.graph[edge.id()])
            })
            .collect::<Vec<IncludePosition>>();
        edges.sort_by(|x, y| x.line.cmp(&y.line));
        edges.into_iter()
    }

    pub fn add_node(&mut self, name: &Path) -> NodeIndex {
        if let Some(idx) = self.cache.get(name) {
            return *idx;
        }
        let idx = self.graph.add_node(name.to_str().unwrap().to_string());
        self.cache.insert(name.to_owned(), idx);
        self.reverse_index.insert(idx, name.to_owned());
        idx
    }

    pub fn add_edge(&mut self, parent: NodeIndex, child: NodeIndex, meta: IncludePosition) -> EdgeIndex {
        self.graph.add_edge(parent, child, meta)
    }

    pub fn remove_edge(&mut self, parent: NodeIndex, child: NodeIndex, position: IncludePosition) {
        self.graph
            .edges(parent)
            .find(|edge| self.graph.edge_endpoints(edge.id()).unwrap().1 == child && *edge.weight() == position)
            .map(|edge| edge.id())
            .and_then(|edge| self.graph.remove_edge(edge));
    }

    pub fn child_node_metas(&self, node: NodeIndex) -> impl Iterator<Item = (PathBuf, IncludePosition)> + '_ {
        self.graph.neighbors(node).map(move |n| {
            let edge = self.graph.find_edge(node, n).unwrap();
            let edge_meta = self.graph.edge_weight(edge).unwrap();
            return (self.reverse_index.get(&n).unwrap().clone(), *edge_meta);
        })
    }

    pub fn child_node_indexes(&self, node: NodeIndex) -> impl Iterator<Item = NodeIndex> + '_ {
        self.graph.neighbors(node)
    }

    pub fn collect_root_ancestors(&self, node: NodeIndex) -> Vec<NodeIndex> {
        let mut visited = HashSet::new();
        self.get_root_ancestors(node, node, &mut visited)
    }

    // TODO: impl Iterator
    fn parent_node_indexes(&self, node: NodeIndex) -> Vec<NodeIndex> {
        self.graph.neighbors_directed(node, Direction::Incoming).collect()
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

#[cfg(test)]
impl CachedStableGraph {
    fn parent_node_names(&self, node: NodeIndex) -> Vec<PathBuf> {
        self.graph
            .neighbors_directed(node, Direction::Incoming)
            .map(|n| self.reverse_index.get(&n).unwrap().clone())
            .collect()
    }

    fn child_node_names(&self, node: NodeIndex) -> Vec<PathBuf> {
        self.graph
            .neighbors(node)
            .map(|n| self.reverse_index.get(&n).unwrap().clone())
            .collect()
    }

    fn remove_node(&mut self, name: &Path) {
        let idx = self.cache.remove(name);
        if let Some(idx) = idx {
            self.graph.remove_node(idx);
        }
    }
}

#[cfg(test)]
mod graph_test {
    use std::path::PathBuf;

    use petgraph::graph::NodeIndex;

    use crate::{graph::CachedStableGraph, IncludePosition};

    #[test]
    #[logging_macro::log_scope]
    fn test_graph_two_connected_nodes() {
        let mut graph = CachedStableGraph::new();

        let idx1 = graph.add_node(&PathBuf::from("sample"));
        let idx2 = graph.add_node(&PathBuf::from("banana"));
        graph.add_edge(idx1, idx2, IncludePosition { line: 3, start: 0, end: 0 });

        let children = graph.child_node_names(idx1);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0], Into::<PathBuf>::into("banana".to_string()));

        let children: Vec<NodeIndex> = graph.child_node_indexes(idx1).collect();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0], idx2);

        let parents = graph.parent_node_names(idx1);
        assert_eq!(parents.len(), 0);

        let parents = graph.parent_node_names(idx2);
        assert_eq!(parents.len(), 1);
        assert_eq!(parents[0], Into::<PathBuf>::into("sample".to_string()));

        let parents = graph.parent_node_indexes(idx2);
        assert_eq!(parents.len(), 1);
        assert_eq!(parents[0], idx1);

        let ancestors = graph.collect_root_ancestors(idx2);
        assert_eq!(ancestors.len(), 1);
        assert_eq!(ancestors[0], idx1);

        let ancestors = graph.collect_root_ancestors(idx1);
        assert_eq!(ancestors.len(), 0);

        graph.remove_node(&PathBuf::from("sample"));
        assert_eq!(graph.graph.node_count(), 1);
        assert!(graph.find_node(&PathBuf::from("sample")).is_none());

        let neighbors = graph.child_node_names(idx2);
        assert_eq!(neighbors.len(), 0);
    }

    #[test]
    #[logging_macro::log_scope]
    fn test_double_import() {
        let mut graph = CachedStableGraph::new();

        let idx0 = graph.add_node(&PathBuf::from("0"));
        let idx1 = graph.add_node(&PathBuf::from("1"));

        graph.add_edge(idx0, idx1, IncludePosition { line: 2, start: 0, end: 0 });
        graph.add_edge(idx0, idx1, IncludePosition { line: 4, start: 0, end: 0 });

        //     0
        //    / \
        //   1   1

        assert_eq!(2, graph.get_edge_metas(idx0, idx1).count());

        let mut edge_metas = graph.get_edge_metas(idx0, idx1);
        assert_eq!(Some(IncludePosition { line: 2, start: 0, end: 0 }), edge_metas.next());
        assert_eq!(Some(IncludePosition { line: 4, start: 0, end: 0 }), edge_metas.next());
    }

    #[test]
    #[logging_macro::log_scope]
    fn test_collect_root_ancestors() {
        {
            let mut graph = CachedStableGraph::new();

            let idx0 = graph.add_node(&PathBuf::from("0"));
            let idx1 = graph.add_node(&PathBuf::from("1"));
            let idx2 = graph.add_node(&PathBuf::from("2"));
            let idx3 = graph.add_node(&PathBuf::from("3"));

            graph.add_edge(idx0, idx1, IncludePosition { line: 2, start: 0, end: 0 });
            graph.add_edge(idx1, idx2, IncludePosition { line: 3, start: 0, end: 0 });
            graph.add_edge(idx3, idx1, IncludePosition { line: 4, start: 0, end: 0 });

            //       0  3
            //       |/
            //       1
            //       |
            //       2

            let roots = graph.collect_root_ancestors(idx2);
            assert_eq!(roots, vec![idx3, idx0]);

            let roots = graph.collect_root_ancestors(idx1);
            assert_eq!(roots, vec![idx3, idx0]);

            let roots = graph.collect_root_ancestors(idx0);
            assert_eq!(roots, vec![]);

            let roots = graph.collect_root_ancestors(idx3);
            assert_eq!(roots, vec![]);
        }
        {
            let mut graph = CachedStableGraph::new();

            let idx0 = graph.add_node(&PathBuf::from("0"));
            let idx1 = graph.add_node(&PathBuf::from("1"));
            let idx2 = graph.add_node(&PathBuf::from("2"));
            let idx3 = graph.add_node(&PathBuf::from("3"));

            graph.add_edge(idx0, idx1, IncludePosition { line: 2, start: 0, end: 0 });
            graph.add_edge(idx0, idx2, IncludePosition { line: 3, start: 0, end: 0 });
            graph.add_edge(idx1, idx3, IncludePosition { line: 5, start: 0, end: 0 });

            //       0
            //      / \
            //     1   2
            //    /
            //   3

            let roots = graph.collect_root_ancestors(idx3);
            assert_eq!(roots, vec![idx0]);

            let roots = graph.collect_root_ancestors(idx2);
            assert_eq!(roots, vec![idx0]);

            let roots = graph.collect_root_ancestors(idx1);
            assert_eq!(roots, vec![idx0]);

            let roots = graph.collect_root_ancestors(idx0);
            assert_eq!(roots, vec![]);
        }
        {
            let mut graph = CachedStableGraph::new();

            let idx0 = graph.add_node(&PathBuf::from("0"));
            let idx1 = graph.add_node(&PathBuf::from("1"));
            let idx2 = graph.add_node(&PathBuf::from("2"));
            let idx3 = graph.add_node(&PathBuf::from("3"));

            graph.add_edge(idx0, idx1, IncludePosition { line: 2, start: 0, end: 0 });
            graph.add_edge(idx2, idx3, IncludePosition { line: 3, start: 0, end: 0 });
            graph.add_edge(idx1, idx3, IncludePosition { line: 5, start: 0, end: 0 });

            //       0
            //        \
            //     2   1
            //      \ /
            //       3

            let roots = graph.collect_root_ancestors(idx3);
            assert_eq!(roots, vec![idx0, idx2]);

            let roots = graph.collect_root_ancestors(idx2);
            assert_eq!(roots, vec![]);

            let roots = graph.collect_root_ancestors(idx1);
            assert_eq!(roots, vec![idx0]);

            let roots = graph.collect_root_ancestors(idx0);
            assert_eq!(roots, vec![]);
        }
        {
            let mut graph = CachedStableGraph::new();

            let idx0 = graph.add_node(&PathBuf::from("0"));
            let idx1 = graph.add_node(&PathBuf::from("1"));
            let idx2 = graph.add_node(&PathBuf::from("2"));
            let idx3 = graph.add_node(&PathBuf::from("3"));

            graph.add_edge(idx0, idx1, IncludePosition { line: 2, start: 0, end: 0 });
            graph.add_edge(idx1, idx2, IncludePosition { line: 4, start: 0, end: 0 });
            graph.add_edge(idx1, idx3, IncludePosition { line: 6, start: 0, end: 0 });

            //       0
            //       |
            //       1
            //      / \
            //     2   3

            let roots = graph.collect_root_ancestors(idx3);
            assert_eq!(roots, vec![idx0]);

            let roots = graph.collect_root_ancestors(idx2);
            assert_eq!(roots, vec![idx0]);

            let roots = graph.collect_root_ancestors(idx1);
            assert_eq!(roots, vec![idx0]);

            let roots = graph.collect_root_ancestors(idx0);
            assert_eq!(roots, vec![]);
        }
    }
}
