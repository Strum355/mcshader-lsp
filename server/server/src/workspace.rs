use std::{collections::HashMap, sync::Arc};

use filesystem::NormalizedPathBuf;
use graph::dfs::CycleError;
use include_merger::MergeViewBuilder;
use logging::{info, logger, warn, FutureExt};
use opengl::{diagnostics_parser::DiagnosticsParser, GPUVendor, TreeType};
use sourcefile::{SourceMapper, IncludeLine};
use tokio::sync::Mutex;
use tower_lsp::lsp_types::Diagnostic;
use url::Url;
use workspace::TreeError;

pub struct Workspace<S: opengl::ShaderValidator> {
    pub(super) root: NormalizedPathBuf,
    pub(super) workspace_view: Arc<Mutex<workspace::WorkspaceTree>>,
    gl_context: Arc<Mutex<S>>,
}

impl<S: opengl::ShaderValidator> Workspace<S> {
    pub fn new(root: NormalizedPathBuf, gl: S) -> Self {
        Workspace {
            workspace_view: Arc::new(Mutex::new(workspace::WorkspaceTree::new(&root))),
            gl_context: Arc::new(Mutex::new(gl)),
            root,
        }
    }

    pub async fn build(&self) {
        info!("initializing workspace"; "root" => &self.root);

        let mut tree = self.workspace_view.lock().with_logger(logger()).await;
        tree.build();

        info!("build graph"; "connected" => tree.num_connected_entries()/* , "disconnected" => tree.num_disconnected_entries() */);
    }

    pub async fn delete_sourcefile(&self, path: &NormalizedPathBuf) -> Result<HashMap<Url, Vec<Diagnostic>>, anyhow::Error> {
        info!("path deleted on filesystem"; "path" => path);
        let mut workspace = self.workspace_view.lock().with_logger(logger()).await;

        // need to get the old trees first so we know what to lint to remove now stale diagnostics
        let old_roots: Vec<NormalizedPathBuf> = match workspace.trees_for_entry(path) {
            Ok(trees) => trees,
            Err(_) => {
                warn!("path not known to the workspace, this might be a bug"; "path" => path);
                return Ok(HashMap::new());
            }
        }
        .into_iter()
        .filter_map(|maybe_tree| maybe_tree.ok())
        // want to extract the root of each tree so we can build the trees _after_ removing the deleted file
        .map(|mut tree| tree.next().expect("unexpected zero-sized tree").unwrap().child.path.clone())
        .collect::<Vec<_>>();

        info!("found existing roots"; "roots" => format!("{:?}", old_roots));

        workspace.remove_sourcefile(path);

        let mut all_diagnostics: HashMap<Url, Vec<Diagnostic>> = HashMap::new();

        for old_root in old_roots {
            let new_trees = workspace.trees_for_entry(&old_root).expect("should be a known existing path");
            assert_eq!(new_trees.len(), 1, "root should not be able to yield more than one tree");
            let tree = new_trees
                .into_iter()
                .next()
                .unwrap()
                .expect("should be a top-level path")
                .map(|tree| match tree {
                    Ok(t) => Ok(Ok(t)),
                    Err(TreeError::DfsError(e)) => Err(e),
                    Err(TreeError::FileNotFound(e)) => Ok(Err(e)),
                })
                .collect();
            all_diagnostics.extend(self.lint(path, tree).with_logger(logger()).await);
        }

        Ok(all_diagnostics)
    }

    pub async fn update_sourcefile(&self, path: &NormalizedPathBuf, text: String) -> Result<HashMap<Url, Vec<Diagnostic>>, anyhow::Error> {
        let mut workspace = self.workspace_view.lock().with_logger(logger()).await;

        workspace.update_sourcefile(path, text);

        let mut all_diagnostics: HashMap<Url, Vec<Diagnostic>> = HashMap::new();

        for tree in match workspace.trees_for_entry(path) {
            Ok(trees) => trees,
            Err(err) => {
                return Err(err.into());
                // back_fill(Box::new(all_sources.keys()), &mut diagnostics);
                // return Ok(diagnostics);
            }
        }
        .into_iter()
        // filter out roots that aren't valid top level ones, we cant lint them
        .filter_map(|maybe_tree| maybe_tree.ok())
        .map(|tree| {
            tree.map(|tree| match tree {
                Ok(t) => Ok(Ok(t)),
                Err(TreeError::DfsError(e)) => Err(e),
                Err(TreeError::FileNotFound(e)) => Ok(Err(e)),
            })
            .collect()
        }) {
            all_diagnostics.extend(self.lint(path, tree).with_logger(logger()).await);
        }

        Ok(all_diagnostics)
    }

    async fn lint<'a>(
        &'a self, path: &'a NormalizedPathBuf, tree: Result<workspace::MaterializedTree<'a>, CycleError<NormalizedPathBuf, IncludeLine>>,
    ) -> HashMap<Url, Vec<Diagnostic>> {
        // the set of filepath->list of diagnostics to report
        let mut diagnostics: HashMap<Url, Vec<Diagnostic>> = HashMap::new();

        // we want to backfill the diagnostics map with all linked sources
        let back_fill = |all_sources: &[&NormalizedPathBuf], diagnostics: &mut HashMap<Url, Vec<Diagnostic>>| {
            for path in all_sources {
                eprintln!(
                    "BACKFILLING FOR {:?}, EXISTS {}",
                    path,
                    diagnostics.contains_key(&Url::from_file_path(path).unwrap())
                );
                diagnostics.entry(Url::from_file_path(path).unwrap()).or_default();
            }
        };

        let gpu_vendor: GPUVendor = self.gl_context.lock().with_logger(logger()).await.vendor().as_str().into();

        let tree: Vec<_> = match tree {
            Ok(tree) => tree.into_iter().filter_map(|tup| tup.ok()).collect(),
            Err(ref cycle) => {
                let cycle_diagnostics: Vec<_> = cycle.into();
                for (node, diagnostic) in cycle.path().iter().zip(cycle_diagnostics.into_iter()) {
                    diagnostics
                        .entry(Url::from_file_path(node).unwrap())
                        .or_default()
                        .push(diagnostic);
                }
                return diagnostics;
            }
        };

        let mut source_mapper = SourceMapper::new(tree.len());

        let (tree_type, document_glsl_version) = {
            let root = tree.first().expect("expected non-zero sized tree").child;
            (
                root.path.extension().unwrap().into(),
                root.version().expect("fatal error parsing file for include version"),
            )
        };

        let view = MergeViewBuilder::new(&self.root, &tree, &mut source_mapper).build();

        let stdout = match self.compile_shader_source(&view, tree_type, path).with_logger(logger()).await {
            Some(s) => s,
            None => {
                let paths: Vec<_> = tree.iter().map(|s| &s.child.path).collect();
                back_fill(&paths, &mut diagnostics);
                return diagnostics;
            }
        };

        for diagnostic in DiagnosticsParser::new(gpu_vendor, document_glsl_version).parse_diagnostics_output(
            stdout,
            path,
            &source_mapper,
            &tree.iter().map(|tup| (&tup.child.path, tup.child)).collect(),
        ) {
            diagnostics.entry(diagnostic.0).or_default().extend(diagnostic.1);
        }
        let paths: Vec<_> = tree.iter().map(|s| &s.child.path).collect();
        back_fill(&paths, &mut diagnostics);

        eprintln!("DIAGS {:?}", diagnostics);
        // back_fill(Box::new(all_sources.keys()), &mut diagnostics);
        diagnostics
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
