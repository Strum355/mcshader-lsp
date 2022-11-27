use crate::{graph::CachedStableGraph, FilialTuple};
use petgraph::stable_graph::{NodeIndex, StableDiGraph};
use std::{fmt::Debug, hash::Hash};

use anyhow::Result;

struct VisitCount {
    node: NodeIndex,
    // how many times we have backtracked to this node
    // after exhausting a DFS along one of this node's
    // outgoing edges
    touch: usize,
    // how many times we have to backtrack to this node
    // after exhausting a DFS along one of this node's
    // outgoing edges before we backtrack to the parent
    // node of this node that we came from during the
    // traversal. Aint that a mouthful.
    children: usize,
}

/// Performs a depth-first search with duplicates
pub struct Dfs<'a, K, V>
where
    K: Hash + Clone + Display + Eq + Debug,
    V: Ord + Copy,
{
    graph: &'a CachedStableGraph<K, V>,
    // TODO: how can we collapse these
    stack: Vec<NodeIndex>,
    cycle: Vec<VisitCount>,
}

impl<'a, K, V> Dfs<'a, K, V>
where
    K: Hash + Clone + Display + Eq + Debug,
    V: Ord + Copy,
{
    pub fn new(graph: &'a CachedStableGraph<K, V>, start: NodeIndex) -> Self {
        Dfs {
            stack: vec![start],
            graph,
            cycle: Vec::new(),
        }
    }

    fn reset_path_to_branch(&mut self) {
        while let Some(par) = self.cycle.last_mut() {
            par.touch += 1;
            if par.touch > par.children {
                self.cycle.pop();
            } else {
                break;
            }
        }
    }

    fn check_for_cycle(&self, children: &[NodeIndex]) -> Result<(), CycleError<K>> {
        for prev in &self.cycle {
            for child in children {
                if prev.node == *child {
                    let cycle_nodes: Vec<NodeIndex> = self.cycle.iter().map(|n| n.node).collect();
                    return Err(CycleError::new(&cycle_nodes, *child, self.graph));
                }
            }
        }
        Ok(())
    }
}

impl<'a, K, V> Iterator for Dfs<'a, K, V>
where
    K: Hash + Clone + Display + Eq + Debug,
    V: Ord + Copy,
{
    type Item = Result<FilialTuple<NodeIndex>, CycleError<K>>;

    fn next(&mut self) -> Option<Result<FilialTuple<NodeIndex>, CycleError<K>>> {
        let parent = self.cycle.last().map(|p| p.node);

        if let Some(child) = self.stack.pop() {
            self.cycle.push(VisitCount {
                node: child,
                children: Into::<&StableDiGraph<K, V>>::into(self.graph).edges(child).count(),
                touch: 1,
            });

            let children: Vec<_> = self.graph.get_all_edges_from(child).rev().collect();

            if !children.is_empty() {
                let child_nodes: Vec<_> = children.iter().map(|(n, _)| n).copied().collect();
                if let Err(e) = self.check_for_cycle(&child_nodes) {
                    return Some(Err(e));
                }

                for child in children {
                    self.stack.push(child.0);
                }
            } else {
                self.reset_path_to_branch();
            }

            return Some(Ok(FilialTuple { child, parent }));
        }
        None
    }
}

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

use std::{error::Error as StdError, fmt::Display};

#[derive(Debug)]
// TODO: how can we include the line-of-import
pub struct CycleError<K>(Vec<K>);

impl<K> StdError for CycleError<K> where K: Display + Debug {}

impl<K> CycleError<K>
where
    K: Hash + Clone + Eq + Debug,
{
    pub fn new<V>(nodes: &[NodeIndex], current_node: NodeIndex, graph: &CachedStableGraph<K, V>) -> Self
    where
        V: Ord + Copy,
    {
        let mut resolved_nodes: Vec<K> = nodes.iter().map(|i| graph[*i].clone()).collect();
        resolved_nodes.push(graph[current_node].clone());
        CycleError(resolved_nodes.into_iter().collect())
    }
}

impl<K: Display> Display for CycleError<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut disp = String::new();
        disp.push_str(format!("Include cycle detected:\n{} imports ", self.0[0]).as_str());
        for p in &self.0[1..self.0.len() - 1] {
            disp.push_str(&format!("\n{}, which imports ", *p));
        }
        disp.push_str(&format!("\n{}", self.0[self.0.len() - 1]));
        f.write_str(disp.as_str())
    }
}

impl<K: Display> From<CycleError<K>> for Diagnostic {
    fn from(e: CycleError<K>) -> Diagnostic {
        Diagnostic {
            severity: Some(DiagnosticSeverity::ERROR),
            range: Range::new(Position::new(0, 0), Position::new(0, 500)),
            source: Some("mcglsl".into()),
            message: e.into(),
            code: None,
            tags: None,
            related_information: None,
            code_description: Option::None,
            data: Option::None,
        }
    }
}

impl<K: Display> From<CycleError<K>> for String {
    fn from(e: CycleError<K>) -> String {
        format!("{}", e)
    }
}

#[cfg(test)]
mod dfs_test {
    use hamcrest2::prelude::*;
    use hamcrest2::{assert_that, ok};
    use petgraph::stable_graph::StableDiGraph;
    use petgraph::{algo::is_cyclic_directed, graph::NodeIndex};

    use crate::dfs;
    use crate::graph::CachedStableGraph;

