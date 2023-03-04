#![feature(result_flattening)]
#![feature(arc_unwrap_or_clone)]
#![feature(type_alias_impl_trait)]

use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    fs::read_to_string,
};

use filesystem::{is_top_level, NormalizedPathBuf};
use graph::{
    dfs::{CycleError, Dfs},
    CachedStableGraph, FilialTuple, NodeIndex,
};
use logging::{debug, info, warn};
use sourcefile::{IncludeLine, Sourcefile};
use walkdir::WalkDir;

mod tree;

pub struct WorkspaceTree {
    root: NormalizedPathBuf,
    pub graph: CachedStableGraph<NormalizedPathBuf, IncludeLine>,
    disconnected: HashSet<NormalizedPathBuf>,
    sources: HashMap<NormalizedPathBuf, Sourcefile>,
}

// #[derive(thiserror::Error, Debug)]
// pub enum TreesGenError {
//     #[error("got a non-valid top-level file: {0}")]
//     NonTopLevel(NormalizedPathBuf),
//     #[error(transparent)]
//     PathNotFound(#[from] graph::NotFound<NormalizedPathBuf>),
// }

#[derive(thiserror::Error, Debug)]
pub enum TreeError {
    #[error(transparent)]
    DfsError(#[from] CycleError<NormalizedPathBuf>),
    #[error("file {missing} not found; imported by {importing}.")]
    FileNotFound {
        importing: NormalizedPathBuf,
        missing: NormalizedPathBuf,
    },
    // #[error(transparent)]
    // PathNotFound(#[from] graph::NotFound<NormalizedPathBuf>),
}

pub type MaterializedTree<'a> = Vec<FilialTuple<&'a Sourcefile>>;

pub type ImmaterializedTree<'a> = impl Iterator<Item = Result<FilialTuple<&'a Sourcefile>, TreeError>>;

#[derive(thiserror::Error, Debug)]
#[error("got a non-valid top-level file: {0}")]
pub struct NonTopLevelError(NormalizedPathBuf);

pub type SingleTreeGenResult<'a> = Result<ImmaterializedTree<'a>, NonTopLevelError>;

pub type AllTreesGenResult<'a> = Result<Vec<SingleTreeGenResult<'a>>, graph::NotFound<NormalizedPathBuf>>;

impl WorkspaceTree {
    pub fn new(root: &NormalizedPathBuf) -> Self {
        WorkspaceTree {
            root: root.clone(),
            graph: CachedStableGraph::new(),
            disconnected: HashSet::new(),
            sources: HashMap::new(),
        }
    }

    pub fn num_connected_entries(&self) -> usize {
        self.graph.node_count()
    }

    // pub fn num_disconnected_entries(&self) -> usize {
    //     self.disconnected.len()
    // }

    /// builds the set of connected and disconnected GLSL files from the root of the
    /// workspace.
    // TODO: support user-defined additional file extensions.
    pub fn build(&mut self) {
        let root = self.root.clone();

        enum GraphEntry {
            // represents top-level nodes
            TopLevel(Sourcefile),
            // represents non-top-level nodes
            Leaf(Sourcefile),
        }

        // let mut roots = Vec::new();

        for entry in WalkDir::new(&root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_file())
            .map(|entry| NormalizedPathBuf::from(entry.into_path()))
            .filter_map(|path| {
                // files not imported anywhere wont be included in the graph,
                // this is ok for now.
                if !is_top_level(&path.strip_prefix(&root)) {
                    let ext = path.extension();
                    if ext == Some("fsh") || ext == Some("gsh") || ext == Some("vsh") || ext == Some("glsl") || ext == Some("csh") {
                        return Some(GraphEntry::Leaf(Sourcefile::new(read_to_string(&path).ok()?, path, root.clone())));
                    }
                    return None;
                }

                Some(GraphEntry::TopLevel(Sourcefile::new(
                    read_to_string(&path).ok()?,
                    path,
                    root.clone(),
                )))
            })
        {
            // iterate all valid found files, search for includes, add a node into the graph for each
            // file and add a file->includes KV into the map
            match entry {
                GraphEntry::TopLevel(file) => {
                    // eprintln!("TOP LEVEL {}", file.path);
                    let path = file.path.clone();
                    // roots.push(file.clone());
                    // self.sources.insert(path.clone(), file);
                    self.update_sourcefile(&path, file.source);
                }
                GraphEntry::Leaf(file) => {
                    // eprintln!("LEAF {}", file.path);
                    let path = file.path.clone();
                    // self.sources.insert(path.clone(), file);
                    self.update_sourcefile(&path, file.source);
                    // self.disconnected.insert(path);
                }
            };
        }
    }

    /// Returns the lazy depth first iterators for the possible trees given any node.
    /// If it is a top-level node, only a single tree should be instantiated. If not a top-level node,
    /// a tree will be instantiated for every top-level root ancestor.
    ///
    /// Error modes:
    ///   - Top [`Result`]
    ///     - The node is not known to the workspace
    ///   - Middle [`Result`] (only for >1 ancestor)
    ///     - A non-valid top-level ancestor was found
    ///   - Bottom [`Result`]
    ///     - A cycle was detected while iterating
    ///     - A node was not found on the filesystem while synthesizing a Sourcefile instance
    pub fn trees_for_entry<'a>(&'a mut self, entry: &NormalizedPathBuf) -> AllTreesGenResult<'a> {
        let root_ancestors = self.graph.root_ancestors_for_key(entry)?.unwrap_or_default();

