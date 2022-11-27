use anyhow::format_err;
use anyhow::Result;
use petgraph::stable_graph::EdgeIndex;
use petgraph::stable_graph::NodeIndex;
use petgraph::stable_graph::StableDiGraph;
use petgraph::visit::EdgeRef;
use petgraph::Direction;

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Index;
use std::ops::IndexMut;

/// Wraps a `StableDiGraph` with caching behaviour for node search by maintaining
/// an index for node value to node index and a reverse index.
/// This allows for **O(1)** lookup for a value if it exists, else **O(n)**.
pub struct CachedStableGraph<K, V>
where
    K: Hash + Clone + Eq + Debug,
    V: Ord + Copy,
{
    // StableDiGraph is used as it allows for String node values, essential for
    // generating the GraphViz DOT render.
    pub graph: StableDiGraph<K, V>,
    cache: HashMap<K, NodeIndex>,
    // Maps a node index to its abstracted string representation.
    // Mainly used as the graph is based on NodeIndex.
    #[cfg(test)]
    reverse_index: HashMap<NodeIndex, K>,
}

impl<K, V> CachedStableGraph<K, V>
where
    K: Hash + Clone + Eq + Debug,
    V: Ord + Copy,
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        CachedStableGraph {
            graph: StableDiGraph::new(),
            cache: HashMap::new(),
            #[cfg(test)]
            reverse_index: HashMap::new(),
        }
    }

    #[inline]
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    // #[inline]
    // pub fn inner(&self) -> &StableDiGraph<K, V> {
    //     &self.graph
    // }

    pub fn parents(&self, node: NodeIndex) -> impl Iterator<Item = NodeIndex> + '_ {
        self.graph.edges_directed(node, Direction::Incoming).map(|e| e.source())
    }

    /// Returns the `NodeIndex` for a given graph node with the value of `name`
    /// and caches the result in the `HashMap`. Complexity is **O(1)** if the value
    /// is cached (which should always be the case), else **O(n)** where **n** is
    /// the number of node indices, as an exhaustive search must be done.
    pub fn find_node(&mut self, name: &K) -> Option<NodeIndex> {
        match self.cache.get(name) {
            Some(n) => Some(*n),
            None => {
                // If the string is not in cache, O(n) search the graph (i know...) and then cache the NodeIndex
                // for later
                let n = self.graph.node_indices().find(|n| self.graph[*n] == *name);
                if let Some(n) = n {
                    self.cache.insert(name.clone(), n);
                }
                n
            }
        }
    }

    /// Returns an iterator over all the edge values of type `V`'s between a parent and its child for all the
    /// positions that the child may be imported into the parent, in order of import.
    pub fn get_edges_between(&self, parent: NodeIndex, child: NodeIndex) -> impl DoubleEndedIterator<Item = V> + '_ {
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
            .collect::<Vec<V>>();
        edges.sort();
        edges.into_iter()
    }

    /// Returns an iterator over all the `(NodeIndex, T)` tuples between a node and all its children, in order
    /// of import.
    pub fn get_all_edges_from(&self, parent: NodeIndex) -> impl DoubleEndedIterator<Item = (NodeIndex, V)> + '_ {
        let mut edges = self
            .graph
            .edges(parent)
            .map(|edge| {
                let child = self.graph.edge_endpoints(edge.id()).unwrap().1;
                (child, self.graph[edge.id()])
            })
            .collect::<Vec<_>>();
        edges.sort_by(|x, y| x.1.cmp(&y.1));
        edges.into_iter()
    }

    // pub fn symmetric_closure(&self) {}

    pub fn add_node(&mut self, name: &K) -> NodeIndex {
        if let Some(idx) = self.cache.get(name) {
            return *idx;
        }
        let idx = self.graph.add_node(name.clone());
        self.cache.insert(name.to_owned(), idx);
        #[cfg(test)]
        self.reverse_index.insert(idx, name.to_owned());
        idx
    }

    /// Adds a directional edge of type `V` between `parent` and `child`.
    #[inline]
    pub fn add_edge(&mut self, parent: NodeIndex, child: NodeIndex, meta: V) -> EdgeIndex {
        self.graph.add_edge(parent, child, meta)
    }

    #[inline]
    pub fn remove_edge(&mut self, parent: NodeIndex, child: NodeIndex, position: V) {
        self.graph
            .edges(parent)
            .find(|edge| self.graph.edge_endpoints(edge.id()).unwrap().1 == child && *edge.weight() == position)
            .map(|edge| edge.id())
            .and_then(|edge| self.graph.remove_edge(edge));
    }

    #[inline]
    pub fn child_node_indexes(&self, node: NodeIndex) -> impl Iterator<Item = NodeIndex> + '_ {
        self.graph.neighbors(node)
    }

    #[inline]
    pub fn parent_node_indexes(&self, node: NodeIndex) -> impl Iterator<Item = NodeIndex> + '_ {
        self.graph.neighbors_directed(node, Direction::Incoming)
    }

    pub fn root_ancestors_for_key(&mut self, path: &K) -> Result<Option<Vec<NodeIndex>>> {
        let node = match self.find_node(path) {
            Some(n) => n,
            None => return Err(format_err!("node not found {:?}", path)),
        };
        Ok(self.root_ancestors(node))
    }

    #[inline]
    pub fn root_ancestors(&self, node: NodeIndex) -> Option<Vec<NodeIndex>> {
        let mut visited = HashSet::new();
        self.get_root_ancestors(node, node, &mut visited)
    }

    fn get_root_ancestors(&self, initial: NodeIndex, node: NodeIndex, visited: &mut HashSet<NodeIndex>) -> Option<Vec<NodeIndex>> {
        if node == initial && !visited.is_empty() {
            return None;
        }

        let parents: Vec<_> = self.parent_node_indexes(node).collect();
        let mut collection = Vec::with_capacity(parents.len());

        for ancestor in &parents {
            visited.insert(*ancestor);
        }

        for ancestor in &parents {
            if self.parent_node_indexes(*ancestor).next().is_some() {
                collection.extend(self.get_root_ancestors(initial, *ancestor, visited).unwrap_or_default());
            } else {
                collection.push(*ancestor);
            }
        }

        Some(collection)
    }
}

