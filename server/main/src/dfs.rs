use petgraph::stable_graph::NodeIndex;

use crate::{graph::CachedStableGraph, merge_views::FilialTuple};

use anyhow::Result;

struct VisitCount {
    node: NodeIndex,
    touch: usize,
    children: usize,
}

/// Performs a depth-first search with duplicates
pub struct Dfs<'a> {
    stack: Vec<NodeIndex>,
    graph: &'a CachedStableGraph,
    cycle: Vec<VisitCount>,
}

impl<'a> Dfs<'a> {
    pub fn new(graph: &'a CachedStableGraph, start: NodeIndex) -> Self {
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

    fn check_for_cycle(&self, children: &[NodeIndex]) -> Result<(), error::CycleError> {
        for prev in &self.cycle {
            for child in children {
                if prev.node == *child {
                    let cycle_nodes: Vec<NodeIndex> = self.cycle.iter().map(|n| n.node).collect();
                    return Err(error::CycleError::new(&cycle_nodes, *child, self.graph));
                }
            }
        }
        Ok(())
    }
}

impl<'a> Iterator for Dfs<'a> {
    type Item = Result<FilialTuple, error::CycleError>;

    fn next(&mut self) -> Option<Result<FilialTuple, error::CycleError>> {
        let parent = self.cycle.last().map(|p| p.node);

        if let Some(node) = self.stack.pop() {
            self.cycle.push(VisitCount {
                node,
                children: self.graph.graph.edges(node).count(),
                touch: 1,
            });

            let mut children = self.graph.child_node_indexes(node);

            if !children.is_empty() {
                // sort by line number in parent
                children.sort_by(|x, y| {
                    let graph = &self.graph.graph;
                    let edge1 = graph.edge_weight(graph.find_edge(node, *x).unwrap()).unwrap();
                    let edge2 = graph.edge_weight(graph.find_edge(node, *y).unwrap()).unwrap();

                    edge2.line.cmp(&edge1.line)
                });

                match self.check_for_cycle(&children) {
                    Ok(_) => {}
                    Err(e) => return Some(Err(e)),
                };

                for child in children {
                    self.stack.push(child);
                }
            } else {
                self.reset_path_to_branch();
            }

            return Some(Ok((node, parent)));
        }
        None
    }
}

pub mod error {
    use petgraph::stable_graph::NodeIndex;

    use std::{
        error::Error as StdError,
        fmt::{Debug, Display},
        path::PathBuf,
    };

    use crate::{consts, graph::CachedStableGraph};

    use rust_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

    #[derive(Debug)]
    pub struct CycleError(Vec<PathBuf>);

    impl StdError for CycleError {}

    impl CycleError {
        pub fn new(nodes: &[NodeIndex], current_node: NodeIndex, graph: &CachedStableGraph) -> Self {
            let mut resolved_nodes: Vec<PathBuf> = nodes.iter().map(|i| graph.get_node(*i)).collect();
            resolved_nodes.push(graph.get_node(current_node));
            CycleError(resolved_nodes)
        }
    }

    impl Display for CycleError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut disp = String::new();
            disp.push_str(format!("Include cycle detected:\n{:?} imports ", self.0[0]).as_str());
            for p in &self.0[1..self.0.len() - 1] {
                disp.push_str(format!("\n{:?}, which imports ", *p).as_str());
            }
            disp.push_str(format!("\n{:?}", self.0[self.0.len() - 1]).as_str());
            f.write_str(disp.as_str())
        }
    }

    impl From<CycleError> for Diagnostic {
        fn from(e: CycleError) -> Diagnostic {
            Diagnostic {
                severity: Some(DiagnosticSeverity::ERROR),
                range: Range::new(Position::new(0, 0), Position::new(0, 500)),
                source: Some(consts::SOURCE.into()),
                message: e.into(),
                code: None,
                tags: None,
                related_information: None,
                code_description: Option::None,
                data: Option::None,
            }
        }
    }

    impl From<CycleError> for String {
        fn from(e: CycleError) -> String {
            format!("{}", e)
        }
    }
}

#[cfg(test)]
mod dfs_test {
    use std::path::PathBuf;

    use hamcrest2::prelude::*;
    use hamcrest2::{assert_that, ok};
    use petgraph::{algo::is_cyclic_directed, graph::NodeIndex};

    use crate::graph::CachedStableGraph;
    use crate::{dfs, IncludePosition};