        let mut trees = Vec::with_capacity(root_ancestors.len().max(1));

        info!("top-level file ancestors found";
            "uri" => entry,
            "ancestors" => format!("{:?}", root_ancestors.iter()
            .copied()
            .map(|e| &self.graph.graph[e])
            .collect::<Vec<_>>())
        );

        let node = self.graph.find_node(entry).unwrap();

        let transform_cycle_error =
            |result: Result<FilialTuple<NodeIndex>, CycleError<NormalizedPathBuf>>| result.map_err(TreeError::DfsError);
        let node_to_sourcefile = |result: Result<FilialTuple<NodeIndex>, TreeError>| -> Result<FilialTuple<&Sourcefile>, TreeError> {
            result.and_then(|tup| {
                let parent = tup.parent.map(|p| {
                    let parent_path = &self.graph[p];
                    // fatal error case, shouldnt happen
                    self.sources
                        .get(parent_path)
                        .unwrap_or_else(|| panic!("no entry in sources for parent {}", parent_path))
                });

                let child_path = &self.graph[tup.child];
                // soft-fail case, if file doesnt exist or mistype
                // eprintln!("MISSING? {:?}", self.sources.get(child_path).is_none());
                let child = self.sources.get(child_path).ok_or_else(|| TreeError::FileNotFound {
                    importing: self.graph[tup.parent.unwrap()].clone(),
                    missing: child_path.clone(),
                })?;

                Ok(FilialTuple { child, parent })
            })
        };

        if root_ancestors.is_empty() {
            if !is_top_level(&entry.strip_prefix(&self.root)) {
                trees.push(Err(NonTopLevelError(entry.clone())));
                return Ok(trees);
            }

            let dfs = Dfs::new(&self.graph, node)
                .into_iter()
                .map(transform_cycle_error)
                .map(node_to_sourcefile);
            trees.push(Ok(dfs));
        } else {
            for root in root_ancestors {
                let root_path = &self.graph[root];
                if !is_top_level(&root_path.strip_prefix(&self.root)) {
                    warn!("got a non-valid toplevel file"; "root_ancestor" => root_path);
                    trees.push(Err(NonTopLevelError(root_path.clone())));
                    continue;
                }

                let dfs = Dfs::new(&self.graph, root)
                    .into_iter()
                    .map(transform_cycle_error)
                    .map(node_to_sourcefile);
                trees.push(Ok(dfs));
            }
        }

        Ok(trees)
    }

    /// updates the set of GLSL files connected to the given file, moving unreferenced
    pub fn update_sourcefile(&mut self, path: &NormalizedPathBuf, text: String) {
        match self.sources.entry(path.clone()) {
            Entry::Occupied(mut entry) => entry.get_mut().source = text,
            Entry::Vacant(entry) => {
                entry.insert(Sourcefile::new(text, path.clone(), self.root.clone()));
            }
        };

        let includes = self.sources.get(path).unwrap().includes().unwrap();

        info!("includes found for file"; "file" => path, "includes" => format!("{:?}", includes));

        let idx = self.graph.add_node(path);

        let prev_children: HashSet<_> =
            HashSet::from_iter(self.graph.get_all_edges_from(idx).map(|tup| (self.graph[tup.0].clone(), tup.1)));
        let new_children: HashSet<_> = includes.iter().cloned().collect();

        let to_be_added = new_children.difference(&prev_children);
        let to_be_removed = prev_children.difference(&new_children);

        debug!(
            "include sets diff'd";
            "for removal" => format!("{:?}", to_be_removed),
            "for addition" => format!("{:?}", to_be_added)
        );

        for removal in to_be_removed {
            let child = self.graph.find_node(&removal.0).unwrap();
            self.graph.remove_edge(idx, child, removal.1);
            if removal.0.exists() && self.graph.parents(child).count() == 0 {
                self.disconnected.insert(removal.0.clone());
            }
        }

        // TODO: remove entire subtree from disconnected
        for insertion in to_be_added {
            let (child, position) = includes.iter().find(|f| f.0 == insertion.0).unwrap().clone();
            let child = self.graph.add_node(&child);
            self.graph.add_edge(idx, child, position);
        }
    }

    pub fn remove_sourcefile(&mut self, path: &NormalizedPathBuf) {
        let idx = self
            .graph
            .find_node(path)
            .unwrap_or_else(|| panic!("path {:?} wasn't in the graph to begin with???", path));

        self.disconnected.remove(path);
        self.sources.remove(path);
        self.graph.remove_node(idx);

        info!("removed file from graph"; "file" => path);
    }
}

#[cfg(test)]
mod test {
    use crate::WorkspaceTree;

    #[test]
    fn test_trees() {
        let mut view = WorkspaceTree::new(&("/home/test/banana".into()));
        let parent = view.graph.add_node(&("/home/test/banana/test.fsh".into()));
        let child = view.graph.add_node(&("/home/test/banana/included.glsl".into()));
        view.graph.add_edge(parent, child, 2.into());

        let parent = "/home/test/banana/test.fsh".into();
        let trees = view.trees_for_entry(&parent);
        trees.unwrap()[0].as_ref().err().expect("unexpected Ok result");
    }
}
