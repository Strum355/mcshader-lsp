use std::cell::RefCell;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::rc::Rc;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use petgraph::dot::Config;
use serde_json::Value;

use petgraph::{dot, graph::NodeIndex};

use anyhow::{format_err, Result};
use slog_scope::info;

use std::fs;

use crate::dfs;
use crate::{graph::CachedStableGraph, merge_views, url_norm::FromJson};

pub struct CustomCommandProvider {
    commands: HashMap<String, Box<dyn Invokeable>>,
}

impl CustomCommandProvider {
    pub fn new(commands: Vec<(&str, Box<dyn Invokeable>)>) -> CustomCommandProvider {
        CustomCommandProvider {
            commands: commands.into_iter().map(|tup| (tup.0.into(), tup.1)).collect(),
        }
    }

    pub fn execute(&self, command: &str, args: &[Value], root_path: &Path) -> Result<Value> {
        if self.commands.contains_key(command) {
            return self.commands.get(command).unwrap().run_command(root_path, args);
        }
        Err(format_err!("command doesn't exist"))
    }
}

pub trait Invokeable {
    fn run_command(&self, root: &Path, arguments: &[Value]) -> Result<Value>;
}

pub struct GraphDotCommand {
    pub graph: Rc<RefCell<CachedStableGraph>>,
}

impl Invokeable for GraphDotCommand {
    fn run_command(&self, root: &Path, _: &[Value]) -> Result<Value> {
        let filepath = root.join("graph.dot");

        info!("generating dot file"; "path" => filepath.as_os_str().to_str());

        let mut file = OpenOptions::new().truncate(true).write(true).create(true).open(filepath).unwrap();

        let mut write_data_closure = || -> Result<(), std::io::Error> {
            let graph = self.graph.as_ref();

            file.seek(std::io::SeekFrom::Start(0))?;
            file.write_all("digraph {\n\tgraph [splines=ortho]\n\tnode [shape=box]\n".as_bytes())?;
            file.write_all(dot::Dot::with_config(&graph.borrow().graph, &[Config::GraphContentOnly]).to_string().as_bytes())?;
            file.write_all("\n}".as_bytes())?;
            file.flush()?;
            file.seek(std::io::SeekFrom::Start(0))?;
            Ok(())
        };

        match write_data_closure() {
            Err(err) => Err(format_err!("error generating graphviz data: {}", err)),
            _ => Ok(Value::Null),
        }
    }
}

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

    pub fn get_dfs_for_node(&self, root: NodeIndex) -> Result<Vec<(NodeIndex, Option<NodeIndex>)>, dfs::error::CycleError> {
        let graph_ref = self.graph.borrow();

        let dfs = dfs::Dfs::new(&graph_ref, root);

        dfs.collect::<Result<Vec<_>, _>>()
    }

    pub fn load_sources(&self, nodes: &[(NodeIndex, Option<NodeIndex>)]) -> Result<HashMap<PathBuf, String>> {
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
