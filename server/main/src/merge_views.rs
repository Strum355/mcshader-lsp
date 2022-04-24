use std::cmp::min;
use std::iter::Peekable;
use std::{
    collections::{HashMap, LinkedList, VecDeque},
    path::{Path, PathBuf},
};

use core::slice::Iter;

use petgraph::stable_graph::NodeIndex;
use slog_scope::debug;

use crate::graph::CachedStableGraph;
use crate::source_mapper::SourceMapper;
use crate::IncludePosition;

/// FilialTuple represents a tuple (not really) of a child and any legitimate
/// parent. Parent can be nullable in the case of the child being a top level
/// node in the tree.
#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct FilialTuple {
    pub child: NodeIndex,
    pub parent: Option<NodeIndex>,
}

/// Merges the source strings according to the nodes comprising a tree of imports into a GLSL source string
/// that can be handed off to the GLSL compiler.
pub struct MergeViewBuilder<'a> {
    nodes: &'a [FilialTuple],
    nodes_peeker: Peekable<Iter<'a, FilialTuple>>,

    sources: &'a HashMap<PathBuf, String>,
    graph: &'a CachedStableGraph,
    source_mapper: &'a mut SourceMapper,

    // holds the offset into the child which has been added to the merge list for a parent.
    // A child can have multiple parents for a given tree, and be included multiple times
    // by the same parent, hence we have to track it for a ((child, parent), line) tuple
    // instead of just the child or (child, parent).
    last_offset_set: HashMap<FilialTuple, usize>,
    // holds, for any given filial tuple, the iterator yielding all the positions at which the child
    // is included into the parent in line-sorted order. This is necessary for files that are imported
    // more than once into the same parent, so we can easily get the next include position.
    parent_child_edge_iterator: HashMap<FilialTuple, Box<(dyn Iterator<Item = IncludePosition> + 'a)>>,
}

impl<'a> MergeViewBuilder<'a> {
    pub fn new(
        nodes: &'a [FilialTuple], sources: &'a HashMap<PathBuf, String>, graph: &'a CachedStableGraph, source_mapper: &'a mut SourceMapper,
    ) -> Self {
        MergeViewBuilder {
            nodes,
            nodes_peeker: nodes.iter().peekable(),
            sources,
            graph,
            source_mapper,
            last_offset_set: HashMap::new(),
            parent_child_edge_iterator: HashMap::new(),
        }
    }

    pub fn build(&mut self) -> String {
        // contains additionally inserted lines such as #line and other directives, preamble defines etc
        let mut extra_lines: Vec<String> = Vec::new();
        extra_lines.reserve((self.nodes.len() * 2) + 2);

        // list of source code views onto the below sources
        let mut merge_list: LinkedList<&'a str> = LinkedList::new();

        // invariant: nodes_iter always has _at least_ one element. Can't save a not-file :B
        let first = self.nodes_peeker.next().unwrap().child;
        let first_path = self.graph.get_node(first);
        let first_source = self.sources.get(&first_path).unwrap();

        // seed source_mapper with top-level file
        self.source_mapper.get_num(first);

        let version_line_offset = self.find_version_offset(first_source);
        let _version_char_offsets = self.char_offset_for_line(version_line_offset, first_source);
        // add_preamble(
        //     version_line_offset,
        //     version_char_offsets.1,
        //     &first_path,
        //     first,
        //     first_source,
        //     &mut merge_list,
        //     &mut extra_lines,
        //     source_mapper,
        // );

        // last_offset_set.insert((first, None), version_char_offsets.1);
        self.set_last_offset_for_tuple(None, first, 0);

        // stack to keep track of the depth first traversal
        let mut stack = VecDeque::<NodeIndex>::new();

        self.create_merge_views(&mut merge_list, &mut extra_lines, &mut stack);

        // now we add a view of the remainder of the root file

        let offset = self.get_last_offset_for_tuple(None, first).unwrap();

        let len = first_source.len();
        merge_list.push_back(&first_source[min(offset, len)..]);

        let total_len = merge_list.iter().fold(0, |a, b| a + b.len());

        let mut merged = String::with_capacity(total_len);
        merged.extend(merge_list);

        merged
    }

