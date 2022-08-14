use std::collections::{hash_map::Entry, HashMap};

use filesystem::{LFString, NormalizedPathBuf};
use graph::{dfs, CachedStableGraph};
use logging::{logger, FutureExt};
// use opengl::{merge_views, source_mapper::SourceMapper, IncludePosition};
// use serde_json::Value;

// use anyhow::{format_err, Result};

// pub async fn run(path: &NormalizedPathBuf, graph: &mut CachedStableGraph<NormalizedPathBuf, IncludePosition>) -> Result<Option<Value>> {
//     if graph.root_ancestors_for_key(path)?.is_none() {
//         return Err(format_err!("'{}' is not a top-level file aka has ancestors", path));
//     };

//     //info!("ancestors for {}:\n\t{:?}", path, file_ancestors.iter().map(|e| graph.borrow().graph.node_weight(*e).unwrap().clone()).collect::<Vec<String>>());

//     // if we are a top-level file (this has to be one of the set defined by Optifine, right?)
//     // gather the list of all descendants
//     let root = graph.find_node(path).unwrap();

//     let mut sources = HashMap::new();

//     let tree = dfs::Dfs::new(graph, root)
//         .map(|result| {
//             let node = result?;
//             let path = &graph[node.child];
//             if let Entry::Vacant(entry) = sources.entry(path.clone()) {
//                 let source = futures::executor::block_on(async { LFString::read(path).with_logger(logger()).await })?;
//                 entry.insert(source);
//             };
//             Ok(node)
//         })
//         .collect::<Result<Vec<_>>>()?;

//     let mut source_mapper = SourceMapper::new(sources.len());
//     let view = merge_views::MergeViewBuilder::new(&tree, &sources, graph, &mut source_mapper).build();

//     eprintln!("{:?}", view);

//     Ok(Some(serde_json::value::Value::String(view.to_string())))
// }