    #[test]
    #[logging_macro::log_scope]
    fn test_graph_dfs() {
        {
            let mut graph = CachedStableGraph::new();

            let idx0 = graph.add_node(&PathBuf::from("0"));
            let idx1 = graph.add_node(&PathBuf::from("1"));
            let idx2 = graph.add_node(&PathBuf::from("2"));
            let idx3 = graph.add_node(&PathBuf::from("3"));

            graph.add_edge(idx0, idx1, IncludePosition { line: 2, start: 0, end: 0 });
            graph.add_edge(idx0, idx2, IncludePosition { line: 3, start: 0, end: 0 });
            graph.add_edge(idx1, idx3, IncludePosition { line: 5, start: 0, end: 0 });

            let dfs = dfs::Dfs::new(&graph, idx0);

            let mut collection = Vec::new();

            for i in dfs {
                assert_that!(&i, ok());
                collection.push(i.unwrap());
            }

            let nodes: Vec<NodeIndex> = collection.iter().map(|n| n.0).collect();
            let parents: Vec<Option<NodeIndex>> = collection.iter().map(|n| n.1).collect();
            //          0
            //        /  \
            //      1     2
            //     /
            //    3
            let expected_nodes = vec![idx0, idx1, idx3, idx2];

            assert_eq!(expected_nodes, nodes);

            let expected_parents = vec![None, Some(idx0), Some(idx1), Some(idx0)];

            assert_eq!(expected_parents, parents);

            assert!(!is_cyclic_directed(&graph.graph));
        }
        {
            let mut graph = CachedStableGraph::new();

            let idx0 = graph.add_node(&PathBuf::from("0"));
            let idx1 = graph.add_node(&PathBuf::from("1"));
            let idx2 = graph.add_node(&PathBuf::from("2"));
            let idx3 = graph.add_node(&PathBuf::from("3"));
            let idx4 = graph.add_node(&PathBuf::from("4"));
            let idx5 = graph.add_node(&PathBuf::from("5"));
            let idx6 = graph.add_node(&PathBuf::from("6"));
            let idx7 = graph.add_node(&PathBuf::from("7"));

            graph.add_edge(idx0, idx1, IncludePosition { line: 2, start: 0, end: 0 });
            graph.add_edge(idx0, idx2, IncludePosition { line: 3, start: 0, end: 0 });
            graph.add_edge(idx1, idx3, IncludePosition { line: 5, start: 0, end: 0 });
            graph.add_edge(idx1, idx4, IncludePosition { line: 6, start: 0, end: 0 });
            graph.add_edge(idx2, idx4, IncludePosition { line: 5, start: 0, end: 0 });
            graph.add_edge(idx2, idx5, IncludePosition { line: 4, start: 0, end: 0 });
            graph.add_edge(idx3, idx6, IncludePosition { line: 4, start: 0, end: 0 });
            graph.add_edge(idx4, idx6, IncludePosition { line: 4, start: 0, end: 0 });
            graph.add_edge(idx6, idx7, IncludePosition { line: 4, start: 0, end: 0 });

            let dfs = dfs::Dfs::new(&graph, idx0);

            let mut collection = Vec::new();

            for i in dfs {
                assert_that!(&i, ok());
                collection.push(i.unwrap());
            }

            let nodes: Vec<NodeIndex> = collection.iter().map(|n| n.0).collect();
            let parents: Vec<Option<NodeIndex>> = collection.iter().map(|n| n.1).collect();
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

            assert!(!is_cyclic_directed(&graph.graph));
        }
    }

    #[test]
    #[logging_macro::log_scope]
    fn test_graph_dfs_cycle() {
        {
            let mut graph = CachedStableGraph::new();

            let idx0 = graph.add_node(&PathBuf::from("0"));
            let idx1 = graph.add_node(&PathBuf::from("1"));
            let idx2 = graph.add_node(&PathBuf::from("2"));
            let idx3 = graph.add_node(&PathBuf::from("3"));
            let idx4 = graph.add_node(&PathBuf::from("4"));
            let idx5 = graph.add_node(&PathBuf::from("5"));
            let idx6 = graph.add_node(&PathBuf::from("6"));
            let idx7 = graph.add_node(&PathBuf::from("7"));

            graph.add_edge(idx0, idx1, IncludePosition { line: 2, start: 0, end: 0 });
            graph.add_edge(idx0, idx2, IncludePosition { line: 3, start: 0, end: 0 });
            graph.add_edge(idx1, idx3, IncludePosition { line: 5, start: 0, end: 0 });
            graph.add_edge(idx1, idx4, IncludePosition { line: 6, start: 0, end: 0 });
            graph.add_edge(idx2, idx4, IncludePosition { line: 5, start: 0, end: 0 });
            graph.add_edge(idx2, idx5, IncludePosition { line: 4, start: 0, end: 0 });
            graph.add_edge(idx3, idx6, IncludePosition { line: 4, start: 0, end: 0 });
            graph.add_edge(idx4, idx6, IncludePosition { line: 4, start: 0, end: 0 });
            graph.add_edge(idx6, idx7, IncludePosition { line: 4, start: 0, end: 0 });
            graph.add_edge(idx7, idx4, IncludePosition { line: 4, start: 0, end: 0 });

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

            assert!(is_cyclic_directed(&graph.graph));

            let next = dfs.next().unwrap();
            assert_that!(next, err());
        }
        {
            let mut graph = CachedStableGraph::new();

            let idx0 = graph.add_node(&PathBuf::from("0"));
            let idx1 = graph.add_node(&PathBuf::from("1"));

            graph.add_edge(idx0, idx1, IncludePosition { line: 2, start: 0, end: 0 });
            graph.add_edge(idx1, idx0, IncludePosition { line: 2, start: 0, end: 0 });

            let mut dfs = dfs::Dfs::new(&graph, idx1);

            println!("{:?}", dfs.next());
            println!("{:?}", dfs.next());
            println!("{:?}", dfs.next());
        }
    }
}