    fn create_merge_views(&mut self, merge_list: &mut LinkedList<&'a str>, extra_lines: &mut Vec<String>, stack: &mut VecDeque<NodeIndex>) {
        loop {
            let n = match self.nodes_peeker.next() {
                Some(n) => n,
                None => return,
            };

            // invariant: never None as only the first element in `nodes` should have a None, which is popped off in the calling function
            let (parent, child) = (n.parent.unwrap(), n.child);
            // gets the next include position for the filial tuple, seeding if this is the first time querying this tuple
            let edge = self
                .parent_child_edge_iterator
                .entry(*n)
                .or_insert_with(|| {
                    let child_positions = self.graph.get_child_positions(parent, child);
                    Box::new(child_positions)
                })
                .next()
                .unwrap();
            let parent_path = self.graph.get_node(parent).clone();
            let child_path = self.graph.get_node(child).clone();

            let parent_source = self.sources.get(&parent_path).unwrap();
            let (char_for_line, char_following_line) = self.char_offset_for_line(edge.line, parent_source);

            let offset = *self
                .set_last_offset_for_tuple(stack.back().copied(), parent, char_following_line)
                .get_or_insert(0);

            debug!("creating view to start child file";
                "parent" => parent_path.to_str().unwrap(), "child" => child_path.to_str().unwrap(),
                "grandparent" => stack.back().copied().map(|g| self.graph.get_node(g).to_str().unwrap().to_string()), // self.graph.get_node().to_str().unwrap(),
                "last_parent_offset" => offset, "line" => edge.line, "char_for_line" => char_for_line,
                "char_following_line" => char_following_line,
            );

            merge_list.push_back(&parent_source[offset..char_for_line]);
            self.add_opening_line_directive(&child_path, child, merge_list, extra_lines);

            match self.nodes_peeker.peek() {
                Some(next) => {
                    let next = *next;
                    // if the next pair's parent is not a child of the current pair, we dump the rest of this childs source
                    if next.parent.unwrap() != child {
                        let child_source = self.sources.get(&child_path).unwrap();
                        // if ends in \n\n, we want to exclude the last \n for some reason. Ask optilad
                        let offset = {
                            match child_source.ends_with('\n') {
                                true => child_source.len() - 1,
                                false => child_source.len(),
                            }
                        };
                        merge_list.push_back(&child_source[..offset]);
                        self.set_last_offset_for_tuple(Some(parent), child, 0);
                        // +2 because edge.line is 0 indexed but #line is 1 indexed and references the *following* line
                        self.add_closing_line_directive(edge.line + 2, &parent_path, parent, merge_list, extra_lines);
                        // if the next pair's parent is not the current pair's parent, we need to bubble up
                        if stack.contains(&next.parent.unwrap()) {
                            return;
                        }
                        continue;
                    }

                    stack.push_back(parent);
                    self.create_merge_views(merge_list, extra_lines, stack);
                    stack.pop_back();

                    let offset = self.get_last_offset_for_tuple(Some(parent), child).unwrap();
                    let child_source = self.sources.get(&child_path).unwrap();
                    // this evaluates to false once the file contents have been exhausted aka offset = child_source.len() + 1
                    let end_offset = match child_source.ends_with('\n') {
                        true => 1,
                        false => 0,
                    };
                    if offset < child_source.len() - end_offset {
                        // if ends in \n\n, we want to exclude the last \n for some reason. Ask optilad
                        merge_list.push_back(&child_source[offset..child_source.len() - end_offset]);
                        self.set_last_offset_for_tuple(Some(parent), child, 0);
                    }

                    // +2 because edge.line is 0 indexed but #line is 1 indexed and references the *following* line
                    self.add_closing_line_directive(edge.line + 2, &parent_path, parent, merge_list, extra_lines);

                    // we need to check the next item at the point of original return further down the callstack
                    if self.nodes_peeker.peek().is_some() && stack.contains(&self.nodes_peeker.peek().unwrap().parent.unwrap()) {
                        return;
                    }
                }
                None => {
                    let child_source = self.sources.get(&child_path).unwrap();
                    // if ends in \n\n, we want to exclude the last \n for some reason. Ask optilad
                    let offset = match child_source.ends_with('\n') {
                        true => child_source.len() - 1,
                        false => child_source.len(),
                    };
                    merge_list.push_back(&child_source[..offset]);
                    self.set_last_offset_for_tuple(Some(parent), child, 0);
                    // +2 because edge.line is 0 indexed but #line is 1 indexed and references the *following* line
                    self.add_closing_line_directive(edge.line + 2, &parent_path, parent, merge_list, extra_lines);
                }
            }
        }
    }