impl<K, V> Index<NodeIndex> for CachedStableGraph<K, V>
where
    K: Hash + Clone + Eq + Debug,
    V: Ord + Copy,
{
    type Output = K;

    #[inline]
    fn index(&self, index: NodeIndex) -> &Self::Output {
        &self.graph[index]
    }
}

impl<K, V> IndexMut<NodeIndex> for CachedStableGraph<K, V>
where
    K: Hash + Clone + Eq + Debug,
    V: Ord + Copy,
{
    #[inline]
    fn index_mut(&mut self, index: NodeIndex) -> &mut Self::Output {
        self.graph.index_mut(index)
    }
}

#[cfg(test)]
impl<K, V> CachedStableGraph<K, V>
where
    K: Hash + Clone + Eq + Debug,
    V: Ord + Copy,
{
    fn parent_node_names(&self, node: NodeIndex) -> Vec<K> {
        self.graph
            .neighbors_directed(node, Direction::Incoming)
            .map(|n| self.reverse_index.get(&n).unwrap().clone())
            .collect()
    }

    fn child_node_names(&self, node: NodeIndex) -> Vec<K> {
        self.graph
            .neighbors(node)
            .map(|n| self.reverse_index.get(&n).unwrap().clone())
            .collect()
    }

    fn remove_node(&mut self, name: &K) {
        let idx = self.cache.remove(name);
        if let Some(idx) = idx {
            self.graph.remove_node(idx);
        }
    }
}

impl<'a, K, V> From<&'a CachedStableGraph<K, V>> for &'a StableDiGraph<K, V>
where
    K: Hash + Clone + Eq + Debug,
    V: Ord + Copy,
{
    #[inline]
    fn from(val: &'a CachedStableGraph<K, V>) -> Self {
        &val.graph
    }
}

#[cfg(test)]
mod graph_test {
    use petgraph::graph::NodeIndex;

    use crate::graph::CachedStableGraph;

