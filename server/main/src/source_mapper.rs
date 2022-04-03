use std::{collections::HashMap, fmt::Display};

use petgraph::graph::NodeIndex;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceNum(usize);

impl Display for SourceNum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{}", self.0).as_str())
    }
}

impl From<usize> for SourceNum {
    fn from(val: usize) -> Self {
        SourceNum(val)
    }
}

// Maps from a graph node index to a virtual OpenGL 
// source number (for when building the merged source view),
// and in reverse (for when mapping from GLSL error source numbers to their source path).
// What is a source number: https://community.khronos.org/t/what-is-source-string-number/70976
pub struct SourceMapper {
    next: SourceNum,
    mapping: HashMap<NodeIndex, SourceNum>,
    reverse_mapping: Vec<NodeIndex>,
}

impl SourceMapper {
    pub fn new(capacity: usize) -> Self {
        SourceMapper {
            next: SourceNum(0),
            mapping: HashMap::with_capacity(capacity),
            reverse_mapping: Vec::with_capacity(capacity),
        }
    }

    pub fn get_num(&mut self, node: NodeIndex) -> SourceNum {
        let num = &*self.mapping.entry(node).or_insert_with(|| {
            let next = self.next;
            self.next.0 += 1;
            self.reverse_mapping.push(node);
            next
        });
        *num
    }

    pub fn get_node(&self, num: SourceNum) -> NodeIndex {
        self.reverse_mapping[num.0]
    }
}
