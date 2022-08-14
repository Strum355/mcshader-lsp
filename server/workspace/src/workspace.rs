use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use filesystem::{LFString, NormalizedPathBuf};
use graph::{dfs, CachedStableGraph, FilialTuple, NodeIndex};
use include_merger::MergeViewBuilder;
use logging::{info, logger, warn, FutureExt};
use opengl::{diagnostics_parser::DiagnosticsParser, TreeType};
use sourcefile::{IncludeLine, SourceFile, SourceMapper};
use tokio::sync::Mutex;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};
use url::Url;
use workspace_tree::TreeError;

pub struct Workspace<S: opengl::ShaderValidator> {
    pub root: NormalizedPathBuf,
    workspace_view: Arc<Mutex<workspace_tree::WorkspaceTree>>,
    // graph: Arc<Mutex<CachedStableGraph<NormalizedPathBuf, IncludePosition>>>,
    gl_context: Arc<Mutex<S>>,
}

impl<S: opengl::ShaderValidator> Workspace<S> {
    pub fn new(root: NormalizedPathBuf, gl: S) -> Self {
        Workspace {
            workspace_view: Arc::new(Mutex::new(workspace_tree::WorkspaceTree::new(&root))),
            root,
            // graph: Arc::new(Mutex::new(CachedStableGraph::new())),
            gl_context: Arc::new(Mutex::new(gl)),
        }
    }

    pub async fn build(&self) {
        info!("initializing workspace"; "root" => &self.root);

        let mut tree = self.workspace_view.lock().with_logger(logger()).await;
        tree.build();

        info!("build graph"; "connected" => tree.num_connected_entries()/* , "disconnected" => tree.num_disconnected_entries() */);
    }

    pub async fn refresh_graph_for_file(&self, path: &NormalizedPathBuf) {
        let mut tree = self.workspace_view.lock().with_logger(logger()).await;

        tree.update_sourcefile(path);
    }

    pub async fn lint(&self, path: &NormalizedPathBuf) -> Result<HashMap<Url, Vec<Diagnostic>>> {
        let mut workspace = self.workspace_view.lock().with_logger(logger()).await;

        // the set of all filepath->content.
        let mut all_sources: HashMap<NormalizedPathBuf, LFString> = HashMap::new();
        // the set of filepath->list of diagnostics to report
        let mut diagnostics: HashMap<Url, Vec<Diagnostic>> = HashMap::new();

        // we want to backfill the diagnostics map with all linked sources
        let back_fill = |all_sources: &HashMap<NormalizedPathBuf, LFString>, diagnostics: &mut HashMap<Url, Vec<Diagnostic>>| {
            for path in all_sources.keys() {
                diagnostics.entry(Url::from_file_path(path).unwrap()).or_default();
            }
        };

        let trees = match workspace.trees_for_entry(path) {
            Ok(trees) => trees,
            Err(err) => {
                match err {
                    TreeError::NonTopLevel(e) => warn!("got a non-valid toplevel file"; "root_ancestor" => e, "stripped" => e.strip_prefix(&self.root), "path" => path),
                    e => return Err(e.into()),
                }
                back_fill(&all_sources, &mut diagnostics);
                return Ok(diagnostics);
            }
        }
        .collect::<Vec<_>>();

        for tree in trees {
            let mut tree = match tree {
                Ok(t) => t.peekable(),
                Err(e) => match e {
                    // dont care, didnt ask, skip
                    TreeError::NonTopLevel(_) => continue,
                    e => unreachable!("unexpected error {:?}", e),
                },
            };
            // let tree = match tree.and_then(|t| t.collect::<Result<Vec<_>, TreeError>>()) {
            //     Ok(t) => t,
            //     Err(e) => {
            //         match e {
            //             TreeError::NonTopLevel(f) => {
            //                 warn!("got a non-valid toplevel file"; "root_ancestor" => f, "stripped" => f.strip_prefix(&self.root));
            //                 continue;
            //             }
            //             TreeError::FileNotFound(f) => {
            //                 warn!("child not found"; "child" => f);
            //                 continue;
            //             }
            //             TreeError::DfsError(e) => {
            //                 diagnostics.insert(Url::from_file_path(path).unwrap(), vec![e.into()]);
            //                 back_fill(&all_sources, &mut diagnostics); // TODO: confirm
            //                 return Ok(diagnostics);
            //             }
            //             e => unreachable!("should only yield non-toplevel file error, got {:?}", e),
            //         };
            //     }
            // };

            let tree_size = tree.size_hint().0;

            let mut source_mapper = SourceMapper::new(tree_size); // very rough count

            let root = tree
                .peek()
                .expect("expected non-zero sized tree")
                .as_ref()
                .expect("unexpected cycle or not-found node")
                .child;

            let (tree_type, document_glsl_version) = (
                root.path.extension().unwrap().into(),
                root.version().expect("fatal error parsing file for include version"),
            );

            let mut built_tree = Vec::with_capacity(tree_size);
            for entry in tree {
                match entry {
                    Ok(node) => built_tree.push(node),
                    Err(e) => match e {
                        TreeError::FileNotFound {
                            ref importing,
                            ref missing,
                        } => diagnostics
                            .entry(Url::from_file_path(importing).unwrap())
                            .or_default()
                            .push(Diagnostic {
                                range: Range::new(Position::new(0, 0), Position::new(0, u32::MAX)),
                                severity: Some(DiagnosticSeverity::WARNING),
                                source: Some("mcglsl".to_string()),
                                message: e.to_string(),
                                ..Diagnostic::default()
                            }),
                        TreeError::DfsError(_) => todo!(),
                        e => unreachable!("unexpected error {:?}", e),
                    },
                }
            }

            let view = MergeViewBuilder::new(
                &built_tree,
                &mut source_mapper,
                self.gl_context.lock().with_logger(logger()).await.vendor().as_str().into(),
                document_glsl_version,
            )
            .build();

            let stdout = match self.compile_shader_source(&view, tree_type, path).with_logger(logger()).await {
                Some(s) => s,
                None => {
                    back_fill(&all_sources, &mut diagnostics);
                    return Ok(diagnostics);
                }
            };

            diagnostics.extend(
                DiagnosticsParser::new(&*self.gl_context.lock().with_logger(logger()).await).parse_diagnostics_output(
                    stdout,
                    path,
                    &source_mapper,
                ),
            );
        }

        back_fill(&all_sources, &mut diagnostics);
        Ok(diagnostics)
    }

    async fn compile_shader_source(&self, source: &str, tree_type: TreeType, path: &NormalizedPathBuf) -> Option<String> {
        let result = self.gl_context.lock().with_logger(logger()).await.validate(tree_type, source);
        match &result {
            Some(output) => {
                info!("compilation errors reported"; "errors" => format!("`{}`", output.replace('\n', "\\n")), "tree_root" => path)
            }
            None => info!("compilation reported no errors"; "tree_root" => path),
        };
        result
    }
}
