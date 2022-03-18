use std::cmp::min;
use std::iter::Peekable;
use std::{
    collections::{HashMap, LinkedList, VecDeque},
    path::{Path, PathBuf},
};

use core::slice::Iter;

use petgraph::stable_graph::NodeIndex;

use crate::graph::CachedStableGraph;

/// FilialTuple represents a tuple with a parent at index 0
/// and a child at index 1. Parent can be nullable in the case of
/// the child being a top level node in the tree.
#[derive(PartialEq, Eq, Hash)]
struct FilialTuple(Option<NodeIndex>, NodeIndex);

impl From<(Option<&NodeIndex>, NodeIndex)> for FilialTuple {
    fn from(tuple: (Option<&NodeIndex>, NodeIndex)) -> Self {
        FilialTuple(tuple.0.copied(), tuple.1)
    }
}

pub fn generate_merge_list<'a>(
    nodes: &'a [(NodeIndex, Option<NodeIndex>)], sources: &'a HashMap<PathBuf, String>, graph: &'a CachedStableGraph,
) -> String {
    let mut line_directives: Vec<String> = Vec::new();

    // list of source code views onto the below sources
    let mut merge_list: LinkedList<&'a str> = LinkedList::new();

    line_directives.reserve(nodes.len() * 2);

    let mut last_offset_set: HashMap<FilialTuple, usize> = HashMap::new();

    let mut nodes_iter = nodes.iter().peekable();

    // invariant: nodes_iter always has _at least_ one element. Can't save a not-file :B
    let first = nodes_iter.next().unwrap().0;
    let first_path = graph.get_node(first);

    last_offset_set.insert(FilialTuple(None, first), 0);

    // stack to keep track of the depth first traversal
    let mut stack = VecDeque::<NodeIndex>::new();

    create_merge_views(
        &mut nodes_iter,
        &mut merge_list,
        &mut last_offset_set,
        graph,
        sources,
        &mut line_directives,
        &mut stack,
    );

    // now we add a view of the remainder of the root file
    let offset = *last_offset_set.get(&FilialTuple(None, first)).unwrap();

    let len = sources.get(&first_path).unwrap().len();
    merge_list.push_back(&sources.get(&first_path).unwrap()[min(offset, len)..]);

    let total_len = merge_list.iter().fold(0, |a, b| a + b.len());

    let mut merged = String::with_capacity(total_len);
    for slice in merge_list {
        merged.push_str(slice);
    }

    merged
}

