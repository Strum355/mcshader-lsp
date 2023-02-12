#![feature(once_cell)]
#![feature(option_get_or_insert_default)]

use merge_views::FilialTuple;
use rust_lsp::jsonrpc::{method_types::*, *};
use rust_lsp::lsp::*;
use rust_lsp::lsp_types::{notification::*, *};

use petgraph::stable_graph::NodeIndex;
use path_slash::PathExt;

use serde::Deserialize;
use serde_json::{from_value, Value};

use tree_sitter::Parser;
use url_norm::FromUrl;

use walkdir::WalkDir;

use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::io::{stdin, stdout, BufRead, BufReader};
use std::iter::{Extend, FromIterator};
use std::rc::Rc;
use std::str::FromStr;

use std::{
    cell::RefCell,
    path::{Path, PathBuf},
};

use slog::Level;
use slog_scope::{debug, error, info, warn};

use path_slash::PathBufExt;

use anyhow::{anyhow, Result};

use regex::Regex;

use lazy_static::lazy_static;

mod commands;
mod configuration;
mod consts;
mod dfs;
mod diagnostics_parser;
mod graph;
mod linemap;
mod lsp_ext;
mod merge_views;
mod navigation;
mod opengl;
mod source_mapper;
mod url_norm;

#[cfg(test)]
mod test;

pub fn is_top_level(path: &Path) -> bool {
    let path = path.to_slash().unwrap();
    if !RE_WORLD_FOLDER.is_match(&path) {
        return false;
    }
    let parts: Vec<&str> = path.split("/").collect();
    let len = parts.len();
    (len == 3 || len == 2) && TOPLEVEL_FILES.contains(parts[len - 1])
}