    fn set_last_offset_for_tuple(&mut self, parent: Option<NodeIndex>, child: NodeIndex, offset: usize) -> Option<usize> {
        debug!("inserting last offset";
            "parent" => parent.map(|p| self.graph.get_node(p).to_str().unwrap().to_string()),
            "child" => self.graph.get_node(child).to_str().unwrap().to_string(),
            "offset" => offset);
        self.last_offset_set.insert(FilialTuple { child, parent }, offset)
    }

    fn get_last_offset_for_tuple(&self, parent: Option<NodeIndex>, child: NodeIndex) -> Option<usize> {
        self.last_offset_set.get(&FilialTuple { child, parent }).copied()
    }

    // returns the character offset + 1 of the end of line number `line` and the character
    // offset + 1 for the end of the line after the previous one
    fn char_offset_for_line(&self, line_num: usize, source: &str) -> (usize, usize) {
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

    fn find_version_offset(&self, source: &str) -> usize {
        source
            .lines()
            .enumerate()
            .find(|(_, line)| line.starts_with("#version "))
            .map_or(0, |(i, _)| i)
    }

    // fn add_preamble<'a>(
    //     version_line_offset: usize, version_char_offset: usize, path: &Path, node: NodeIndex, source: &'a str,
    //     merge_list: &mut LinkedList<&'a str>, extra_lines: &mut Vec<String>, source_mapper: &mut SourceMapper,
    // ) {
    //     // TODO: Optifine #define preabmle
    //     merge_list.push_back(&source[..version_char_offset]);
    //     let google_line_directive = format!(
    //         "#extension GL_GOOGLE_cpp_style_line_directive : enable\n#line {} {} // {}\n",
    //         // +2 because 0 indexed but #line is 1 indexed and references the *following* line
    //         version_line_offset + 2,
    //         source_mapper.get_num(node),
    //         path.to_str().unwrap().replace('\\', "\\\\"),
    //     );
    //     extra_lines.push(google_line_directive);
    //     unsafe_get_and_insert(merge_list, extra_lines);
    // }

    fn add_opening_line_directive(
        &mut self, path: &Path, node: NodeIndex, merge_list: &mut LinkedList<&str>, extra_lines: &mut Vec<String>,
    ) {
        let line_directive = format!(
            "#line 1 {} // {}\n",
            self.source_mapper.get_num(node),
            path.to_str().unwrap().replace('\\', "\\\\")
        );
        extra_lines.push(line_directive);
        self.unsafe_get_and_insert(merge_list, extra_lines);
    }

    fn add_closing_line_directive(
        &mut self, line: usize, path: &Path, node: NodeIndex, merge_list: &mut LinkedList<&str>, extra_lines: &mut Vec<String>,
    ) {
        // Optifine doesn't seem to add a leading newline if the previous line was a #line directive
        let line_directive = if let Some(l) = merge_list.back() {
            if l.trim().starts_with("#line") {
                format!(
                    "#line {} {} // {}\n",
                    line,
                    self.source_mapper.get_num(node),
                    path.to_str().unwrap().replace('\\', "\\\\")
                )
            } else {
                format!(
                    "\n#line {} {} // {}\n",
                    line,
                    self.source_mapper.get_num(node),
                    path.to_str().unwrap().replace('\\', "\\\\")
                )
            }
        } else {
            format!(
                "\n#line {} {} // {}\n",
                line,
                self.source_mapper.get_num(node),
                path.to_str().unwrap().replace('\\', "\\\\")
            )
        };

        extra_lines.push(line_directive);
        self.unsafe_get_and_insert(merge_list, extra_lines);
    }

    fn unsafe_get_and_insert(&self, merge_list: &mut LinkedList<&str>, extra_lines: &[String]) {
        // :^)
        unsafe {
            let vec_ptr_offset = extra_lines.as_ptr().add(extra_lines.len() - 1);
            merge_list.push_back(&vec_ptr_offset.as_ref().unwrap()[..]);
        }
    }
}

#[cfg(test)]
mod merge_view_test {
    use std::fs;
    use std::path::PathBuf;

