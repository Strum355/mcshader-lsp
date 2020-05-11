use petgraph::stable_graph::StableDiGraph;
use petgraph::stable_graph::NodeIndex;
use std::collections::HashMap;

pub struct CachedStableGraph {
    pub graph: StableDiGraph<String, String>,
    cache: HashMap<String, NodeIndex>,
}

impl CachedStableGraph {
    pub fn new() -> CachedStableGraph {
        CachedStableGraph{
            graph: StableDiGraph::new(),
            cache: HashMap::new(),
        }
    }

    pub fn find_node(&mut self, name: String) -> Option<NodeIndex> {
        match self.cache.get(&name) {
            Some(n) => Some(*n),
            None => {
                let n = self.graph.node_indices().find(|n| self.graph[*n] == name);
                if n.is_some() {
                    self.cache.insert(name, n.unwrap());
                }
                n
            }
        }
    }

    pub fn add_node(&mut self, name: String) -> NodeIndex {
        let idx = self.graph.add_node(name.clone());
        self.cache.insert(name, idx);
        idx
    }
}