lazy_static! {
    static ref RE_INCLUDE: Regex = Regex::new(r#"^(?:\s)*?(?:#include) "(.+)"\r?"#).unwrap();
    static ref RE_WORLD_FOLDER: Regex = Regex::new(r#"^shaders(/world-?\d+)?"#).unwrap();
    static ref TOPLEVEL_FILES: HashSet<String> = {
        let mut set = HashSet::with_capacity(1716);
        for ext in ["fsh", "vsh", "gsh", "csh"] {
            set.insert(format!("composite.{}", ext));
            set.insert(format!("deferred.{}", ext));
            set.insert(format!("prepare.{}", ext));
            set.insert(format!("shadowcomp.{}", ext));
            for i in 1..=99 {
                set.insert(format!("composite{}.{}", i, ext));
                set.insert(format!("deferred{}.{}", i, ext));
                set.insert(format!("prepare{}.{}", i, ext));
                set.insert(format!("shadowcomp{}.{}", i, ext));
            }
            set.insert(format!("composite_pre.{}", ext));
            set.insert(format!("deferred_pre.{}", ext));
            set.insert(format!("final.{}", ext));
            set.insert(format!("gbuffers_armor_glint.{}", ext));
            set.insert(format!("gbuffers_basic.{}", ext));
            set.insert(format!("gbuffers_beaconbeam.{}", ext));
            set.insert(format!("gbuffers_block.{}", ext));
            set.insert(format!("gbuffers_clouds.{}", ext));
            set.insert(format!("gbuffers_damagedblock.{}", ext));
            set.insert(format!("gbuffers_entities.{}", ext));
            set.insert(format!("gbuffers_entities_glowing.{}", ext));
            set.insert(format!("gbuffers_hand.{}", ext));
            set.insert(format!("gbuffers_hand_water.{}", ext));
            set.insert(format!("gbuffers_item.{}", ext));
            set.insert(format!("gbuffers_line.{}", ext));
            set.insert(format!("gbuffers_skybasic.{}", ext));
            set.insert(format!("gbuffers_skytextured.{}", ext));
            set.insert(format!("gbuffers_spidereyes.{}", ext));
            set.insert(format!("gbuffers_terrain.{}", ext));
            set.insert(format!("gbuffers_terrain_cutout.{}", ext));
            set.insert(format!("gbuffers_terrain_cutout_mip.{}", ext));
            set.insert(format!("gbuffers_terrain_solid.{}", ext));
            set.insert(format!("gbuffers_textured.{}", ext));
            set.insert(format!("gbuffers_textured_lit.{}", ext));
            set.insert(format!("gbuffers_water.{}", ext));
            set.insert(format!("gbuffers_weather.{}", ext));
            set.insert(format!("shadow.{}", ext));
            set.insert(format!("shadow_cutout.{}", ext));
            set.insert(format!("shadow_solid.{}", ext));
        }
        let base_char_num = 'a' as u8;
        for suffix_num in 0u8..=25u8 {
            let suffix_char = (base_char_num + suffix_num) as char;
            set.insert(format!("composite_{}.csh", suffix_char));
            set.insert(format!("deferred_{}.csh", suffix_char));
            set.insert(format!("prepare_{}.csh", suffix_char));
            set.insert(format!("shadowcomp_{}.csh", suffix_char));
            for i in 1..=99 {
                let total_suffix = format!("{}_{}", i, suffix_char);
                set.insert(format!("composite{}.csh", total_suffix));
                set.insert(format!("deferred{}.csh", total_suffix));
                set.insert(format!("prepare{}.csh", total_suffix));
                set.insert(format!("shadowcomp{}.csh", total_suffix));
            }
        }
        set
    };
}

fn main() {
    let guard = logging::set_logger_with_level(Level::Info);

    let endpoint_output = LSPEndpoint::create_lsp_output_with_output_stream(stdout);

    let cache_graph = graph::CachedStableGraph::new();

    let mut parser = Parser::new();
    parser.set_language(tree_sitter_glsl::language()).unwrap();

    let mut langserver = MinecraftShaderLanguageServer {
        endpoint: endpoint_output.clone(),
        graph: Rc::new(RefCell::new(cache_graph)),
        root: "".into(),
        command_provider: None,
        opengl_context: Rc::new(opengl::OpenGlContext::new()),
        tree_sitter: Rc::new(RefCell::new(parser)),
        log_guard: Some(guard),
    };

    langserver.command_provider = Some(commands::CustomCommandProvider::new(vec![
        (
            "graphDot",
            Box::new(commands::graph_dot::GraphDotCommand {
                graph: langserver.graph.clone(),
            }),
        ),
        (
            "virtualMerge",
            Box::new(commands::merged_includes::VirtualMergedDocument {
                graph: langserver.graph.clone(),
            }),
        ),
        (
            "parseTree",
            Box::new(commands::parse_tree::TreeSitterSExpr {
                tree_sitter: langserver.tree_sitter.clone(),
            }),
        ),
    ]));

    LSPEndpoint::run_server_from_input(&mut stdin().lock(), endpoint_output, langserver);
}

pub struct MinecraftShaderLanguageServer {
    endpoint: Endpoint,
    graph: Rc<RefCell<graph::CachedStableGraph>>,
    root: PathBuf,
    command_provider: Option<commands::CustomCommandProvider>,
    opengl_context: Rc<dyn opengl::ShaderValidator>,
    tree_sitter: Rc<RefCell<Parser>>,
    log_guard: Option<slog_scope::GlobalLoggerGuard>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct IncludePosition {
    // the 0-indexed line on which the include lives.
    line: usize,
    // the 0-indexed char offset defining the start of the include path string.
    start: usize,
    // the 0-indexed char offset defining the end of the include path string.
    end: usize,
}

impl Debug for IncludePosition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{line: {}}}", self.line)
    }
}

impl Display for IncludePosition {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{{line: {}}}", self.line)
    }
}

#[derive(Debug)]
pub enum TreeType {
    Fragment,
    Vertex,
    Geometry,
    Compute,
}

impl MinecraftShaderLanguageServer {
    pub fn error_not_available<DATA>(data: DATA) -> MethodError<DATA> {
        let msg = "Functionality not implemented.".to_string();
        MethodError::<DATA> {
            code: 1,
            message: msg,
            data,
        }
    }

