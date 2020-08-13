use petgraph::stable_graph::NodeIndex;

use crate::graph::CachedStableGraph;

use anyhow::{Result, Error};

struct VisitCount {
    node: NodeIndex,
    touch: usize,
    children: usize,
}

/// Performs a depth-first search with duplicates 
pub struct Dfs<'a> {
    stack: Vec<NodeIndex>,
    graph: &'a CachedStableGraph,
    cycle: Vec<VisitCount>
}

impl <'a> Dfs<'a> {
    pub fn new(graph: &'a CachedStableGraph, start: NodeIndex) -> Self {
        Dfs {
            stack: vec![start],
            graph,
            cycle: Vec::new()
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
                    return Err(
                        error::CycleError::new(&cycle_nodes, *child, self.graph)
                    );
                }
            }
        }
        Ok(())
    }   
}

impl <'a> Iterator for Dfs<'a> {
    type Item = Result<(NodeIndex, Option<NodeIndex>), error::CycleError>;

    fn next(&mut self) -> Option<Result<(NodeIndex, Option<NodeIndex>), error::CycleError>> {
        let parent = match self.cycle.last() {
            Some(p) => Some(p.node),
            None => None,
        };

        if let Some(node) = self.stack.pop() {
            self.cycle.push(VisitCount{
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

    use thiserror::Error;

    use std::fmt::{Debug, Display};

    use crate::{graph::CachedStableGraph, consts};

    use rust_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

    #[derive(Debug, Error)]
    pub struct CycleError(Vec<String>);
    
    impl CycleError {
        pub fn new(nodes: &[NodeIndex], current_node: NodeIndex, graph: &CachedStableGraph) -> Self {
            let mut resolved_nodes: Vec<String> = nodes.iter().map(|i| graph.get_node(*i).clone()).collect();
            resolved_nodes.push(graph.get_node(current_node).clone());
            CycleError(resolved_nodes)
        }
    }
    
    impl Display for CycleError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut disp = String::new();
            disp.push_str(format!("Include cycle detected:\n{} imports ", self.0[0]).as_str());
            for p in &self.0[1..self.0.len()-1] {
                disp.push_str(format!("\n{}, which imports ", *p).as_str());
            }
            disp.push_str(format!("\n{}", self.0[self.0.len()-1]).as_str());
            f.write_str(disp.as_str())
        }
    }

    impl Into<Diagnostic> for CycleError {
        fn into(self) -> Diagnostic {
            Diagnostic{
                severity: Some(DiagnosticSeverity::Error),
                range: Range::new(Position::new(0, 0), Position::new(0, 500)),
                source: Some(consts::SOURCE.into()),
                message: self.into(),
                code: None,
                tags: None,
                related_information: None,
            }
        }
    }
    
    impl Into<String> for CycleError {
        fn into(self) -> String {
            format!("{}", self)
        }
    }
}