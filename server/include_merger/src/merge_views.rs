use std::cmp::min;
use std::collections::{HashMap, LinkedList, VecDeque};
use std::iter::Peekable;

use core::slice::Iter;

use filesystem::{LFString, NormalizedPathBuf};
use graph::FilialTuple;
use logging::debug;
use tree_sitter::{Parser, Query, QueryCursor};
use tree_sitter_glsl::language;

use crate::consts;
use sourcefile::{IncludeLine, SourceMapper, Sourcefile};

const ERROR_DIRECTIVE: &str = "#error ";

/// Merges the source strings according to the nodes comprising a tree of imports into a GLSL source string
/// that can be handed off to the GLSL compiler.
pub struct MergeViewBuilder<'a> {
    root: &'a NormalizedPathBuf,

    nodes: Peekable<Iter<'a, FilialTuple<&'a Sourcefile>>>,

    /// contains additionally inserted lines such as #line and other directives, preamble defines etc
    extra_lines: Vec<String>,

    // graph: &'a CachedStableGraph<NormalizedPathBuf, IncludeLine>,
    source_mapper: &'a mut SourceMapper<NormalizedPathBuf>,

    /// holds the offset into the child which has been added to the merge list for a parent.
    /// A child can have multiple parents for a given tree, and be included multiple times
    /// by the same parent, hence we have to track it for a ((child, parent), line) tuple
    /// instead of just the child or (child, parent).
    last_offset_set: HashMap<FilialTuple<&'a NormalizedPathBuf>, usize>,
    /// is included into the parent in line-sorted order. This is necessary for files that are imported
    /// more than once into the same parent, so we can easily get the next include position.
    parent_child_edge_iterator: HashMap<FilialTuple<&'a NormalizedPathBuf>, Box<(dyn Iterator<Item = IncludeLine> + 'a)>>,
}

