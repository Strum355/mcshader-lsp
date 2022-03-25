use std::cell::RefCell;
use std::rc::Rc;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde_json::Value;

use petgraph::graph::NodeIndex;

use anyhow::{format_err, Result};

use std::fs;

use crate::dfs;
use crate::merge_views::FilialTuple;
use crate::{graph::CachedStableGraph, merge_views, url_norm::FromJson};

use super::Invokeable;

pub struct VirtualMergedDocument {
    pub graph: Rc<RefCell<CachedStableGraph>>,
}

impl VirtualMergedDocument {
    // TODO: DUPLICATE CODE
    fn get_file_toplevel_ancestors(&self, uri: &Path) -> Result<Option<Vec<petgraph::stable_graph::NodeIndex>>> {
        let curr_node = match self.graph.borrow_mut().find_node(uri) {
            Some(n) => n,
            None => return Err(format_err!("node not found {:?}", uri)),
        };
        let roots = self.graph.borrow().collect_root_ancestors(curr_node);
        if roots.is_empty() {
            return Ok(None);
        }
        Ok(Some(roots))
    }

    pub fn get_dfs_for_node(&self, root: NodeIndex) -> Result<Vec<FilialTuple>, dfs::error::CycleError> {
        let graph_ref = self.graph.borrow();

        let dfs = dfs::Dfs::new(&graph_ref, root);

        dfs.collect::<Result<Vec<_>, _>>()
    }

    pub fn load_sources(&self, nodes: &[FilialTuple]) -> Result<HashMap<PathBuf, String>> {
        let mut sources = HashMap::new();

        for node in nodes {
            let graph = self.graph.borrow();
            let path = graph.get_node(node.0);

            if sources.contains_key(&path) {
                continue;
            }

            let source = match fs::read_to_string(&path) {
                Ok(s) => s,
                Err(e) => return Err(format_err!("error reading {:?}: {}", path, e)),
            };
            let source = source.replace("\r\n", "\n");
            sources.insert(path.clone(), source);
        }

        Ok(sources)
    }
}

impl Invokeable for VirtualMergedDocument {
    fn run_command(&self, root: &Path, arguments: &[Value]) -> Result<Value> {
        let path = PathBuf::from_json(arguments.get(0).unwrap())?;

        let file_ancestors = match self.get_file_toplevel_ancestors(&path) {
            Ok(opt) => match opt {
                Some(ancestors) => ancestors,
                None => vec![],
            },
            Err(e) => return Err(e),
        };

        //info!("ancestors for {}:\n\t{:?}", path, file_ancestors.iter().map(|e| self.graph.borrow().graph.node_weight(*e).unwrap().clone()).collect::<Vec<String>>());

        // the set of all filepath->content. TODO: change to Url?
        let mut all_sources: HashMap<PathBuf, String> = HashMap::new();

        // if we are a top-level file (this has to be one of the set defined by Optifine, right?)
        if file_ancestors.is_empty() {
            // gather the list of all descendants
            let root = self.graph.borrow_mut().find_node(&path).unwrap();
            let tree = match self.get_dfs_for_node(root) {
                Ok(tree) => tree,
                Err(e) => return Err(e.into()),
            };

            let sources = match self.load_sources(&tree) {
                Ok(s) => s,
                Err(e) => return Err(e),
            };
            all_sources.extend(sources);

            let graph = self.graph.borrow();
            let view = merge_views::generate_merge_list(&tree, &all_sources, &graph);
            return Ok(serde_json::value::Value::String(view));
        }
        return Err(format_err!(
            "{:?} is not a top-level file aka has ancestors",
            path.strip_prefix(root).unwrap()
        ));
    }
}
