use std::collections::{HashMap, LinkedList};
use std::cell::RefCell;
use std::iter::Peekable;

use core::slice::Iter;

use petgraph::stable_graph::NodeIndex;

use crate::graph::CachedStableGraph;

pub struct MergeViewGenerator<'a> {
    sources: &'a mut HashMap<String, String>,
    graph: &'a CachedStableGraph,
    line_directives: RefCell<Vec<String>>,
}

impl <'a> MergeViewGenerator<'a> {
    pub fn new(sources: &'a mut HashMap<String, String>, graph: &'a CachedStableGraph) -> Self {
        Self { sources, graph, line_directives: RefCell::new(Vec::new()) }
    }

    pub fn generate_merge_list(&'a mut self, nodes: &'a [(NodeIndex, Option<NodeIndex>)]) -> LinkedList<&'a str> {
        // list of source code views onto the below sources
        let mut merge_list: LinkedList<&'a str> = LinkedList::new();

        self.line_directives.borrow_mut().reserve(nodes.len() * 2);

        let mut last_offset_set: HashMap<String, usize> = HashMap::new();
        let mut last_offset: Vec<String> = Vec::new();

        let mut nodes_iter = nodes.iter().peekable();

        let first = nodes_iter.next().unwrap().0;
        let first_path = self.graph.get_node(first).clone();
        
        self.create_merge_views(nodes_iter, &mut merge_list, &mut last_offset_set, &mut last_offset);

        // now we add a view of the remainder of the root file
        let offset = *last_offset_set.get(&first_path).unwrap();
        merge_list.push_back(&self.sources.get(&first_path).unwrap().as_str()[offset..]);

        merge_list
    }

    fn create_merge_views(
        &'a self,
        mut nodes: Peekable<Iter<(NodeIndex, Option<NodeIndex>)>>,
        merge_list: &mut LinkedList<&'a str>,
        last_offset_set: &mut HashMap<String, usize>,
        last_offset: &mut Vec<String>,
    ) {
        let n = match nodes.next() {
            Some(n) => n,
            None => return,
        };

        let parent = n.1.unwrap();
        let child = n.0;
        let edge = self.graph.get_edge_meta(parent, child);
        let parent_path = self.graph.get_node(parent).clone();
        let child_path = self.graph.get_node(child).clone();

        if !last_offset_set.contains_key(&parent_path) {
            last_offset.push(parent_path.clone());
        }

        let source = self.sources.get(&parent_path).unwrap();
        let mut char_for_line: usize = 0;
        let mut char_following_line: usize = 0;
        for (n, line) in source.as_str().lines().enumerate() {
            if n == edge.line {
                char_following_line += line.len()+1;
                break;
            }
            char_for_line += line.len()+1;
            char_following_line = char_for_line;
        }
        
        let offset = *last_offset_set.insert(parent_path.clone(), char_following_line).get_or_insert(0);
        merge_list.push_back(&source.as_str()[offset..char_for_line]);
        merge_list.push_back(&"#line 1\n"[..]);

        match nodes.peek() {
            Some(next) => {
                let next = *next;
                // if the next element is not a child of this element, we dump the rest of this elements source
                if next.1.unwrap() != child {
                    let source = self.sources.get(&child_path).unwrap();
                    merge_list.push_back(&source.as_str()[..]);
                    // +2 because edge.line is 0 indexed but #line is 1 indexed and references the *following* line
                    self.add_line_directive(edge.line+2, merge_list);
                }
                self.create_merge_views(nodes, merge_list, last_offset_set, last_offset);

                if next.1.unwrap() == child {
                    let offset = *last_offset_set.get(&child_path).unwrap();
                    let source = self.sources.get(&child_path).unwrap();
                    if offset <= source.len() {
                        merge_list.push_back(&source.as_str()[offset..]);
                    }

                    // +2 because edge.line is 0 indexed but #line is 1 indexed and references the *following* line
                    self.add_line_directive(edge.line+2, merge_list);
                }
            },
            None => {
                let source = self.sources.get(&child_path).unwrap();
                merge_list.push_back(&source.as_str()[..]);
                // +2 because edge.line is 0 indexed but #line is 1 indexed and references the *following* line
                self.add_line_directive(edge.line+2, merge_list);
            }
        }
    }

    fn add_line_directive(&self, line: usize, merge_list: &mut LinkedList<&str>) {
        // Optifine doesn't seem to add a leading newline if the previous line was
        // a #line directive
        let line_directive = if let Some(l) = merge_list.back() {
            if l.starts_with("\n#line") {
                format!("#line {}\n", line)
            } else {
                format!("\n#line {}\n", line)
            }
        } else {
            format!("\n#line {}\n", line)
        };
        
        self.line_directives.borrow_mut().push(line_directive);
        unsafe {
            self.unsafe_get_and_insert(merge_list);
        }
    }

    unsafe fn unsafe_get_and_insert(&self, merge_list: &mut LinkedList<&str>) {
        // :^)
        let vec_ptr_offset = self.line_directives.borrow().as_ptr().add(self.line_directives.borrow().len()-1);
        merge_list.push_back(&vec_ptr_offset.as_ref().unwrap().as_str()[..]);
    }
}