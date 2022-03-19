use std::cell::RefCell;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;
use std::rc::Rc;

use petgraph::dot::Config;
use serde_json::Value;

use petgraph::dot;

use anyhow::{format_err, Result};
use slog_scope::info;

use crate::graph::CachedStableGraph;

use super::Invokeable;

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
            file.write_all(
                dot::Dot::with_config(&graph.borrow().graph, &[Config::GraphContentOnly])
                    .to_string()
                    .as_bytes(),
            )?;
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
