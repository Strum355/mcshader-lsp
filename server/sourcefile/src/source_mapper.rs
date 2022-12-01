use std::{cmp::Eq, collections::HashMap, fmt::Display, hash::Hash};

pub const ROOT_SOURCE_NUM: SourceNum = SourceNum(0);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceNum(usize);

impl Display for SourceNum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.to_string().as_str())
    }
}

impl From<usize> for SourceNum {
    fn from(val: usize) -> Self {
        SourceNum(val)
    }
}

// Maps from a key to a virtual OpenGL source number (for when building the merged source view),
// and in reverse (for when mapping from GLSL error source numbers to their source path).
// What is a source number: https://community.khronos.org/t/what-is-source-string-number/70976
pub struct SourceMapper<T> {
    next: SourceNum,
    mapping: HashMap<T, SourceNum>,
    reverse_mapping: Vec<T>,
}

impl<T> SourceMapper<T>
where
    T: Eq + Hash + Clone,
{
    pub fn new(capacity: usize) -> Self {
        SourceMapper {
            next: SourceNum(0),
            mapping: HashMap::with_capacity(capacity),
            reverse_mapping: Vec::with_capacity(capacity),
        }
    }

    pub fn get_num(&mut self, node: &T) -> SourceNum {
        let num = &*self.mapping.entry(node.clone()).or_insert_with(|| {
            let next = self.next;
            self.next.0 += 1;
            self.reverse_mapping.push(node.clone());
            next
        });
        *num
    }

    pub fn get_node(&self, num: SourceNum) -> &T {
        &self.reverse_mapping[num.0]
    }
}