    #[test]
    #[logging_macro::scope]
    fn test_graph_two_connected_nodes() {
        let mut graph = CachedStableGraph::new();

        let idx1 = graph.add_node(&"sample");
        let idx2 = graph.add_node(&"banana");
        graph.add_edge(idx1, idx2, 100);

        let children = graph.child_node_names(idx1);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0], "banana");

        let children: Vec<NodeIndex> = graph.child_node_indexes(idx1).collect();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0], idx2);

        let parents = graph.parent_node_names(idx1);
        assert_eq!(parents.len(), 0);

        let parents = graph.parent_node_names(idx2);
        assert_eq!(parents.len(), 1);
        assert_eq!(parents[0], "sample");

        let parents: Vec<_> = graph.parent_node_indexes(idx2).collect();
        assert_eq!(parents.len(), 1);
        assert_eq!(parents[0], idx1);

        let ancestors = graph.root_ancestors(idx2).unwrap();
        assert_eq!(ancestors.len(), 1);
        assert_eq!(ancestors[0], idx1);

        let ancestors = graph.root_ancestors(idx1).unwrap();
        assert_eq!(ancestors.len(), 0);

        graph.remove_node(&"sample");
        assert_eq!(graph.graph.node_count(), 1);
        assert!(graph.find_node(&"sample").is_none());

        let neighbors = graph.child_node_names(idx2);
        assert_eq!(neighbors.len(), 0);
    }

    #[test]
    #[logging_macro::scope]
    fn test_double_import() {
        let mut graph = CachedStableGraph::new();

        let idx0 = graph.add_node(&"0");
        let idx1 = graph.add_node(&"1");

        graph.add_edge(idx0, idx1, 200);
        graph.add_edge(idx0, idx1, 400);

        //     0
        //    / \
        //   1   1

        assert_eq!(2, graph.get_edges_between(idx0, idx1).count());

        let mut edge_metas = graph.get_edges_between(idx0, idx1);
        assert_eq!(Some(200), edge_metas.next());
        assert_eq!(Some(400), edge_metas.next());
    }

    #[test]
    #[logging_macro::scope]
    fn test_collect_root_ancestors() {
        {
            let mut graph = CachedStableGraph::new();

            let idx0 = graph.add_node(&"0");
            let idx1 = graph.add_node(&"1");
            let idx2 = graph.add_node(&"2");
            let idx3 = graph.add_node(&"3");

            graph.add_edge(idx0, idx1, 200);
            graph.add_edge(idx1, idx2, 300);
            graph.add_edge(idx3, idx1, 400);

            //       0  3
            //       |/
            //       1
            //       |
            //       2

            let roots = graph.root_ancestors(idx2).unwrap();
            assert_eq!(roots, vec![idx3, idx0]);

            let roots = graph.root_ancestors(idx1).unwrap();
            assert_eq!(roots, vec![idx3, idx0]);

            let roots = graph.root_ancestors(idx0).unwrap();
            assert_eq!(roots, vec![]);

            let roots = graph.root_ancestors(idx3).unwrap();
            assert_eq!(roots, vec![]);
        }
        {
            let mut graph = CachedStableGraph::new();

            let idx0 = graph.add_node(&"0");
            let idx1 = graph.add_node(&"1");
            let idx2 = graph.add_node(&"2");
            let idx3 = graph.add_node(&"3");

            graph.add_edge(idx0, idx1, 200);
            graph.add_edge(idx0, idx2, 300);
            graph.add_edge(idx1, idx3, 500);

            //       0
            //      / \
            //     1   2
            //    /
            //   3

            let roots = graph.root_ancestors(idx3).unwrap();
            assert_eq!(roots, vec![idx0]);

            let roots = graph.root_ancestors(idx2).unwrap();
            assert_eq!(roots, vec![idx0]);

            let roots = graph.root_ancestors(idx1).unwrap();
            assert_eq!(roots, vec![idx0]);

            let roots = graph.root_ancestors(idx0).unwrap();
            assert_eq!(roots, vec![]);
        }
        {
            let mut graph = CachedStableGraph::new();

            let idx0 = graph.add_node(&"0");
            let idx1 = graph.add_node(&"1");
            let idx2 = graph.add_node(&"2");
            let idx3 = graph.add_node(&"3");

            graph.add_edge(idx0, idx1, 200);
            graph.add_edge(idx2, idx3, 300);
            graph.add_edge(idx1, idx3, 500);

            //       0
            //        \
            //     2   1
            //      \ /
            //       3

            let roots = graph.root_ancestors(idx3).unwrap();
            assert_eq!(roots, vec![idx0, idx2]);

            let roots = graph.root_ancestors(idx2).unwrap();
            assert_eq!(roots, vec![]);

            let roots = graph.root_ancestors(idx1).unwrap();
            assert_eq!(roots, vec![idx0]);

            let roots = graph.root_ancestors(idx0).unwrap();
            assert_eq!(roots, vec![]);
        }
        {
            let mut graph = CachedStableGraph::new();

            let idx0 = graph.add_node(&"0");
            let idx1 = graph.add_node(&"1");
            let idx2 = graph.add_node(&"2");
            let idx3 = graph.add_node(&"3");

            graph.add_edge(idx0, idx1, 200);
            graph.add_edge(idx1, idx2, 400);
            graph.add_edge(idx1, idx3, 600);

            //       0
            //       |
            //       1
            //      / \
            //     2   3

            let roots = graph.root_ancestors(idx3).unwrap();
            assert_eq!(roots, vec![idx0]);

            let roots = graph.root_ancestors(idx2).unwrap();
            assert_eq!(roots, vec![idx0]);

            let roots = graph.root_ancestors(idx1).unwrap();
            assert_eq!(roots, vec![idx0]);

            let roots = graph.root_ancestors(idx0).unwrap();
            assert_eq!(roots, vec![]);
        }
    }
}
