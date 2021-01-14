use std::collections::{HashMap, LinkedList, VecDeque};
use std::iter::Peekable;

use core::slice::Iter;

use petgraph::stable_graph::NodeIndex;

use crate::graph::CachedStableGraph;

pub fn generate_merge_list<'a>(
    nodes: &'a [(NodeIndex, Option<NodeIndex>)],
    sources: &'a HashMap<String, String>, 
    graph: &'a CachedStableGraph
) -> String {
    let mut line_directives: Vec<String> = Vec::new();

    // list of source code views onto the below sources
    let mut merge_list: LinkedList<&'a str> = LinkedList::new();

    line_directives.reserve(nodes.len() * 2);

    let mut last_offset_set: HashMap<String, usize> = HashMap::new();

    let mut nodes_iter = nodes.iter().peekable();

    let first = nodes_iter.next().unwrap().0;
    let first_path = graph.get_node(first).clone();

    last_offset_set.insert(first_path.clone(), 0);

    // stack to keep track of the depth first traversal
    let mut stack = VecDeque::<NodeIndex>::new();

    create_merge_views(&mut nodes_iter, &mut merge_list, &mut last_offset_set, graph, sources, &mut line_directives, &mut stack);

    // now we add a view of the remainder of the root file
    let offset = *last_offset_set.get(&first_path).unwrap();
    merge_list.push_back(&sources.get(&first_path).unwrap()[offset..]);

    let total_len = merge_list.iter().fold(0, |a, b| {
        let a  = a + (*b).len();
        a
    });

    let mut merged = String::with_capacity(total_len);
    for slice in merge_list {
        merged.push_str(slice);
    }

    merged
}

fn create_merge_views<'a>(
    nodes: &mut Peekable<Iter<(NodeIndex, Option<NodeIndex>)>>,
    merge_list: &mut LinkedList<&'a str>,
    last_offset_set: &mut HashMap<String, usize>,
    graph: &'a CachedStableGraph,
    sources: &'a HashMap<String, String>,
    line_directives: &mut Vec<String>,
    stack: &mut VecDeque<NodeIndex>,
) {
    
    loop {
        let n = match nodes.next() {
            Some(n) => n,
            None => return,
        };
    
        
        let parent = n.1.unwrap();
        let child = n.0;
        let edge = graph.get_edge_meta(parent, child);
        let parent_path = graph.get_node(parent).clone();
        let child_path = graph.get_node(child).clone();
        
        let parent_source = sources.get(&parent_path).unwrap();
        let (char_for_line, char_following_line) = char_offset_for_line(edge.line, parent_source);
        
        let offset = *last_offset_set.insert(parent_path.clone(), char_following_line).get_or_insert(0);
        //eprintln!("offset={} char_for_line={} len={} path={}", offset, char_for_line, parent_source.len(), parent_path);
        merge_list.push_back(&parent_source[offset..char_for_line]);
        add_opening_line_directive(&child_path, merge_list, line_directives);

        match nodes.peek() {
            Some(next) => {
                let next = *next;
                // if the next pair's parent is not a child of the current pair, we dump the rest of this childs source
                if next.1.unwrap() != child {
                    let child_source = sources.get(&child_path).unwrap();
                    merge_list.push_back(&child_source[..]);
                    // +2 because edge.line is 0 indexed but #line is 1 indexed and references the *following* line
                    add_closing_line_directive(edge.line+2, &parent_path, merge_list, line_directives);
                    // if the next pair's parent is not the current pair's parent, we need to bubble up
                    if stack.contains(&next.1.unwrap()) {
                        return;
                    }
                    continue;
                }
                
                stack.push_back(parent);
                create_merge_views(nodes, merge_list, last_offset_set, graph, sources, line_directives, stack);
                stack.pop_back();

                if next.1.unwrap() == child {
                    let offset = *last_offset_set.get(&child_path).unwrap();
                    let child_source = sources.get(&child_path).unwrap();
                    if offset <= child_source.len() {
                        merge_list.push_back(&child_source[offset..]);
                    }

                    // +2 because edge.line is 0 indexed but #line is 1 indexed and references the *following* line
                    add_closing_line_directive(edge.line+2, &parent_path, merge_list, line_directives);
                }
            },
            None => {
                let child_source = sources.get(&child_path).unwrap();
                merge_list.push_back(&child_source[..]);
                // +2 because edge.line is 0 indexed but #line is 1 indexed and references the *following* line
                add_closing_line_directive(edge.line+2, &parent_path, merge_list, line_directives);
            }
        }
    }
}

// returns the character offset + 1 of the end of line number `line` and the character
// offset + 1 for the end of the line after the previous one
fn char_offset_for_line(line_num: usize, source: &str) -> (usize, usize) {
    let mut char_for_line: usize = 0;
    let mut char_following_line: usize = 0;
    for (n, line) in source.lines().enumerate() {
        if n == line_num {
            char_following_line += line.len()+1;
            break;
        }
        char_for_line += line.len()+1;
        char_following_line = char_for_line;
    }
    (char_for_line, char_following_line)
}

fn add_opening_line_directive(path: &str, merge_list: &mut LinkedList<&str>, line_directives: &mut Vec<String>) {
    let line_directive = format!("#line 1 \"{}\"\n", path);
    line_directives.push(line_directive);
    unsafe_get_and_insert(merge_list, line_directives);
}

fn add_closing_line_directive(line: usize, path: &str, merge_list: &mut LinkedList<&str>, line_directives: &mut Vec<String>) {
    // Optifine doesn't seem to add a leading newline if the previous line was a #line directive
    let line_directive = if let Some(l) = merge_list.back() {
        if l.starts_with("\n#line") {
            format!("#line {} \"{}\"\n", line, path)
        } else {
            format!("\n#line {} \"{}\"\n", line, path)
        }
    } else {
        format!("\n#line {} \"{}\"\n", line, path)
    };
    
    line_directives.push(line_directive);
    unsafe_get_and_insert(merge_list, line_directives);
}

fn unsafe_get_and_insert(merge_list: &mut LinkedList<&str>, line_directives: &Vec<String>) {
    // :^)
    unsafe {
        let vec_ptr_offset = line_directives.as_ptr().add(line_directives.len()-1);
        merge_list.push_back(&vec_ptr_offset.as_ref().unwrap()[..]);
    }
}