    #[test]
    #[logging_macro::scope]
    fn test_graph_dfs() {
        {
            let mut graph = CachedStableGraph::new();

            let idx0 = graph.add_node(&"0".to_string());
            let idx1 = graph.add_node(&"1".to_string());
            let idx2 = graph.add_node(&"2".to_string());
            let idx3 = graph.add_node(&"3".to_string());

            graph.add_edge(idx0, idx1, 2);
            graph.add_edge(idx0, idx2, 3);
            graph.add_edge(idx1, idx3, 5);

            let dfs = dfs::Dfs::new(&graph, idx0);

            let mut collection = Vec::new();

            for i in dfs {
                assert_that!(&i, ok());
                collection.push(i.unwrap());
            }

            let nodes: Vec<NodeIndex> = collection.iter().map(|n| n.child).collect();
            let parents: Vec<Option<NodeIndex>> = collection.iter().map(|n| n.parent).collect();
            //          0
            //        /  \
            //      1     2
            //     /
            //    3
            let expected_nodes = vec![idx0, idx1, idx3, idx2];

            assert_eq!(expected_nodes, nodes);

            let expected_parents = vec![None, Some(idx0), Some(idx1), Some(idx0)];

            assert_eq!(expected_parents, parents);

            assert!(!is_cyclic_directed(Into::<&StableDiGraph<_, _>>::into(&graph)));
        }
        {
            let mut graph = CachedStableGraph::new();

            let idx0 = graph.add_node(&"0".to_string());
            let idx1 = graph.add_node(&"1".to_string());
            let idx2 = graph.add_node(&"2".to_string());
            let idx3 = graph.add_node(&"3".to_string());
            let idx4 = graph.add_node(&"4".to_string());
            let idx5 = graph.add_node(&"5".to_string());
            let idx6 = graph.add_node(&"6".to_string());
            let idx7 = graph.add_node(&"7".to_string());

            graph.add_edge(idx0, idx1, 2);
            graph.add_edge(idx0, idx2, 3);
            graph.add_edge(idx1, idx3, 5);
            graph.add_edge(idx1, idx4, 6);
            graph.add_edge(idx2, idx4, 5);
            graph.add_edge(idx2, idx5, 4);
            graph.add_edge(idx3, idx6, 4);
            graph.add_edge(idx4, idx6, 4);
            graph.add_edge(idx6, idx7, 4);

            let dfs = dfs::Dfs::new(&graph, idx0);

            let mut collection = Vec::new();

            for i in dfs {
                assert_that!(&i, ok());
                collection.push(i.unwrap());
            }

            let nodes: Vec<NodeIndex> = collection.iter().map(|n| n.child).collect();
            let parents: Vec<Option<NodeIndex>> = collection.iter().map(|n| n.parent).collect();
            //          0
            //        /  \
            //      1     2
            //     / \   / \
            //    3    4    5
            //     \ /
            //      6 - 7
            let expected_nodes = vec![idx0, idx1, idx3, idx6, idx7, idx4, idx6, idx7, idx2, idx5, idx4, idx6, idx7];

            assert_eq!(expected_nodes, nodes);

            let expected_parents = vec![
                None,
                Some(idx0),
                Some(idx1),
                Some(idx3),
                Some(idx6),
                Some(idx1),
                Some(idx4),
                Some(idx6),
                Some(idx0),
                Some(idx2),
                Some(idx2),
                Some(idx4),
                Some(idx6),
            ];

            assert_eq!(expected_parents, parents);

            assert!(!is_cyclic_directed(Into::<&StableDiGraph<_, _>>::into(&graph)));
        }
    }

    #[test]
    #[logging_macro::scope]
    fn test_graph_dfs_cycle() {
        {
            let mut graph = CachedStableGraph::new();

            let idx0 = graph.add_node(&"0".to_string());
            let idx1 = graph.add_node(&"1".to_string());
            let idx2 = graph.add_node(&"2".to_string());
            let idx3 = graph.add_node(&"3".to_string());
            let idx4 = graph.add_node(&"4".to_string());
            let idx5 = graph.add_node(&"5".to_string());
            let idx6 = graph.add_node(&"6".to_string());
            let idx7 = graph.add_node(&"7".to_string());

            graph.add_edge(idx0, idx1, 2);
            graph.add_edge(idx0, idx2, 3);
            graph.add_edge(idx1, idx3, 5);
            graph.add_edge(idx1, idx4, 6);
            graph.add_edge(idx2, idx4, 5);
            graph.add_edge(idx2, idx5, 4);
            graph.add_edge(idx3, idx6, 4);
            graph.add_edge(idx4, idx6, 4);
            graph.add_edge(idx6, idx7, 4);
            graph.add_edge(idx7, idx4, 4);

            let mut dfs = dfs::Dfs::new(&graph, idx0);

            for _ in 0..5 {
                if let Some(i) = dfs.next() {
                    assert_that!(&i, ok());
                }
            }

            //          0
            //        /  \
            //      1     2
            //     / \   / \
            //    3    4    5
            //     \ /  \
            //      6 - 7

            let next = dfs.next().unwrap();
            assert_that!(next, err());

            assert!(is_cyclic_directed(Into::<&StableDiGraph<_, _>>::into(&graph)));
        }
        {
            let mut graph = CachedStableGraph::new();

            let idx0 = graph.add_node(&"0".to_string());
            let idx1 = graph.add_node(&"1".to_string());

            graph.add_edge(idx0, idx1, 2);
            graph.add_edge(idx1, idx0, 2);

            let mut dfs = dfs::Dfs::new(&graph, idx1);

            println!("{:?}", dfs.next());
            println!("{:?}", dfs.next());
            println!("{:?}", dfs.next());
        }
    }
}