    fn build_initial_graph(&self) {
        info!("generating graph for current root"; "root" => self.root.to_str().unwrap());

        // filter directories and files not ending in any of the 3 extensions
        WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|entry| {
                if entry.is_err() {
                    return None;
                }

                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_dir() {
                    return None;
                }

                let ext = match path.extension() {
                    Some(e) => e,
                    None => return None,
                };

                // TODO: include user added extensions with a set
                if ext != "vsh" && ext != "fsh" && ext  != "csh" && ext != "gsh" && ext != "glsl" && ext != "inc" {
                    return None;
                }

                Some(entry.into_path())
            })
            .for_each(|path| {
                // iterate all valid found files, search for includes, add a node into the graph for each
                // file and add a file->includes KV into the map
                self.add_file_and_includes_to_graph(&path);
            });

        info!("finished building project include graph");
    }

    fn add_file_and_includes_to_graph(&self, path: &Path) {
        let includes = self.find_includes(path);

        let idx = self.graph.borrow_mut().add_node(path);

        debug!("adding includes for new file"; "file" => path.to_str().unwrap(), "includes" => format!("{:?}", includes));
        for include in includes {
            self.add_include(include, idx);
        }
    }

    fn add_include(&self, include: (PathBuf, IncludePosition), node: NodeIndex) {
        let child = self.graph.borrow_mut().add_node(&include.0);
        self.graph.borrow_mut().add_edge(node, child, include.1);
    }

    pub fn find_includes(&self, file: &Path) -> Vec<(PathBuf, IncludePosition)> {
        let mut includes = Vec::default();

        let buf = BufReader::new(std::fs::File::open(file).unwrap());
        buf.lines()
            .enumerate()
            .filter_map(|line| match line.1 {
                Ok(t) => Some((line.0, t)),
                Err(_e) => None,
            })
            .filter(|line| RE_INCLUDE.is_match(line.1.as_str()))
            .for_each(|line| {
                let cap = RE_INCLUDE.captures(line.1.as_str()).unwrap().get(1).unwrap();

                let start = cap.start();
                let end = cap.end();
                let mut path: String = cap.as_str().into();

                let full_include = if path.starts_with('/') {
                    path = path.strip_prefix('/').unwrap().to_string();
                    self.root.join("shaders").join(PathBuf::from_slash(&path))
                } else {
                    file.parent().unwrap().join(PathBuf::from_slash(&path))
                };

                includes.push((full_include, IncludePosition { line: line.0, start, end }));
            });

        includes
    }

    fn update_includes(&self, file: &Path) {
        let includes = self.find_includes(file);

        info!("includes found for file"; "file" => file.to_str().unwrap(), "includes" => format!("{:?}", includes));

        let idx = match self.graph.borrow_mut().find_node(file) {
            None => return,
            Some(n) => n,
        };

        let prev_children: HashSet<_> = HashSet::from_iter(self.graph.borrow().get_all_child_positions(idx).map(|tup| {
            (self.graph.borrow().get_node(tup.0), tup.1)
        }));
        let new_children: HashSet<_> = includes.iter().cloned().collect();

        let to_be_added = new_children.difference(&prev_children);
        let to_be_removed = prev_children.difference(&new_children);

        debug!(
            "include sets diff'd";
            "for removal" => format!("{:?}", to_be_removed),
            "for addition" => format!("{:?}", to_be_added)
        );

        for removal in to_be_removed {
            let child = self.graph.borrow_mut().find_node(&removal.0).unwrap();
            self.graph.borrow_mut().remove_edge(idx, child, removal.1);
        }

        for insertion in to_be_added {
            self.add_include(includes.iter().find(|f| f.0 == *insertion.0).unwrap().clone(), idx);
        }
    }

    pub fn lint(&self, uri: &Path) -> Result<HashMap<Url, Vec<Diagnostic>>> {
        // get all top level ancestors of this file
        let file_ancestors = match self.get_file_toplevel_ancestors(uri) {
            Ok(opt) => match opt {
                Some(ancestors) => ancestors,
                None => vec![],
            },
            Err(e) => return Err(e),
        };

        info!(
            "top-level file ancestors found";
            "uri" => uri.to_str().unwrap(),
            "ancestors" => format!("{:?}", file_ancestors
                .iter()
                .map(|e| PathBuf::from_str(
                    &self.graph.borrow().graph[*e].clone()
                )
                .unwrap())
                .collect::<Vec<PathBuf>>())
        );

        // the set of all filepath->content.
        let mut all_sources: HashMap<PathBuf, String> = HashMap::new();
        // the set of filepath->list of diagnostics to report
        let mut diagnostics: HashMap<Url, Vec<Diagnostic>> = HashMap::new();

        // we want to backfill the diagnostics map with all linked sources
        let back_fill = |all_sources: &HashMap<PathBuf, String>, diagnostics: &mut HashMap<Url, Vec<Diagnostic>>| {
            for path in all_sources.keys() {
                diagnostics.entry(Url::from_file_path(path).unwrap()).or_default();
            }
        };

        // if we are a top-level file (this has to be one of the set defined by Optifine, right?)
        if file_ancestors.is_empty() {
            // gather the list of all descendants
            let root = self.graph.borrow_mut().find_node(uri).unwrap();
            let tree = match self.get_dfs_for_node(root) {
                Ok(tree) => tree,
                Err(e) => {
                    diagnostics.insert(Url::from_file_path(uri).unwrap(), vec![e.into()]);
                    return Ok(diagnostics);
                }
            };

            all_sources.extend(self.load_sources(&tree)?);

            let mut source_mapper = source_mapper::SourceMapper::new(all_sources.len());

            let view = {
                let graph = self.graph.borrow();
                let merged_string = {
                    merge_views::MergeViewBuilder::new(&tree, &all_sources, &graph, &mut source_mapper).build()
                };
                merged_string
            };

            let root_path = self.graph.borrow().get_node(root);
            let ext = match root_path.extension() {
                Some(ext) => ext.to_str().unwrap(),
                None => {
                    back_fill(&all_sources, &mut diagnostics);
                    return Ok(diagnostics);
                }
            };

            if !is_top_level(root_path.strip_prefix(&self.root).unwrap()) {
                warn!("got a non-valid toplevel file"; "root_ancestor" => root_path.to_str().unwrap(), "stripped" => root_path.strip_prefix(&self.root).unwrap().to_str().unwrap());
                back_fill(&all_sources, &mut diagnostics);
                return Ok(diagnostics);
            }

            let tree_type = if ext == "fsh" {
                TreeType::Fragment
            } else if ext == "vsh" {
                TreeType::Vertex
            } else if ext == "gsh" {
                TreeType::Geometry
            } else if ext == "csh" {
                TreeType::Compute
            } else {
                unreachable!();
            };

            let stdout = match self.compile_shader_source(&view, tree_type, &root_path) {
                Some(s) => s,
                None => {
                    back_fill(&all_sources, &mut diagnostics);
                    return Ok(diagnostics);
                }
            };

            let diagnostics_parser = diagnostics_parser::DiagnosticsParser::new(self.opengl_context.as_ref());

            diagnostics.extend(diagnostics_parser.parse_diagnostics_output(stdout, uri, &source_mapper, &self.graph.borrow()));
        } else {
            let mut all_trees: Vec<(TreeType, Vec<FilialTuple>)> = Vec::new();

            for root in &file_ancestors {
                let nodes = match self.get_dfs_for_node(*root) {
                    Ok(nodes) => nodes,
                    Err(e) => {
                        diagnostics.insert(Url::from_file_path(uri).unwrap(), vec![e.into()]);
                        back_fill(&all_sources, &mut diagnostics); // TODO: confirm
                        return Ok(diagnostics);
                    }
                };

                let root_path = self.graph.borrow().get_node(*root).clone();
                let ext = match root_path.extension() {
                    Some(ext) => ext.to_str().unwrap(),
                    None => continue,
                };

                if !is_top_level(root_path.strip_prefix(&self.root).unwrap()) {
                    warn!("got a non-valid toplevel file"; "root_ancestor" => root_path.to_str().unwrap(), "stripped" => root_path.strip_prefix(&self.root).unwrap().to_str().unwrap());
                    continue;
                }

                let tree_type = if ext == "fsh" {
                    TreeType::Fragment
                } else if ext == "vsh" {
                    TreeType::Vertex
                } else if ext == "gsh" {
                    TreeType::Geometry
                } else if ext == "csh" {
                    TreeType::Compute
                } else {
                    unreachable!();
                };

                let sources = self.load_sources(&nodes)?;
                all_trees.push((tree_type, nodes));
                all_sources.extend(sources);
            }

            for tree in all_trees {
                // bit over-zealous in allocation but better than having to resize
                let mut source_mapper = source_mapper::SourceMapper::new(all_sources.len());
                let view = {
                    let graph = self.graph.borrow();
                    let merged_string = {
                        merge_views::MergeViewBuilder::new(&tree.1, &all_sources, &graph, &mut source_mapper).build()
                    };
                    merged_string
                };

                let root_path = self.graph.borrow().get_node(tree.1.first().unwrap().child);
                let stdout = match self.compile_shader_source(&view, tree.0, &root_path) {
                    Some(s) => s,
                    None => continue,
                };

                let diagnostics_parser = diagnostics_parser::DiagnosticsParser::new(self.opengl_context.as_ref());

                diagnostics.extend(diagnostics_parser.parse_diagnostics_output(stdout, uri, &source_mapper, &self.graph.borrow()));
            }
        };

        back_fill(&all_sources, &mut diagnostics);
        Ok(diagnostics)
    }

    fn compile_shader_source(&self, source: &str, tree_type: TreeType, path: &Path) -> Option<String> {
        let result = self.opengl_context.clone().validate(tree_type, source);
        match &result {
            Some(output) => {
                info!("compilation errors reported"; "errors" => format!("`{}`", output.replace('\n', "\\n")), "tree_root" => path.to_str().unwrap())
            }
            None => info!("compilation reported no errors"; "tree_root" => path.to_str().unwrap()),
        };
        result
    }

    pub fn get_dfs_for_node(&self, root: NodeIndex) -> Result<Vec<FilialTuple>, dfs::error::CycleError> {
        let graph_ref = self.graph.borrow();

        let dfs = dfs::Dfs::new(&graph_ref, root);

        dfs.collect::<Result<_, _>>()
    }

    pub fn load_sources(&self, nodes: &[FilialTuple]) -> Result<HashMap<PathBuf, String>> {
        let mut sources = HashMap::new();

        for node in nodes {
            let graph = self.graph.borrow();
            let path = graph.get_node(node.child);

            if sources.contains_key(&path) {
                continue;
            }

            let source = match fs::read_to_string(&path) {
                Ok(s) => s,
                Err(e) => return Err(anyhow!("error reading {:?}: {}", path, e)),
            };
            let source = source.replace("\r\n", "\n");
            sources.insert(path.clone(), source);
        }

        Ok(sources)
    }

    fn get_file_toplevel_ancestors(&self, uri: &Path) -> Result<Option<Vec<petgraph::stable_graph::NodeIndex>>> {
        let curr_node = match self.graph.borrow_mut().find_node(uri) {
            Some(n) => n,
            None => return Err(anyhow!("node not found {:?}", uri)),
        };
        let roots = self.graph.borrow().collect_root_ancestors(curr_node);
        if roots.is_empty() {
            return Ok(None);
        }
        Ok(Some(roots))
    }

    pub fn publish_diagnostic(&self, diagnostics: HashMap<Url, Vec<Diagnostic>>, document_version: Option<i32>) {
        // info!("DIAGNOSTICS:\n{:?}", diagnostics);
        for (uri, diagnostics) in diagnostics {
            self.endpoint
                .send_notification(
                    PublishDiagnostics::METHOD,
                    PublishDiagnosticsParams {
                        uri,
                        diagnostics,
                        version: document_version,
                    },
                )
                .expect("failed to publish diagnostics");
        }
    }

    fn set_status(&self, status: impl Into<String>, message: impl Into<String>, icon: impl Into<String>) {
        self.endpoint
            .send_notification(
                lsp_ext::Status::METHOD,
                lsp_ext::StatusParams {
                    status: status.into(),
                    message: Some(message.into()),
                    icon: Some(icon.into()),
                },
            )
            .unwrap_or(());
    }
}

