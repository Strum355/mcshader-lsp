use petgraph::stable_graph::NodeIndex;

use thiserror::Error;

use crate::graph::CachedStableGraph;

use anyhow::{Result, Error};
use std::fmt::{Debug, Display};

/// Performs a depth-first search with duplicates 
pub struct Dfs<'a> {
    stack: Vec<NodeIndex>,
    graph: &'a CachedStableGraph,
    cycle: Vec<NodeIndex>
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
        while let Some(par) = self.cycle.pop() {
            if self.graph.graph.edges(par).count() > 0 {
                self.cycle.pop();
                break;
            }
        }
    }

    fn check_for_cycle(&self, children: &[NodeIndex]) -> Result<()> {
        for prev in &self.cycle {
            for child in children {
                if *prev == *child {
                    return Err(
                        Error::new(
                            CycleError::new(&self.cycle, *child, self.graph)
                        )
                    );
                }
            }
        }
        Ok(())
    }   
}

impl <'a> Iterator for Dfs<'a> {
    type Item = Result<NodeIndex>;

    fn next(&mut self) -> Option<Result<NodeIndex>> {
        if let Some(node) = self.stack.pop() {
            self.cycle.push(node);

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
            return Some(Ok(node));
        }
        None
    }
}

#[derive(Debug, Error)]
pub struct CycleError(Vec<String>);

impl CycleError {
    fn new(nodes: &[NodeIndex], current_node: NodeIndex, graph: &CachedStableGraph) -> Self {
        let mut resolved_nodes: Vec<String> = nodes.iter().map(|i| graph.get_node(*i).clone()).collect();
        resolved_nodes.push(graph.get_node(current_node).clone());
        CycleError(resolved_nodes)
    }
}

impl Display for CycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        eprintln!("cycle path {:?}", self.0);
        let mut disp = String::new();
        disp.push_str(format!("Include cycle detected:\n{} imports ", self.0[0]).as_str());
        for p in &self.0[1..self.0.len()-1] {
            disp.push_str(format!("\n{}, which imports ", *p).as_str());
        }
        disp.push_str(format!("\n{}", self.0[self.0.len()-1]).as_str());
        f.write_str(disp.as_str())
    }
}

impl Into<String> for CycleError {
    fn into(self) -> String {
        format!("{}", self)
    }
    
}