    use crate::merge_views::MergeViewBuilder;
    use crate::source_mapper::SourceMapper;
    use crate::test::{copy_to_and_set_root, new_temp_server};
    use crate::IncludePosition;

    #[test]
    #[logging_macro::log_scope]
    fn test_generate_merge_list_01() {
        let mut server = new_temp_server(None);

        let (_tmp_dir, tmp_path) = copy_to_and_set_root("./testdata/01", &mut server);
        server.endpoint.request_shutdown();

        let final_idx = server.graph.borrow_mut().add_node(&tmp_path.join("shaders").join("final.fsh"));
        let common_idx = server.graph.borrow_mut().add_node(&tmp_path.join("shaders").join("common.glsl"));

        server
            .graph
            .borrow_mut()
            .add_edge(final_idx, common_idx, IncludePosition { line: 2, start: 0, end: 0 });

        let nodes = server.get_dfs_for_node(final_idx).unwrap();
        let sources = server.load_sources(&nodes).unwrap();

        let graph_borrow = server.graph.borrow();
        let mut source_mapper = SourceMapper::new(0);
        let result = MergeViewBuilder::new(&nodes, &sources, &graph_borrow, &mut source_mapper).build();

        let merge_file = tmp_path.join("shaders").join("final.fsh.merge");

        let mut truth = fs::read_to_string(merge_file).unwrap();
        // truth = truth.replacen(
        //     "!!",
        //     &tmp_path.join("shaders").join("final.fsh").to_str().unwrap().replace('\\', "\\\\"),
        //     1,
        // );
        truth = truth.replacen(
            "!!",
            &tmp_path.join("shaders").join("common.glsl").to_str().unwrap().replace('\\', "\\\\"),
            1,
        );
        truth = truth.replace(
            "!!",
            &tmp_path.join("shaders").join("final.fsh").to_str().unwrap().replace('\\', "\\\\"),
        );

        assert_eq!(result, truth);
    }

    #[test]
    #[logging_macro::log_scope]
    fn test_generate_merge_list_02() {
        let mut server = new_temp_server(None);

        let (_tmp_dir, tmp_path) = copy_to_and_set_root("./testdata/02", &mut server);
        server.endpoint.request_shutdown();

        let final_idx = server.graph.borrow_mut().add_node(&tmp_path.join("shaders").join("final.fsh"));
        let test_idx = server
            .graph
            .borrow_mut()
            .add_node(&tmp_path.join("shaders").join("utils").join("test.glsl"));
        let burger_idx = server
            .graph
            .borrow_mut()
            .add_node(&tmp_path.join("shaders").join("utils").join("burger.glsl"));
        let sample_idx = server
            .graph
            .borrow_mut()
            .add_node(&tmp_path.join("shaders").join("utils").join("sample.glsl"));

        server
            .graph
            .borrow_mut()
            .add_edge(final_idx, sample_idx, IncludePosition { line: 2, start: 0, end: 0 });
        server
            .graph
            .borrow_mut()
            .add_edge(sample_idx, burger_idx, IncludePosition { line: 4, start: 0, end: 0 });
        server
            .graph
            .borrow_mut()
            .add_edge(sample_idx, test_idx, IncludePosition { line: 6, start: 0, end: 0 });

        let nodes = server.get_dfs_for_node(final_idx).unwrap();
        let sources = server.load_sources(&nodes).unwrap();

        let graph_borrow = server.graph.borrow();
        let mut source_mapper = SourceMapper::new(0);
        let result = MergeViewBuilder::new(&nodes, &sources, &graph_borrow, &mut source_mapper).build();

        let merge_file = tmp_path.join("shaders").join("final.fsh.merge");

        let mut truth = fs::read_to_string(merge_file).unwrap();

        // truth = truth.replacen(
        //     "!!",
        //     &tmp_path.join("shaders").join("final.fsh").to_str().unwrap().replace('\\', "\\\\"),
        //     1,
        // );

        for file in &["sample.glsl", "burger.glsl", "sample.glsl", "test.glsl", "sample.glsl"] {
            let path = tmp_path.clone();
            truth = truth.replacen(
                "!!",
                &path
                    .join("shaders")
                    .join("utils")
                    .join(file)
                    .to_str()
                    .unwrap()
                    .replace('\\', "\\\\"),
                1,
            );
        }
        truth = truth.replacen(
            "!!",
            &tmp_path.join("shaders").join("final.fsh").to_str().unwrap().replace('\\', "\\\\"),
            1,
        );

        assert_eq!(result, truth);
    }

