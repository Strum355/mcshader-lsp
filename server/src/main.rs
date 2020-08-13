#![feature(write_all_vectored)]
#![feature(iter_map_while)]

use rust_lsp::jsonrpc::{*, method_types::*};
use rust_lsp::lsp::*;
use rust_lsp::lsp_types::{*, notification::*};

use petgraph::stable_graph::NodeIndex;

use walkdir::WalkDir;

use std::cell::RefCell;
use std::collections::{HashMap, LinkedList};
use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter, Debug};
use std::cmp::max;
use std::io::{stdin, stdout, BufRead, BufReader, Write};
use std::ops::Add;
use std::process;
use std::rc::Rc;
use std::fs;
use std::iter::Extend;

use anyhow::{Result, anyhow};

use chan::WaitGroup;

use percent_encoding::percent_decode_str;

use regex::Regex;

use lazy_static::lazy_static;

mod graph;
mod provider;
mod lsp_ext;
mod dfs;
mod merge_views;
mod consts;

#[cfg(test)]
mod test;

lazy_static! {
    static ref RE_DIAGNOSTIC: Regex = Regex::new(r#"^(ERROR|WARNING): ([^?<>*|"]+?):(\d+): (?:'.*?' : )?(.+)\r?"#).unwrap();
    static ref RE_VERSION: Regex = Regex::new(r#"#version [\d]{3}"#).unwrap();
    static ref RE_INCLUDE: Regex = Regex::new(r#"^(?:\s)*?(?:#include) "(.+)"\r?"#).unwrap();
    static ref RE_INCLUDE_EXTENSION: Regex = Regex::new(r#"#extension GL_GOOGLE_include_directive ?: ?require"#).unwrap();
}


fn main() {
    let stdin = stdin();

    let endpoint_output = LSPEndpoint::create_lsp_output_with_output_stream(stdout);

    let cache_graph = graph::CachedStableGraph::new();

    let mut langserver = MinecraftShaderLanguageServer {
        endpoint: endpoint_output.clone(),
        graph: Rc::new(RefCell::new(cache_graph)),
        config: Configuration::default(),
        wait: WaitGroup::new(),
        root: "".to_string(),
        command_provider: None,
    };

    langserver.command_provider = Some(provider::CustomCommandProvider::new(vec![(
        "graphDot",
        Box::new(provider::GraphDotCommand {
            graph: Rc::clone(&langserver.graph),
        }),
    )]));

    LSPEndpoint::run_server_from_input(&mut stdin.lock(), endpoint_output, langserver);
}

struct MinecraftShaderLanguageServer {
    endpoint: Endpoint,
    graph: Rc<RefCell<graph::CachedStableGraph>>,
    config: Configuration,
    wait: WaitGroup,
    root: String,
    command_provider: Option<provider::CustomCommandProvider>,
}

struct Configuration {
    glslang_validator_path: String,
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration{
            glslang_validator_path: "glslangValidator".into(),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct IncludePosition {
    line: usize,
    start: usize,
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


impl MinecraftShaderLanguageServer {
    pub fn error_not_available<DATA>(data: DATA) -> MethodError<DATA> {
        let msg = "Functionality not implemented.".to_string();
        MethodError::<DATA> {
            code: 1,
            message: msg,
            data,
        }
    }

    pub fn gen_initial_graph(&self) {
        eprintln!("root of project is {}", self.root);

        // filter directories and files not ending in any of the 3 extensions
        let file_iter = WalkDir::new(&self.root)
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
                    None => {
                        eprintln!("filepath {} had no extension", path.to_str().unwrap());
                        return None;
                    }
                };
                if ext != "vsh" && ext != "fsh" && ext != "glsl" && ext != "inc" {
                    return None;
                }
                Some(String::from(path.to_str().unwrap()))
            });

        // iterate all valid found files, search for includes, add a node into the graph for each
        // file and add a file->includes KV into the map
        for path in file_iter {
            let includes = self.find_includes(path.as_str());

            let idx = self.graph.borrow_mut().add_node(&path);

            //eprintln!("adding {} with\n{:?}", path.clone(), includes);
            for include in includes {
                let child = self.graph.borrow_mut().add_node(&include.0);
                self.graph.borrow_mut().add_edge(
                    idx, 
                    child,
                    include.1,
                );
            }
        }

        eprintln!("finished building project include graph");
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    pub fn find_includes(&self, file: &str) -> Vec<(String, IncludePosition)> {
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
                let cap = RE_INCLUDE
                    .captures(line.1.as_str())
                    .unwrap()
                    .get(1)
                    .unwrap();
                //eprintln!("{:?}", caps);

                let start = cap.start();
                let end = cap.end();
                let mut path: String = cap.as_str().into();
                if !path.starts_with('/') {
                    path.insert(0, '/');
                }
                let full_include = String::from(&self.root).add("/shaders").add(path.as_str());
                includes.push((
                    full_include,
                    IncludePosition {
                        line: line.0,
                        start,
                        end,
                    }
                ));
                //eprintln!("{} includes {}", file, full_include);
            });

        includes
    }

    //fn update_includes_for_node(&self, )

    pub fn lint(&self, uri: &str) -> Result<HashMap<Url, Vec<Diagnostic>>> {
        // get all top level ancestors of this file
        let file_ancestors = match self.get_file_toplevel_ancestors(uri) {
            Ok(opt) => match opt {
                Some(ancestors) => ancestors,
                None => vec![],
            },
            Err(e) => return Err(e),
        };
        
        eprintln!("ancestors for {}:\n{:?}", uri, file_ancestors.iter().map(|e| self.graph.borrow().graph.node_weight(*e).unwrap().clone()).collect::<Vec<String>>());
        
        let mut all_sources: HashMap<String, String> = HashMap::new();
        let mut diagnostics: HashMap<Url, Vec<Diagnostic>> = HashMap::new();//Vec::new();

        if file_ancestors.is_empty() {
            let root = self.graph.borrow_mut().find_node(uri).unwrap();

            let tree = match self.get_dfs_for_node(root) {
                Ok(tree) => tree,
                Err(e) => {
                    diagnostics.insert(Url::from_file_path(uri).unwrap(), vec![e.into()]);
                    return Ok(diagnostics);
                }
            };

            all_sources.extend( self.load_sources(&tree)?);

            let graph = self.graph.borrow();
            let mut merger = merge_views::MergeViewGenerator::new(&mut all_sources, &graph);
        
            let views = merger.generate_merge_list(&tree);

            let stdout = self.invoke_validator(views)?;
            eprintln!("glslangValidator output: {}\n", stdout);
            diagnostics.extend(self.parse_validator_stdout(&stdout, ""));
        } else {
            let mut all_trees = Vec::new();

            for root in &file_ancestors {
                let nodes = match self.get_dfs_for_node(*root) {
                    Ok(nodes) => nodes,
                    Err(e) => {
                        diagnostics.insert(Url::from_file_path(uri).unwrap(), vec![e.into()]);
                        return Ok(diagnostics);
                    }
                };
                let sources = self.load_sources(&nodes)?;
                all_trees.push(nodes);
                all_sources.extend(sources);
            }

            for tree in all_trees {
                let graph = self.graph.borrow();
                let mut merger = merge_views::MergeViewGenerator::new(&mut all_sources, &graph);
            
                let views = merger.generate_merge_list(&tree);

                let stdout = self.invoke_validator(views)?;
                eprintln!("glslangValidator output: {}\n", stdout);
                diagnostics.extend(self.parse_validator_stdout(&stdout, ""));
            }
        };

        for (path, _) in all_sources {
            diagnostics.entry(Url::from_file_path(path).unwrap()).or_default();
        }

        Ok(diagnostics)
    }

    fn parse_validator_stdout(&self, stdout: &str, source: &str) -> HashMap<Url, Vec<Diagnostic>> {
        let stdout_lines = stdout.split('\n');
        let mut diagnostics: HashMap<Url, Vec<Diagnostic>> = HashMap::with_capacity(stdout_lines.count());
        let stdout_lines = stdout.split('\n');
        
        for line in stdout_lines {
            let diagnostic_capture = match RE_DIAGNOSTIC.captures(line) {
                Some(d) => d,
                None => continue
            };

            eprintln!("match {:?}", diagnostic_capture);
            
            let msg = diagnostic_capture.get(4).unwrap().as_str().trim();

            if msg.starts_with("compilation terminated") {
                continue;
            }

            let line = max(match diagnostic_capture.get(3) {
                Some(c) => match c.as_str().parse::<u64>() {
                    Ok(i) => i,
                    Err(_) => 0,
                },
                None => 0,
            }, 2) - 2;

            // TODO: line matching maybe
            /* let line_text = source_lines[line as usize];
            let leading_whitespace = line_text.len() - line_text.trim_start().len(); */

            let severity = match diagnostic_capture.get(1) {
                Some(c) => match c.as_str() {
                    "ERROR" => DiagnosticSeverity::Error,
                    "WARNING" => DiagnosticSeverity::Warning,
                    _ => DiagnosticSeverity::Information,
                }
                _ => DiagnosticSeverity::Information,
            };

            let origin = match diagnostic_capture.get(2) {
                Some(o) => o.as_str().to_string(),
                None => "".to_string(),
            };

            let diagnostic = Diagnostic {
                range: Range::new(
                    /* Position::new(line, leading_whitespace as u64),
                    Position::new(line, line_text.len() as u64) */
                    Position::new(line, 0),
                    Position::new(line, 1000)
                ),
                code: None,
                severity: Some(severity),
                source: Some(consts::SOURCE.into()),
                message: msg.into(),
                related_information: None,
                tags: None,
            };

            let origin_url = Url::from_file_path(origin).unwrap();
            match diagnostics.get_mut(&origin_url) {
                Some(d) => d.push(diagnostic),
                None => {
                    diagnostics.insert(origin_url, vec![diagnostic]);
                },
            };
        }
        diagnostics
    }

    pub fn get_dfs_for_node(&self, root: NodeIndex) -> Result<Vec<(NodeIndex, Option<NodeIndex>)>, dfs::error::CycleError> {
        let graph_ref = self.graph.borrow();

        let dfs = dfs::Dfs::new(&graph_ref, root);

        dfs.collect::<Result<Vec<_>, _>>()
    }

    pub fn load_sources(&self, nodes: &[(NodeIndex, Option<NodeIndex>)]) -> Result<HashMap<String, String>> {
        let mut sources = HashMap::new();

        for node in nodes {
            let graph = self.graph.borrow();
            let path = graph.get_node(node.0);

            if sources.contains_key(path) {
                continue;
            }

            let source = fs::read_to_string(path)?;
            sources.insert(path.clone(), source);
        }

        Ok(sources)
    }

    fn get_file_toplevel_ancestors(&self, uri: &str) -> Result<Option<Vec<petgraph::stable_graph::NodeIndex>>> {
        let curr_node = match self.graph.borrow_mut().find_node(uri) {
            Some(n) => n,
            None => return Err(anyhow!("node not found")),
        };
        let roots = self.graph.borrow().collect_root_ancestors(curr_node);
        if roots.is_empty() {
            return Ok(None);
        }
        Ok(Some(roots))
    }

    fn invoke_validator(&self, source: LinkedList<&str>) -> Result<String> {
        let cmd = process::Command::new(&self.config.glslang_validator_path)
            .args(&["--stdin", "-S", "frag"])
            .stdin(process::Stdio::piped())
            .stdout(process::Stdio::piped())
            .spawn();

        let mut child = cmd?;//.expect("glslangValidator failed to spawn");
        let stdin = child.stdin.as_mut().expect("no stdin handle found");

        eprintln!("MERGED FILE:");
        for s in &source {
            eprint!("{}", s);
        }
        
        let mut io_slices: Vec<std::io::IoSlice> = source.iter().map(|s| std::io::IoSlice::new(s.as_bytes())).collect();

        stdin.write_all_vectored(&mut io_slices[..])?;//.expect("failed to write to stdin");
        stdin.flush()?;
        let output = child.wait_with_output()?;//.expect("expected output");
        let stdout = String::from_utf8(output.stdout).unwrap();
        Ok(stdout.trim().to_string())
    }

    pub fn publish_diagnostic(&self, diagnostics: HashMap<Url, Vec<Diagnostic>>, document_version: Option<i64>) {
        eprintln!("DIAGNOSTICS:\n{:?}", diagnostics);
        for (uri, diagnostics) in diagnostics {
            self.endpoint.send_notification(PublishDiagnostics::METHOD, PublishDiagnosticsParams {
                uri,
                diagnostics,
                version: document_version,
            }).expect("failed to publish diagnostics");
        }
    }

    fn set_status(&self, status: impl Into<String>, message: impl Into<String>, icon: impl Into<String>) {
        self.endpoint.send_notification(lsp_ext::Status::METHOD, lsp_ext::StatusParams {
            status: status.into(),
            message: Some(message.into()),
            icon: Some(icon.into()),
        }).unwrap_or(());
    }
}

impl LanguageServerHandling for MinecraftShaderLanguageServer {
    fn initialize(&mut self, params: InitializeParams, completable: MethodCompletable<InitializeResult, InitializeError>) {
        self.wait.add(1);

        let mut capabilities = ServerCapabilities::default();
        capabilities.hover_provider = None;
        capabilities.document_link_provider = Some(DocumentLinkOptions {
            resolve_provider: None,
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: None,
            },
        });
        capabilities.execute_command_provider = Some(ExecuteCommandOptions {
            commands: vec!["graphDot".into()],
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: None,
            },
        });
        capabilities.text_document_sync = Some(TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                open_close: Some(true),
                will_save: None,
                will_save_wait_until: None,
                change: Some(TextDocumentSyncKind::Full),
                save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                    include_text: Some(true),
                }))
            },
        ));

        let root_path: String = match params.root_uri {
            Some(uri) => uri.path().into(),
            None => {
                completable.complete(Err(MethodError {
                    code: 42069,
                    message: "Must be in workspace".into(),
                    data: InitializeError {
                        retry: false,
                    },
                }));
                return;
            }
        };

        completable.complete(Ok(InitializeResult {
            capabilities,
            server_info: None,
        }));

        self.set_status("loading", "Building dependency graph...", "$(loading~spin)");

        let root: String = match percent_decode_str(root_path.as_ref()).decode_utf8() {
            Ok(s) => s.into(),
            Err(e) => {
                self.set_status("failed", format!("{}", e), "$(close)");
                return
            },
        };

        self.root = root;

        self.gen_initial_graph();

        self.set_status("ready", "Project initialized", "$(check)");
    }

    fn shutdown(&mut self, _: (), completable: LSCompletable<()>) {
        eprintln!("shutting down language server...");
        completable.complete(Ok(()));
    }

    fn exit(&mut self, _: ()) {
        self.endpoint.request_shutdown();
    }

    fn workspace_change_configuration(&mut self, params: DidChangeConfigurationParams) {
        let config = params.settings.as_object().unwrap().get("mcglsl").unwrap();

        self.config.glslang_validator_path = config
                .get("glslangValidatorPath")
                .unwrap()
                .as_str()
                .unwrap()
                .into();

        eprintln!("{:?}", params.settings.as_object().unwrap());

        self.wait.done();
    }

    fn did_open_text_document(&mut self, params: DidOpenTextDocumentParams) {
        eprintln!("opened doc {}", params.text_document.uri);
        match self.lint(params.text_document.uri.path()/* , params.text_document.text */) {
            Ok(diagnostics) => self.publish_diagnostic(diagnostics, None),
            Err(e) => eprintln!("error linting: {}", e),
        }
    }

    fn did_change_text_document(&mut self, params: DidChangeTextDocumentParams) {
        self.wait.wait();

        /* #[allow(unused_variables)]
        let text_change = params.content_changes[0]; */
        //eprintln!("changed {} changes: {}", text_change., params.text_document.uri);
    }

    fn did_close_text_document(&mut self, _: DidCloseTextDocumentParams) {}

    fn did_save_text_document(&mut self, params: DidSaveTextDocumentParams) {
        eprintln!("saved doc {}", params.text_document.uri);

        let path: String = percent_encoding::percent_decode_str(params.text_document.uri.path()).decode_utf8().unwrap().into();
        
        /*let file_content = fs::read(path).unwrap(); */
        match self.lint(path.as_str()/* , String::from_utf8(file_content).unwrap() */) {
            Ok(diagnostics) => self.publish_diagnostic(diagnostics, None),
            Err(e) => eprintln!("error linting: {}", e),
        }
    }

    fn did_change_watched_files(&mut self, _: DidChangeWatchedFilesParams) {}

    fn completion(&mut self, _: TextDocumentPositionParams, completable: LSCompletable<CompletionList>) {
        completable.complete(Err(Self::error_not_available(())));
    }

    fn resolve_completion_item(&mut self, _: CompletionItem, completable: LSCompletable<CompletionItem>) {
        completable.complete(Err(Self::error_not_available(())));
    }

    fn hover(&mut self, _: TextDocumentPositionParams, _: LSCompletable<Hover>) {
        self.wait.wait();
        /* completable.complete(Ok(Hover{
            contents: HoverContents::Markup(MarkupContent{
                kind: MarkupKind::Markdown,
                value: String::from("# Hello World"),
            }),
            range: None,
        })); */
    }

    fn execute_command(&mut self, mut params: ExecuteCommandParams, completable: LSCompletable<WorkspaceEdit>) {
        params
            .arguments
            .push(serde_json::Value::String(self.root.clone()));
        match self
            .command_provider
            .as_ref()
            .unwrap()
            .execute(params.command.as_ref(), params.arguments)
        {
            Ok(_) => {
                eprintln!("executed {} successfully", params.command);
                self.endpoint.send_notification(ShowMessage::METHOD, ShowMessageParams {
                    typ: MessageType::Info,
                    message: format!("Command {} executed successfully.", params.command),
                }).expect("failed to send popup/show message notification");
                completable.complete(Ok(WorkspaceEdit {
                    changes: None,
                    document_changes: None,
                }))
            },
            Err(err) => {
                self.endpoint.send_notification(ShowMessage::METHOD, ShowMessageParams {
                    typ: MessageType::Error,
                    message: format!("Failed to execute command '{}'", params.command),
                }).expect("failed to send popup/show message notification");
                eprintln!("failed to execute {}: {}", params.command, err);
                completable.complete(Err(MethodError::new(32420, err, ())))
            },
        }
    }

    fn signature_help(&mut self, _: TextDocumentPositionParams, completable: LSCompletable<SignatureHelp>) {
        completable.complete(Err(Self::error_not_available(())));
    }

    fn goto_definition(&mut self, _: TextDocumentPositionParams, completable: LSCompletable<Vec<Location>>) {
        completable.complete(Err(Self::error_not_available(())));
    }

    fn references(&mut self, _: ReferenceParams, completable: LSCompletable<Vec<Location>>) {
        completable.complete(Err(Self::error_not_available(())));
    }

    fn document_highlight(&mut self, _: TextDocumentPositionParams, completable: LSCompletable<Vec<DocumentHighlight>>) {
        completable.complete(Err(Self::error_not_available(())));
    }

    fn document_symbols(&mut self, _: DocumentSymbolParams, completable: LSCompletable<Vec<SymbolInformation>>) {
        completable.complete(Err(Self::error_not_available(())));
    }

    fn workspace_symbols(&mut self, _: WorkspaceSymbolParams, completable: LSCompletable<Vec<SymbolInformation>>) {
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
        eprintln!("document link file: {:?}", params.text_document.uri.to_file_path().unwrap());
        // node for current document
        let curr_doc = params
            .text_document
            .uri
            .to_file_path()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string();
        let node = match self.graph.borrow_mut().find_node(&curr_doc) {
            Some(n) => n,
            None => {
                completable.complete(Ok(vec![]));
                return
            },
        };

        let edges: Vec<DocumentLink> = self
            .graph
            .borrow()
            .child_node_indexes(node)
            .into_iter()
            .filter_map(|child| {
                let graph = self.graph.borrow();
                let value = graph.get_edge_meta(node, child);
                let path: std::ffi::OsString = graph.get_node(child).try_into().unwrap();
                let path = std::path::Path::new(&path);
                let url = match Url::from_file_path(path) {
                    Ok(url) => url,
                    Err(e) => {
                        eprintln!("error converting {:?} into url: {:?}", path, e);
                        return None;
                    }
                };

                Some(DocumentLink {
                    range: Range::new(
                        Position::new(
                            u64::try_from(value.line).unwrap(),
                            u64::try_from(value.start).unwrap()),
                        Position::new(
                            u64::try_from(value.line).unwrap(),
                            u64::try_from(value.end).unwrap()),
                    ),
                    target: Some(url),
                    //tooltip: Some(url.path().to_string().strip_prefix(self.root.clone().unwrap().as_str()).unwrap().to_string()),
                    tooltip: None,
                    data: None,
                })
            })
            .collect();
        eprintln!("links: {:?}", edges);
        completable.complete(Ok(edges));
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