impl LanguageServerHandling for MinecraftShaderLanguageServer {
    fn initialize(&mut self, params: InitializeParams, completable: MethodCompletable<InitializeResult, InitializeError>) {
        logging::slog_with_trace_id(|| {
            info!("starting server...");

            let capabilities = ServerCapabilities {
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                document_link_provider: Some(DocumentLinkOptions {
                    resolve_provider: None,
                    work_done_progress_options: WorkDoneProgressOptions { work_done_progress: None },
                }),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["graphDot".into()],
                    work_done_progress_options: WorkDoneProgressOptions { work_done_progress: None },
                }),
                text_document_sync: Some(TextDocumentSyncCapability::Options(TextDocumentSyncOptions {
                    open_close: Some(true),
                    will_save: None,
                    will_save_wait_until: None,
                    change: Some(TextDocumentSyncKind::FULL),
                    save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions { include_text: Some(true) })),
                })),
                ..ServerCapabilities::default()
            };

            let root = match params.root_uri {
                Some(uri) => PathBuf::from_url(uri),
                None => {
                    completable.complete(Err(MethodError {
                        code: 42069,
                        message: "Must be in workspace".into(),
                        data: InitializeError { retry: false },
                    }));
                    return;
                }
            };

            completable.complete(Ok(InitializeResult {
                capabilities,
                server_info: None,
            }));

            self.set_status("loading", "Building dependency graph...", "$(loading~spin)");

            self.root = root;


            self.build_initial_graph();

            self.set_status("ready", "Project initialized", "$(check)");
        });
    }

    fn shutdown(&mut self, _: (), completable: LSCompletable<()>) {
        warn!("shutting down language server...");
        completable.complete(Ok(()));
    }

    fn exit(&mut self, _: ()) {
        self.endpoint.request_shutdown();
    }

    fn workspace_change_configuration(&mut self, params: DidChangeConfigurationParams) {
        logging::slog_with_trace_id(|| {
            #[derive(Deserialize)]
            struct Configuration {
                #[serde(alias = "logLevel")]
                log_level: String,
            }

            if let Some(settings) = params.settings.as_object().unwrap().get("mcglsl") {
                let config: Configuration = from_value(settings.to_owned()).unwrap();

                info!("got updated configuration"; "config" => params.settings.as_object().unwrap().get("mcglsl").unwrap().to_string());

                configuration::handle_log_level_change(config.log_level, |level| {
                    self.log_guard = None; // set to None so Drop is invoked
                    self.log_guard = Some(logging::set_logger_with_level(level));
                })
            }
        });
    }

    fn did_open_text_document(&mut self, params: DidOpenTextDocumentParams) {
        logging::slog_with_trace_id(|| {
            //info!("opened doc {}", params.text_document.uri);
            let path = PathBuf::from_url(params.text_document.uri);
            if !path.starts_with(&self.root) {
                return;
            }

            if self.graph.borrow_mut().find_node(&path) == None {
                self.add_file_and_includes_to_graph(&path);
            }
            match self.lint(&path) {
                Ok(diagnostics) => self.publish_diagnostic(diagnostics, None),
                Err(e) => error!("error linting"; "error" => format!("{:?}", e), "path" => path.to_str().unwrap()),
            }
        });
    }

    fn did_change_text_document(&mut self, _: DidChangeTextDocumentParams) {}

    fn did_close_text_document(&mut self, _: DidCloseTextDocumentParams) {}

    fn did_save_text_document(&mut self, params: DidSaveTextDocumentParams) {
        logging::slog_with_trace_id(|| {
            let path = PathBuf::from_url(params.text_document.uri);
            if !path.starts_with(&self.root) {
                return;
            }
            self.update_includes(&path);

            match self.lint(&path) {
                Ok(diagnostics) => self.publish_diagnostic(diagnostics, None),
                Err(e) => error!("error linting"; "error" => format!("{:?}", e), "path" => path.to_str().unwrap()),
            }
        });
    }

    fn did_change_watched_files(&mut self, _: DidChangeWatchedFilesParams) {}

    fn completion(&mut self, _: TextDocumentPositionParams, completable: LSCompletable<CompletionList>) {
        completable.complete(Err(Self::error_not_available(())));
    }

    fn resolve_completion_item(&mut self, _: CompletionItem, completable: LSCompletable<CompletionItem>) {
        completable.complete(Err(Self::error_not_available(())));
    }

    fn hover(&mut self, _: TextDocumentPositionParams, _: LSCompletable<Hover>) {
        /* completable.complete(Ok(Hover{
            contents: HoverContents::Markup(MarkupContent{
                kind: MarkupKind::Markdown,
                value: String::from("# Hello World"),
            }),
            range: None,
        })); */
    }

    fn execute_command(&mut self, params: ExecuteCommandParams, completable: LSCompletable<Option<Value>>) {
        logging::slog_with_trace_id(|| {
            match self
                .command_provider
                .as_ref()
                .unwrap()
                .execute(&params.command, &params.arguments, &self.root)
            {
                Ok(resp) => {
                    info!("executed command successfully"; "command" => params.command.clone());
                    self.endpoint
                        .send_notification(
                            ShowMessage::METHOD,
                            ShowMessageParams {
                                typ: MessageType::INFO,
                                message: format!("Command {} executed successfully.", params.command),
                            },
                        )
                        .expect("failed to send popup/show message notification");
                    completable.complete(Ok(Some(resp)))
                }
                Err(err) => {
                    error!("failed to execute command"; "command" => params.command.clone(), "error" => format!("{:?}", err));
                    self.endpoint
                        .send_notification(
                            ShowMessage::METHOD,
                            ShowMessageParams {
                                typ: MessageType::ERROR,
                                message: format!("Failed to execute `{}`. Reason: {}", params.command, err),
                            },
                        )
                        .expect("failed to send popup/show message notification");
                    completable.complete(Err(MethodError::new(32420, err.to_string(), ())))
                }
            }
        });
    }

    fn signature_help(&mut self, _: TextDocumentPositionParams, completable: LSCompletable<SignatureHelp>) {
        completable.complete(Err(Self::error_not_available(())));
    }

    fn goto_definition(&mut self, params: TextDocumentPositionParams, completable: LSCompletable<Vec<Location>>) {
        logging::slog_with_trace_id(|| {
            let path = PathBuf::from_url(params.text_document.uri);
            if !path.starts_with(&self.root) {
                return;
            }
            let parser = &mut self.tree_sitter.borrow_mut();
            let parser_ctx = match navigation::ParserContext::new(parser, &path) {
                Ok(ctx) => ctx,
                Err(e) => {
                    return completable.complete(Err(MethodError {
                        code: 42069,
                        message: format!("error building parser context: error={}, path={:?}", e, path),
                        data: (),
                    }))
                }
            };

            match parser_ctx.find_definitions(&path, params.position) {
                Ok(locations) => completable.complete(Ok(locations.unwrap_or_default())),
                Err(e) => completable.complete(Err(MethodError {
                    code: 42069,
                    message: format!("error finding definitions: error={}, path={:?}", e, path),
                    data: (),
                })),
            }
        });
    }

    fn references(&mut self, params: ReferenceParams, completable: LSCompletable<Vec<Location>>) {
        logging::slog_with_trace_id(|| {
            let path = PathBuf::from_url(params.text_document_position.text_document.uri);
            if !path.starts_with(&self.root) {
                return;
            }
            let parser = &mut self.tree_sitter.borrow_mut();
            let parser_ctx = match navigation::ParserContext::new(parser, &path) {
                Ok(ctx) => ctx,
                Err(e) => {
                    return completable.complete(Err(MethodError {
                        code: 42069,
                        message: format!("error building parser context: error={}, path={:?}", e, path),
                        data: (),
                    }))
                }
            };

            match parser_ctx.find_references(&path, params.text_document_position.position) {
                Ok(locations) => completable.complete(Ok(locations.unwrap_or_default())),
                Err(e) => completable.complete(Err(MethodError {
                    code: 42069,
                    message: format!("error finding definitions: error={}, path={:?}", e, path),
                    data: (),
                })),
            }
        });
    }

    fn document_highlight(&mut self, _: TextDocumentPositionParams, completable: LSCompletable<Vec<DocumentHighlight>>) {
        completable.complete(Err(Self::error_not_available(())));
    }

    fn document_symbols(&mut self, params: DocumentSymbolParams, completable: LSCompletable<DocumentSymbolResponse>) {
        logging::slog_with_trace_id(|| {
            let path = PathBuf::from_url(params.text_document.uri);
            if !path.starts_with(&self.root) {
                return;
            }
            let parser = &mut self.tree_sitter.borrow_mut();
            let parser_ctx = match navigation::ParserContext::new(parser, &path) {
                Ok(ctx) => ctx,
                Err(e) => {
                    return completable.complete(Err(MethodError {
                        code: 42069,
                        message: format!("error building parser context: error={}, path={:?}", e, path),
                        data: (),
                    }))
                }
            };

            match parser_ctx.list_symbols(&path) {
                Ok(symbols) => completable.complete(Ok(DocumentSymbolResponse::from(symbols.unwrap_or_default()))),
                Err(e) => {
                    return completable.complete(Err(MethodError {
                        code: 42069,
                        message: format!("error finding definitions: error={}, path={:?}", e, path),
                        data: (),
                    }))
                }
            }
        });
    }

    fn workspace_symbols(&mut self, _: WorkspaceSymbolParams, completable: LSCompletable<DocumentSymbolResponse>) {
        completable.complete(Err(Self::error_not_available(())));
    }

    fn code_action(&mut self, _: CodeActionParams, completable: LSCompletable<Vec<Command>>) {
        completable.complete(Err(Self::error_not_available(())));
    }

    fn code_lens(&mut self, _: CodeLensParams, completable: LSCompletable<Vec<CodeLens>>) {
        completable.complete(Err(Self::error_not_available(())));
    }

    fn code_lens_resolve(&mut self, _: CodeLens, completable: LSCompletable<CodeLens>) {
        completable.complete(Err(Self::error_not_available(())));
    }

    fn document_link(&mut self, params: DocumentLinkParams, completable: LSCompletable<Vec<DocumentLink>>) {
        logging::slog_with_trace_id(|| {
            // node for current document
            let curr_doc = PathBuf::from_url(params.text_document.uri);
            let node = match self.graph.borrow_mut().find_node(&curr_doc) {
                Some(n) => n,
                None => {
                    warn!("document not found in graph"; "path" => curr_doc.to_str().unwrap());
                    completable.complete(Ok(vec![]));
                    return;
                }
            };

            let edges: Vec<DocumentLink> = self
                .graph
                .borrow()
                .child_node_indexes(node)
                .filter_map::<Vec<DocumentLink>, _>(|child| {
                    let graph = self.graph.borrow();
                    graph.get_child_positions(node, child).map(|value| {
                        let path = graph.get_node(child);
                        let url = match Url::from_file_path(&path) {
                            Ok(url) => url,
                            Err(e) => {
                                error!("error converting into url"; "path" => path.to_str().unwrap(), "error" => format!("{:?}", e));
                                return None;
                            }
                        };
    
                        Some(DocumentLink {
                            range: Range::new(
                                Position::new(u32::try_from(value.line).unwrap(), u32::try_from(value.start).unwrap()),
                                Position::new(u32::try_from(value.line).unwrap(), u32::try_from(value.end).unwrap()),
                            ),
                            target: Some(url.clone()),
                            tooltip: Some(url.path().to_string()),
                            data: None,
                        })
                    }).collect()
                })
                .flatten()
                .collect();
            debug!("document link results";
                "links" => format!("{:?}", edges.iter().map(|e| (e.range, e.target.as_ref().unwrap().path())).collect::<Vec<_>>()),
                "path" => curr_doc.to_str().unwrap(),
            );
            completable.complete(Ok(edges));
        });
    }

    fn document_link_resolve(&mut self, _: DocumentLink, completable: LSCompletable<DocumentLink>) {
        completable.complete(Err(Self::error_not_available(())));
    }

    fn formatting(&mut self, _: DocumentFormattingParams, completable: LSCompletable<Vec<TextEdit>>) {
        completable.complete(Err(Self::error_not_available(())));
    }

    fn range_formatting(&mut self, _: DocumentRangeFormattingParams, completable: LSCompletable<Vec<TextEdit>>) {
        completable.complete(Err(Self::error_not_available(())));
    }

    fn on_type_formatting(&mut self, _: DocumentOnTypeFormattingParams, completable: LSCompletable<Vec<TextEdit>>) {
        completable.complete(Err(Self::error_not_available(())));
    }

    fn rename(&mut self, _: RenameParams, completable: LSCompletable<WorkspaceEdit>) {
        completable.complete(Err(Self::error_not_available(())));
    }
}