impl<'a> MergeViewBuilder<'a> {
    pub fn new(
        root: &'a NormalizedPathBuf, nodes: &'a [FilialTuple<&'a Sourcefile>], source_mapper: &'a mut SourceMapper<NormalizedPathBuf>,
    ) -> Self {
        let mut all_includes: Vec<_> = nodes
            .iter()
            .flat_map(|tup| tup.child.includes().unwrap())
            .map(|tup| tup.0)
            .collect();
        all_includes.sort_unstable();
        all_includes.dedup();

        MergeViewBuilder {
            root,
            nodes: nodes.iter().peekable(),
            // 1 start + 1 end #line & 1 preamble + 1 end #line + at worst the amount of #include directives found
            // TODO: more compatibility inserts
            extra_lines: Vec::with_capacity((nodes.len() * 2) + 2 + all_includes.len()),
            source_mapper,
            last_offset_set: HashMap::new(),
            parent_child_edge_iterator: HashMap::new(),
        }
    }

    pub fn build(&mut self) -> LFString {
        // list of source code views onto the below sources
        let mut merge_list: LinkedList<&'a str> = LinkedList::new();

        // invariant: nodes_iter always has _at least_ one element. Can't save a not-file :B
        let first = self.nodes.next().unwrap().child;
        let first_path = &first.path;
        let first_source = &first.source;

        // seed source_mapper with top-level file
        self.source_mapper.get_num(&first.path);

        // add the optifine preamble (and extra compatibility mangling eventually)
        let version_line_offset = self.find_version_offset(first_source);
        let (_, version_char_following_line) = self.char_offset_for_line(version_line_offset, first_source);
        self.add_preamble(
            version_line_offset,
            version_char_following_line,
            first_path,
            first_source,
            &mut merge_list,
        );

        self.set_last_offset_for_tuple(None, first_path, version_char_following_line);
        // self.set_last_offset_for_tuple(None, first, 0);

        // stack to keep track of the depth first traversal
        let mut stack: VecDeque<_> = VecDeque::<&'a NormalizedPathBuf>::new();

        // where the magic happens!
        self.create_merge_views(&mut merge_list, &mut stack);

        // now we add a view of the remainder of the root file
        let offset = self.get_last_offset_for_tuple(None, first_path).unwrap();
        let len = first_source.len();
        self.process_slice_addition(&mut merge_list, first_path, &first_source[min(offset, len)..]);
        // merge_list.push_back(&first_source[min(offset, len)..]);

        // Now merge all the views into one singular String to return
        let total_len = merge_list.iter().fold(0, |a, b| a + b.len());
        let mut merged = String::with_capacity(total_len);
        merged.extend(merge_list);

        LFString::from_unchecked(merged)
    }

    fn create_merge_views(&mut self, merge_list: &mut LinkedList<&'a str>, stack: &mut VecDeque<&'a NormalizedPathBuf>) {
        loop {
            let n = match self.nodes.next() {
                Some(n) => n,
                None => return,
            };

            // invariant: never None as only the first element in `nodes` should have a None, which is popped off in the calling function
            let (parent, child) = (n.parent.unwrap(), n.child);
            let parent_path = &parent.path;
            let child_path = &child.path;
            // gets the next include position for the filial tuple, seeding if this is the first time querying this tuple
            let edge = self
                .parent_child_edge_iterator
                .entry(FilialTuple {
                    child: &n.child.path,
                    parent: n.parent.map(|p| &p.path),
                })
                .or_insert_with(|| Box::new(parent.includes_of_path(child_path).unwrap()))
                .next()
                .unwrap();

            let parent_source = &parent.source;
            let (char_for_line, char_following_line) = self.char_offset_for_line(edge, parent_source);

            let offset = *self
                .set_last_offset_for_tuple(stack.back().copied(), parent_path, char_following_line)
                .get_or_insert(0);

            debug!("creating view to start child file";
                "parent" => parent_path, "child" => child_path,
                "grandparent" => stack.back(),
                "last_parent_offset" => offset, "line" => edge, "char_for_line" => char_for_line,
                "char_following_line" => char_following_line,
            );

            self.process_slice_addition(merge_list, parent_path, &parent_source[offset..char_for_line]);

            // merge_list.push_back(&parent_source[offset..char_for_line]);
            self.add_opening_line_directive(child_path, merge_list);

            match self.nodes.peek() {
                Some(next) => {
                    let next = *next;
                    // if the next pair's parent is not a child of the current pair, we dump the rest of this childs source
                    if &next.parent.unwrap().path != child_path {
                        let child_source = &child.source;
                        // if ends in \n\n, we want to exclude the last \n for some reason. Ask optilad
                        let double_newline_offset = match child_source.ends_with('\n') {
                            true => child_source.len() - 1,
                            false => child_source.len(),
                        };
                        self.process_slice_addition(merge_list, child_path, &child_source[..double_newline_offset]);
                        // merge_list.push_back(&child_source[..double_newline_offset]);
                        self.set_last_offset_for_tuple(Some(parent_path), child_path, 0);
                        self.add_closing_line_directive(edge + 1, parent_path, merge_list);

                        // if the next pair's parent is not the current pair's parent, we need to bubble up
                        if stack.contains(&&next.parent.unwrap().path) {
                            return;
                        }
                        continue;
                    }

                    stack.push_back(&parent.path);
                    self.create_merge_views(merge_list, stack);
                    stack.pop_back();

                    let offset = self.get_last_offset_for_tuple(Some(parent_path), child_path).unwrap();
                    let child_source = &child.source;
                    // this evaluates to false once the file contents have been exhausted aka offset = child_source.len() + 1
                    let end_offset = match child_source.ends_with('\n') {
                        true => 1,
                        false => 0,
                    };
                    if offset < child_source.len() - end_offset {
                        // if ends in \n\n, we want to exclude the last \n for some reason. Ask optilad
                        self.process_slice_addition(merge_list, child_path, &child_source[offset..child_source.len() - end_offset]);
                        // merge_list.push_back(&child_source[offset..child_source.len() - end_offset]);
                        self.set_last_offset_for_tuple(Some(parent_path), child_path, 0);
                    }

                    self.add_closing_line_directive(edge + 1, parent_path, merge_list);

                    // we need to check the next item at the point of original return further down the callstack
                    if self.nodes.peek().is_some() && stack.contains(&&self.nodes.peek().unwrap().parent.unwrap().path) {
                        return;
                    }
                }
                None => {
                    // let child_source = self.sources.get(child_path).unwrap();
                    let child_source = &child.source;
                    // if ends in \n\n, we want to exclude the last \n for some reason. Ask optilad
                    let double_newline_offset = match child_source.ends_with('\n') {
                        true => child_source.len() - 1,
                        false => child_source.len(),
                    };
                    self.process_slice_addition(merge_list, child_path, &child_source[..double_newline_offset]);
                    // merge_list.push_back(&child_source[..double_newline_offset]);
                    self.set_last_offset_for_tuple(Some(parent_path), child_path, 0);
                    self.add_closing_line_directive(edge + 1, parent_path, merge_list);
                }
            }
        }
    }

    // process each new view here e.g. to replace #include statements that were not removed by a file existing with
    // #error etc
    fn process_slice_addition(&mut self, merge_list: &mut LinkedList<&'a str>, path: &NormalizedPathBuf, slice: &'a str) {
        let mut parser = Parser::new();
        parser.set_language(language()).unwrap();

        let query = Query::new(language(), sourcefile::GET_INCLUDES).unwrap();
        let mut query_cursor = QueryCursor::new();

        let mut start_offset = 0;
        let mut end_offset = slice.len();

        for (m, _) in query_cursor.captures(&query, parser.parse(slice, None).unwrap().root_node(), slice.as_bytes()) {
            if m.captures.is_empty() {
                continue;
            }

            let include = m.captures[0];
            let include_str = {
                let mut string = include.node.utf8_text(slice.as_bytes()).unwrap();
                string = &string[1..string.len() - 1];
                if string.starts_with('/') {
                    self.root.join("shaders").join(string.strip_prefix('/').unwrap())
                } else {
                    path.parent().unwrap().join(string)
                }
            };

            let line_offset = slice[start_offset..include.node.byte_range().start].rfind('\n').unwrap() + 1;
            merge_list.push_back(&slice[start_offset..line_offset]);
            end_offset = include.node.byte_range().end;
            start_offset = end_offset;
            merge_list.push_back(ERROR_DIRECTIVE);
            self.extra_lines.push(format!("Couldn't import file {}\n", include_str));
            self.unsafe_get_and_insert(merge_list)
        }

        merge_list.push_back(&slice[start_offset..end_offset]);
    }

    fn set_last_offset_for_tuple(
        &mut self, parent: Option<&'a NormalizedPathBuf>, child: &'a NormalizedPathBuf, offset: usize,
    ) -> Option<usize> {
        debug!("inserting last offset";
            "parent" => parent,
            "child" => &child,
            "offset" => offset);
        self.last_offset_set.insert(FilialTuple { child, parent }, offset)
    }

    #[inline]
    fn get_last_offset_for_tuple(&self, parent: Option<&'a NormalizedPathBuf>, child: &'a NormalizedPathBuf) -> Option<usize> {
        self.last_offset_set.get(&FilialTuple { child, parent }).copied()
    }

    // returns the character offset + 1 of the end of line number `line` and the character
    // offset + 1 for the end of the line after the previous one
    fn char_offset_for_line(&self, line_num: impl Into<usize> + Copy, source: &str) -> (usize, usize) {
        let mut char_for_line: usize = 0;
        let mut char_following_line: usize = 0;
        for (n, line) in source.lines().enumerate() {
            if n == line_num.into() {
                char_following_line += line.len() + 1;
                break;
            }
            char_for_line += line.len() + 1;
            char_following_line = char_for_line;
        }
        (char_for_line, char_following_line)
    }

    #[inline]
    fn find_version_offset(&self, source: &str) -> usize {
        source
            .lines()
            .enumerate()
            .find(|(_, line)| line.starts_with("#version "))
            .map_or(0, |(i, _)| i)
    }

    fn add_preamble(
        &mut self, version_line_offset: impl Into<usize>, version_char_offset: usize, path: &NormalizedPathBuf, source: &'a str,
        merge_list: &mut LinkedList<&'a str>,
    ) {
        self.process_slice_addition(merge_list, path, &source[..version_char_offset]);
        // merge_list.push_back(&source[..version_char_offset]);
        self.extra_lines.push(consts::OPTIFINE_PREAMBLE.into());
        self.unsafe_get_and_insert(merge_list);
        self.add_closing_line_directive(version_line_offset.into() + 1, path, merge_list);
    }

    fn add_opening_line_directive(&mut self, path: &NormalizedPathBuf, merge_list: &mut LinkedList<&str>) {
        let line_directive = format!("#line 0 {} // {}\n", self.source_mapper.get_num(path), path);
        self.extra_lines.push(line_directive);
        self.unsafe_get_and_insert(merge_list);
    }

    fn add_closing_line_directive(&mut self, line: impl Into<usize>, path: &NormalizedPathBuf, merge_list: &mut LinkedList<&str>) {
        // Optifine doesn't seem to add a leading newline if the previous line was a #line directive
        let line_directive = if let Some(l) = merge_list.back() {
            if l.trim().starts_with("#line") {
                format!("#line {} {} // {}\n", line.into(), self.source_mapper.get_num(path), path)
            } else {
                format!("\n#line {} {} // {}\n", line.into(), self.source_mapper.get_num(path), path)
            }
        } else {
            format!("\n#line {} {} // {}\n", line.into(), self.source_mapper.get_num(path), path)
        };

        self.extra_lines.push(line_directive);
        self.unsafe_get_and_insert(merge_list);
    }

    fn unsafe_get_and_insert(&self, merge_list: &mut LinkedList<&str>) {
        // :^)
        unsafe {
            let vec_ptr_offset = self.extra_lines.as_ptr().add(self.extra_lines.len() - 1);
            merge_list.push_back(&vec_ptr_offset.as_ref().unwrap()[..]);
        }
    }
}

#[cfg(test)]
mod test {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use anyhow::Result;