    #[test]
    #[logging_macro::log_scope]
    fn test_generate_merge_list_03() {
        let mut server = new_temp_server(None);

        let (_tmp_dir, tmp_path) = copy_to_and_set_root("./testdata/03", &mut server);
        server.endpoint.request_shutdown();

        let final_idx = server.graph.borrow_mut().add_node(&tmp_path.join("shaders").join("final.fsh"));
        let test_idx = server
            .graph
            .borrow_mut()
            .add_node(&tmp_path.join("shaders").join("utils").join("test.glsl"));
        let burger_idx = server
            .graph
            .borrow_mut()
            .add_node(&tmp_path.join("shaders").join("utils").join("burger.glsl"));
        let sample_idx = server
            .graph
            .borrow_mut()
            .add_node(&tmp_path.join("shaders").join("utils").join("sample.glsl"));

        server
            .graph
            .borrow_mut()
            .add_edge(final_idx, sample_idx, IncludePosition { line: 2, start: 0, end: 0 });
        server
            .graph
            .borrow_mut()
            .add_edge(sample_idx, burger_idx, IncludePosition { line: 4, start: 0, end: 0 });
        server
            .graph
            .borrow_mut()
            .add_edge(sample_idx, test_idx, IncludePosition { line: 6, start: 0, end: 0 });

        let nodes = server.get_dfs_for_node(final_idx).unwrap();
        let sources = server.load_sources(&nodes).unwrap();

        let graph_borrow = server.graph.borrow();
        let mut source_mapper = SourceMapper::new(0);
        let result = MergeViewBuilder::new(&nodes, &sources, &graph_borrow, &mut source_mapper).build();

        let merge_file = tmp_path.join("shaders").join("final.fsh.merge");

        let mut truth = fs::read_to_string(merge_file).unwrap();

        // truth = truth.replacen(
        //     "!!",
        //     &tmp_path.join("shaders").join("final.fsh").to_str().unwrap().replace('\\', "\\\\"),
        //     1,
        // );

        for file in &["sample.glsl", "burger.glsl", "sample.glsl", "test.glsl", "sample.glsl"] {
            let path = tmp_path.clone();
            truth = truth.replacen(
                "!!",
                &path
                    .join("shaders")
                    .join("utils")
                    .join(file)
                    .to_str()
                    .unwrap()
                    .replace('\\', "\\\\"),
                1,
            );
        }
        truth = truth.replacen(
            "!!",
            &tmp_path.join("shaders").join("final.fsh").to_str().unwrap().replace('\\', "\\\\"),
            1,
        );

        assert_eq!(result, truth);
    }

