use std::fs::OpenOptions;
use std::io::prelude::*;

use filesystem::NormalizedPathBuf;
use graph::{CachedStableGraph, Config, dot};
use logging::info;
// use opengl::IncludePosition;
// use serde_json::Value;

// use anyhow::{format_err, Result};

// pub(crate) fn run(root: &NormalizedPathBuf, graph: &CachedStableGraph<NormalizedPathBuf, IncludePosition>) -> Result<Option<Value>> {
//     let filepath = root.join("graph.dot");

//     info!("generating dot file"; "path" => &filepath);

//     let mut file = OpenOptions::new().truncate(true).write(true).create(true).open(filepath).unwrap();

//     let mut write_data_closure = || -> Result<(), std::io::Error> {
//         file.seek(std::io::SeekFrom::Start(0))?;
//         file.write_all("digraph {\n\tgraph [splines=ortho]\n\tnode [shape=box]\n".as_bytes())?;
//         file.write_all(
//             dot::Dot::with_config(&graph.graph, &[Config::GraphContentOnly])
//                 .to_string()
//                 .as_bytes(),
//         )?;
//         file.write_all("\n}".as_bytes())?;
//         file.flush()?;
//         file.seek(std::io::SeekFrom::Start(0))?;
//         Ok(())
//     };

//     match write_data_closure() {
//         Err(err) => Err(format_err!("error generating graphviz data: {}", err)),
//         _ => Ok(None),
//     }
// }