    use filesystem::{LFString, NormalizedPathBuf};
    use fs_extra::{copy_items, dir};
    use pretty_assertions::assert_str_eq;
    use sourcefile::SourceMapper;
    use tempdir::TempDir;
    use workspace::{TreeError, WorkspaceTree};

    use crate::MergeViewBuilder;

    fn copy_to_tmp_dir(test_path: &str) -> (TempDir, NormalizedPathBuf) {
        let tmp_dir = TempDir::new("mcshader").unwrap();
        fs::create_dir(tmp_dir.path().join("shaders")).unwrap();

        {
            let test_path = Path::new(test_path)
                .canonicalize()
                .unwrap_or_else(|_| panic!("canonicalizing '{}'", test_path));
            let opts = &dir::CopyOptions::new();
            let files = fs::read_dir(test_path)
                .unwrap()
                .map(|e| String::from(e.unwrap().path().to_str().unwrap()))
                .collect::<Vec<String>>();
            copy_items(&files, tmp_dir.path().join("shaders"), opts).unwrap();
        }

        let tmp_path = tmp_dir.path().to_str().unwrap().into();

        (tmp_dir, tmp_path)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[logging_macro::scope]
    async fn test_generate_merge_list_01() {
        let (_tmp_dir, tmp_path) = copy_to_tmp_dir("../testdata/01");
        let mut workspace = WorkspaceTree::new(&tmp_path.clone());
        workspace.build();

        let final_path = tmp_path.join("shaders").join("final.fsh");
        let common_path = tmp_path.join("shaders").join("common.glsl");

        let mut trees_vec = workspace
            .trees_for_entry(&final_path)
            .expect("expected successful tree initializing")
            .collect::<Result<Vec<_>, TreeError>>()
            .expect("expected successful tree-building");
        let mut trees = trees_vec.iter_mut();

        let tree = trees.next().unwrap();

        assert!(trees.next().is_none());

        let tree = tree
            .collect::<Result<Vec<_>, TreeError>>()
            .expect("expected successful tree-building");

        let mut source_mapper = SourceMapper::new(2);

        let result = MergeViewBuilder::new(&tmp_path, &tree, &mut source_mapper).build();

        let merge_file = tmp_path.join("shaders").join("final.fsh.merge");

        let mut truth = LFString::read(merge_file).await.unwrap();
        truth = LFString::from_unchecked(truth.replacen("!!", &final_path.to_string(), 1));
        truth = LFString::from_unchecked(truth.replacen("!!", &common_path.to_string(), 1));
        truth = LFString::from_unchecked(truth.replace("!!", &final_path.to_string()));

        assert_str_eq!(*truth, *result);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[logging_macro::scope]
    async fn test_generate_merge_list_02() {
        let (_tmp_dir, tmp_path) = copy_to_tmp_dir("../testdata/02");
        let mut workspace = WorkspaceTree::new(&tmp_path.clone());
        workspace.build();

        // println!(
        //     "connected {}. leaf {}",
        //     workspace.num_connected_entries(),
        //     // workspace.num_disconnected_entries(),
        // );

        let final_path = tmp_path.join("shaders").join("final.fsh");

        let mut trees_vec = workspace
            .trees_for_entry(&final_path)
            .expect("expected successful tree initializing")
            .collect::<Result<Vec<_>, TreeError>>()
            .expect("expected successful tree-building");
        let mut trees = trees_vec.iter_mut();

        let tree = trees.next().unwrap();

        assert!(trees.next().is_none());

        let tree = tree
            .collect::<Result<Vec<_>, TreeError>>()
            .expect("expected successful tree-building");

        let mut source_mapper = SourceMapper::new(2);

        let result = MergeViewBuilder::new(&tmp_path, &tree, &mut source_mapper).build();

        let merge_file = tmp_path.join("shaders").join("final.fsh.merge");

        let mut truth = LFString::read(merge_file).await.unwrap();

        truth = LFString::from_unchecked(truth.replacen("!!", &tmp_path.join("shaders").join("final.fsh").to_string(), 1));
        for file in &["sample.glsl", "burger.glsl", "sample.glsl", "test.glsl", "sample.glsl"] {
            let path = tmp_path.clone();
            truth = LFString::from_unchecked(truth.replacen("!!", &path.join("shaders").join("utils").join(file).to_string(), 1));
        }
        truth = LFString::from_unchecked(truth.replacen("!!", &tmp_path.join("shaders").join("final.fsh").to_string(), 1));

        assert_str_eq!(*truth, *result);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[logging_macro::scope]
    async fn test_generate_merge_list_03() {
        let (_tmp_dir, tmp_path) = copy_to_tmp_dir("../testdata/03");
        let mut workspace = WorkspaceTree::new(&tmp_path.clone());
        workspace.build();

        let final_path = tmp_path.join("shaders").join("final.fsh");

        let mut trees_vec = workspace
            .trees_for_entry(&final_path)
            .expect("expected successful tree initializing")
            .collect::<Result<Vec<_>, TreeError>>()
            .expect("expected successful tree-building");
        let mut trees = trees_vec.iter_mut();

        let tree = trees.next().unwrap();

        assert!(trees.next().is_none());

        let tree = tree
            .collect::<Result<Vec<_>, TreeError>>()
            .expect("expected successful tree-building");

        let mut source_mapper = SourceMapper::new(2);

        let result = MergeViewBuilder::new(&tmp_path, &tree, &mut source_mapper).build();

        let merge_file = tmp_path.join("shaders").join("final.fsh.merge");

        let mut truth = LFString::read(merge_file).await.unwrap();

        truth = LFString::from_unchecked(truth.replacen("!!", &tmp_path.join("shaders").join("final.fsh").to_string(), 1));
        for file in &["sample.glsl", "burger.glsl", "sample.glsl", "test.glsl", "sample.glsl"] {
            let path = tmp_path.clone();
            truth = LFString::from_unchecked(truth.replacen("!!", &path.join("shaders").join("utils").join(file).to_string(), 1));
        }
        truth = LFString::from_unchecked(truth.replacen("!!", &tmp_path.join("shaders").join("final.fsh").to_string(), 1));

        assert_str_eq!(*truth, *result);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[logging_macro::scope]
    async fn test_generate_merge_list_04() {
        let (_tmp_dir, tmp_path) = copy_to_tmp_dir("../testdata/04");
        let mut workspace = WorkspaceTree::new(&tmp_path.clone());
        workspace.build();

        let final_path = tmp_path.join("shaders").join("final.fsh");

        let mut trees_vec = workspace
            .trees_for_entry(&final_path)
            .expect("expected successful tree initializing")
            .collect::<Result<Vec<_>, TreeError>>()
            .expect("expected successful tree-building");
        let mut trees = trees_vec.iter_mut();

        let tree = trees.next().unwrap();

        assert!(trees.next().is_none());

        let tree = tree
            .collect::<Result<Vec<_>, TreeError>>()
            .expect("expected successful tree-building");

        let mut source_mapper = SourceMapper::new(2);

        let result = MergeViewBuilder::new(&tmp_path, &tree, &mut source_mapper).build();

        let merge_file = tmp_path.join("shaders").join("final.fsh.merge");

        let mut truth = LFString::read(merge_file).await.unwrap();

        for file in &[
            PathBuf::new().join("final.fsh").to_str().unwrap(),
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
            truth = LFString::from_unchecked(truth.replacen("!!", &path.join("shaders").join(file).to_string(), 1));
        }

        assert_str_eq!(*truth, *result);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[logging_macro::scope]
    async fn test_generate_merge_list_06() {
        let (_tmp_dir, tmp_path) = copy_to_tmp_dir("../testdata/06");
        let mut workspace = WorkspaceTree::new(&tmp_path.clone());
        workspace.build();

        let final_path = tmp_path.join("shaders").join("final.fsh");

        let mut trees_vec = workspace
            .trees_for_entry(&final_path)
            .expect("expected successful tree initializing")
            .collect::<Result<Vec<_>, TreeError>>()
            .expect("expected successful tree-building");
        let mut trees = trees_vec.iter_mut();

        let tree = trees.next().unwrap();

        assert!(trees.next().is_none());

        let tree = tree
            .collect::<Result<Vec<_>, TreeError>>()
            .expect("expected successful tree-building");

        let mut source_mapper = SourceMapper::new(2);

        let result = MergeViewBuilder::new(&tmp_path, &tree, &mut source_mapper).build();

        let merge_file = tmp_path.join("shaders").join("final.fsh.merge");

        let mut truth = LFString::read(merge_file).await.unwrap();
        for file in &[
            PathBuf::new().join("final.fsh").to_str().unwrap(),
            PathBuf::new().join("test.glsl").to_str().unwrap(),
            PathBuf::new().join("final.fsh").to_str().unwrap(),
            PathBuf::new().join("test.glsl").to_str().unwrap(),
            PathBuf::new().join("final.fsh").to_str().unwrap(),
        ] {
            let path = tmp_path.clone();
            truth = LFString::from_unchecked(truth.replacen("!!", &path.join("shaders").join(file).to_string(), 1));
        }

        assert_str_eq!(*truth, *result);
    }
}