fn create_merge_views<'a>(
    nodes: &mut Peekable<Iter<(NodeIndex, Option<NodeIndex>)>>, merge_list: &mut LinkedList<&'a str>,
    last_offset_set: &mut HashMap<FilialTuple, usize>, graph: &'a CachedStableGraph, sources: &'a HashMap<PathBuf, String>,
    line_directives: &mut Vec<String>, stack: &mut VecDeque<NodeIndex>,
) {
    loop {
        let n = match nodes.next() {
            Some(n) => n,
            None => return,
        };

        // invariant: never None as only the first element in `nodes` should have a None, which is popped off in the calling function
        let parent = n.1.unwrap();
        let child = n.0;
        let edge = graph.get_edge_meta(parent, child);
        let parent_path = graph.get_node(parent).clone();
        let child_path = graph.get_node(child).clone();

        let parent_source = sources.get(&parent_path).unwrap();
        let (char_for_line, char_following_line) = char_offset_for_line(edge.line, parent_source);

        let offset = *last_offset_set
            .insert((stack.back(), parent).into(), char_following_line)
            .get_or_insert(0);
        merge_list.push_back(&parent_source[offset..char_for_line]);
        add_opening_line_directive(&child_path, merge_list, line_directives);

        match nodes.peek() {
            Some(next) => {
                let next = *next;
                // if the next pair's parent is not a child of the current pair, we dump the rest of this childs source
                if next.1.unwrap() != child {
                    let child_source = sources.get(&child_path).unwrap();
                    // if ends in \n\n, we want to exclude the last \n for some reason. Ask optilad
                    let offset = {
                        match child_source.ends_with('\n') {
                            true => child_source.len() - 1,
                            false => child_source.len(),
                        }
                    };
                    merge_list.push_back(&child_source[..offset]);
                    last_offset_set.insert(FilialTuple(Some(parent), child), 0);
                    // +2 because edge.line is 0 indexed but #line is 1 indexed and references the *following* line
                    add_closing_line_directive(edge.line + 2, &parent_path, merge_list, line_directives);
                    // if the next pair's parent is not the current pair's parent, we need to bubble up
                    if stack.contains(&next.1.unwrap()) {
                        return;
                    }
                    continue;
                }

                stack.push_back(parent);
                create_merge_views(nodes, merge_list, last_offset_set, graph, sources, line_directives, stack);
                stack.pop_back();

                let offset = *last_offset_set.get(&FilialTuple(Some(parent), child)).unwrap();
                let child_source = sources.get(&child_path).unwrap();
                // this evaluates to false once the file contents have been exhausted aka offset = child_source.len() + 1
                let end_offset = {
                    match child_source.ends_with('\n') {
                        true => 1,  /* child_source.len()-1 */
                        false => 0, /* child_source.len() */
                    }
                };
                if offset < child_source.len() - end_offset {
                    // if ends in \n\n, we want to exclude the last \n for some reason. Ask optilad
                    merge_list.push_back(&child_source[offset../* std::cmp::max( */child_source.len()-end_offset/* , offset) */]);
                    last_offset_set.insert(FilialTuple(Some(parent), child), 0);
                }

                // +2 because edge.line is 0 indexed but #line is 1 indexed and references the *following* line
                add_closing_line_directive(edge.line + 2, &parent_path, merge_list, line_directives);

                // we need to check the next item at the point of original return further down the callstack
                if nodes.peek().is_some() && stack.contains(&nodes.peek().unwrap().1.unwrap()) {
                    return;
                }
            }
            None => {
                let child_source = sources.get(&child_path).unwrap();
                // if ends in \n\n, we want to exclude the last \n for some reason. Ask optilad
                let offset = {
                    match child_source.ends_with('\n') {
                        true => child_source.len() - 1,
                        false => child_source.len(),
                    }
                };
                merge_list.push_back(&child_source[..offset]);
                last_offset_set.insert(FilialTuple(Some(parent), child), 0);
                // +2 because edge.line is 0 indexed but #line is 1 indexed and references the *following* line
                add_closing_line_directive(edge.line + 2, &parent_path, merge_list, line_directives);
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
            char_following_line += line.len() + 1;
            break;
        }
        char_for_line += line.len() + 1;
        char_following_line = char_for_line;
    }
    (char_for_line, char_following_line)
}

fn add_opening_line_directive(path: &Path, merge_list: &mut LinkedList<&str>, line_directives: &mut Vec<String>) {
    let line_directive = format!("#line 1 \"{}\"\n", path.to_str().unwrap().replace('\\', "\\\\"));
    line_directives.push(line_directive);
    unsafe_get_and_insert(merge_list, line_directives);
}

fn add_closing_line_directive(line: usize, path: &Path, merge_list: &mut LinkedList<&str>, line_directives: &mut Vec<String>) {
    // Optifine doesn't seem to add a leading newline if the previous line was a #line directive
    let line_directive = if let Some(l) = merge_list.back() {
        if l.trim().starts_with("#line") {
            format!("#line {} \"{}\"\n", line, path.to_str().unwrap().replace('\\', "\\\\"))
        } else {
            format!("\n#line {} \"{}\"\n", line, path.to_str().unwrap().replace('\\', "\\\\"))
        }
    } else {
        format!("\n#line {} \"{}\"\n", line, path.to_str().unwrap().replace('\\', "\\\\"))
    };

    line_directives.push(line_directive);
    unsafe_get_and_insert(merge_list, line_directives);
}

fn unsafe_get_and_insert(merge_list: &mut LinkedList<&str>, line_directives: &[String]) {
    // :^)
    unsafe {
        let vec_ptr_offset = line_directives.as_ptr().add(line_directives.len() - 1);
        merge_list.push_back(&vec_ptr_offset.as_ref().unwrap()[..]);
    }
}