    #[test]
    #[logging_macro::log_scope]
    fn test_generate_merge_list_04() {
        let mut server = new_temp_server(None);

        let (_tmp_dir, tmp_path) = copy_to_and_set_root("./testdata/04", &mut server);
        server.endpoint.request_shutdown();

        let final_idx = server.graph.borrow_mut().add_node(&tmp_path.join("shaders").join("final.fsh"));
        let utilities_idx = server
            .graph
            .borrow_mut()
            .add_node(&tmp_path.join("shaders").join("utils").join("utilities.glsl"));
        let stuff1_idx = server
            .graph
            .borrow_mut()
            .add_node(&tmp_path.join("shaders").join("utils").join("stuff1.glsl"));
        let stuff2_idx = server
            .graph
            .borrow_mut()
            .add_node(&tmp_path.join("shaders").join("utils").join("stuff2.glsl"));
        let matrices_idx = server
            .graph
            .borrow_mut()
            .add_node(&tmp_path.join("shaders").join("lib").join("matrices.glsl"));

        server
            .graph
            .borrow_mut()
            .add_edge(final_idx, utilities_idx, IncludePosition { line: 2, start: 0, end: 0 });
        server
            .graph
            .borrow_mut()
            .add_edge(utilities_idx, stuff1_idx, IncludePosition { line: 0, start: 0, end: 0 });
        server
            .graph
            .borrow_mut()
            .add_edge(utilities_idx, stuff2_idx, IncludePosition { line: 1, start: 0, end: 0 });
        server
            .graph
            .borrow_mut()
            .add_edge(final_idx, matrices_idx, IncludePosition { line: 3, start: 0, end: 0 });

        let nodes = server.get_dfs_for_node(final_idx).unwrap();
        let sources = server.load_sources(&nodes).unwrap();

        let graph_borrow = server.graph.borrow();
        let mut source_mapper = SourceMapper::new(0);
        let result = MergeViewBuilder::new(&nodes, &sources, &graph_borrow, &mut source_mapper).build();

        let merge_file = tmp_path.join("shaders").join("final.fsh.merge");

        let mut truth = fs::read_to_string(merge_file).unwrap();

        for file in &[
            // PathBuf::new().join("final.fsh").to_str().unwrap(),
            PathBuf::new().join("utils").join("utilities.glsl").to_str().unwrap(),
            PathBuf::new().join("utils").join("stuff1.glsl").to_str().unwrap(),
            PathBuf::new().join("utils").join("utilities.glsl").to_str().unwrap(),
            PathBuf::new().join("utils").join("stuff2.glsl").to_str().unwrap(),
            PathBuf::new().join("utils").join("utilities.glsl").to_str().unwrap(),
            PathBuf::new().join("final.fsh").to_str().unwrap(),
            PathBuf::new().join("lib").join("matrices.glsl").to_str().unwrap(),
            PathBuf::new().join("final.fsh").to_str().unwrap(),
        ] {
            let path = tmp_path.clone();
            truth = truth.replacen("!!", &path.join("shaders").join(file).to_str().unwrap().replace('\\', "\\\\"), 1);
        }

        assert_eq!(result, truth);
    }

    #[test]
    #[logging_macro::log_scope]
    fn test_generate_merge_list_06() {
        let mut server = new_temp_server(None);

        let (_tmp_dir, tmp_path) = copy_to_and_set_root("./testdata/06", &mut server);
        server.endpoint.request_shutdown();

        let final_idx = server.graph.borrow_mut().add_node(&tmp_path.join("shaders").join("final.fsh"));
        let test_idx = server.graph.borrow_mut().add_node(&tmp_path.join("shaders").join("test.glsl"));

        server
            .graph
            .borrow_mut()
            .add_edge(final_idx, test_idx, IncludePosition { line: 3, start: 0, end: 0 });
        server
            .graph
            .borrow_mut()
            .add_edge(final_idx, test_idx, IncludePosition { line: 5, start: 0, end: 0 });

        let nodes = server.get_dfs_for_node(final_idx).unwrap();
        let sources = server.load_sources(&nodes).unwrap();

        let graph_borrow = server.graph.borrow();
        let mut source_mapper = SourceMapper::new(0);
        let result = MergeViewBuilder::new(&nodes, &sources, &graph_borrow, &mut source_mapper).build();

        let merge_file = tmp_path.join("shaders").join("final.fsh.merge");

        let mut truth = fs::read_to_string(merge_file).unwrap();

        for file in &[
            // PathBuf::new().join("final.fsh").to_str().unwrap(),
            PathBuf::new().join("test.glsl").to_str().unwrap(),
            PathBuf::new().join("final.fsh").to_str().unwrap(),
            PathBuf::new().join("test.glsl").to_str().unwrap(),
            PathBuf::new().join("final.fsh").to_str().unwrap(),
        ] {
            let path = tmp_path.clone();
            truth = truth.replacen("!!", &path.join("shaders").join(file).to_str().unwrap().replace('\\', "\\\\"), 1);
        }

        assert_eq!(result, truth);
    }
}
