use std::{collections::HashMap, ffi::OsStr, marker::Sync, path::Path, sync::Arc, str::FromStr};

use filesystem::NormalizedPathBuf;
use futures::future::join_all;
use logging::{error, info, logger, trace, warn, FutureExt};
use serde::Deserialize;
use serde_json::{Value, from_value};

use tokio::sync::Mutex;

#[cfg(test)]
use test::Client;
#[cfg(not(test))]
use tower_lsp::Client;

use glob::{glob_with, MatchOptions};
use tower_lsp::{
    jsonrpc::{Error, ErrorCode},
    lsp_types::{
        notification::{ShowMessage, TelemetryEvent},
        *,
    },
    LanguageServer,
};

use tst::TSTMap;

use crate::{commands, workspace::Workspace};

pub struct WorkspaceIndex(usize);

pub struct Server<G: 'static, F: 'static>
where
    G: opengl::ShaderValidator + Send,
    F: Fn() -> G,
{
    pub client: Arc<Mutex<Client>>,
    workspaces: Arc<Mutex<TSTMap<Arc<Workspace<G>>>>>,
    gl_factory: F,
    _log_guard: logging::GlobalLoggerGuard
}

impl<G, F> Server<G, F>
where
    G: opengl::ShaderValidator + Send,
    F: Fn() -> G,
{
    pub fn new(client: Client, gl_factory: F) -> Self {
        Server {
            client: Arc::new(Mutex::new(client)),
            workspaces: Default::default(),
            gl_factory,
            _log_guard: logging::init_logger()
        }
    }
}

#[tower_lsp::async_trait]
impl<G, F> LanguageServer for Server<G, F>
where
    G: opengl::ShaderValidator + Send,
    F: Fn() -> G + Send + Sync,
{
    #[logging::with_trace_id]
    async fn initialize(&self, params: InitializeParams) -> tower_lsp::jsonrpc::Result<InitializeResult> {
        info!("starting server...");

        let capabilities = Server::<G, F>::capabilities();

        let root: NormalizedPathBuf = match params.root_uri {
            Some(uri) => uri.into(),
            None => {
                return Err(Error {
                    code: ErrorCode::InvalidParams,
                    message: "Must be in workspace".into(),
                    data: Some(serde_json::to_value(InitializeError { retry: false }).unwrap()),
                });
            }
        };

        self.client
            .lock()
            .with_logger(logger())
            .await
            .send_notification::<TelemetryEvent>(serde_json::json!({
                "status": "loading",
                "message": "Building dependency graph...",
                "icon": "$(loading~spin)",
            }))
            .with_logger(logger())
            .await;

        self.gather_workspaces(&root).with_logger(logger()).await;

        self.client
            .lock()
            .with_logger(logger())
            .await
            .send_notification::<TelemetryEvent>(serde_json::json!({
                "status": "ready",
                "message": "Project(s) initialized...",
                "icon": "$(check)",
            }))
            .with_logger(logger())
            .await;

        Ok(InitializeResult {
            capabilities,
            server_info: None,
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        // self.client
        //     .lock()
        //     .with_logger(logger())
        //     .await
        //     .log_message(MessageType::INFO, "command executed!")
        //     .with_logger(logger())
        //     .await;
    }

    async fn shutdown(&self) -> tower_lsp::jsonrpc::Result<()> {
        warn!("shutting down language server...");
        Ok(())
    }

    #[logging::with_trace_id]
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        info!("opened document"; "uri" => params.text_document.uri.as_str());

        let path: NormalizedPathBuf = params.text_document.uri.into();

        if let Some(workspace) = self.workspace_for_file(&path).await {
            trace!("found workspace"; "root" => &workspace.root);

            workspace
                .update_sourcefile(&path, params.text_document.text)
                .with_logger(logger())
                .await;

            match workspace.lint(&path).with_logger(logger()).await {
                Ok(diagnostics) => self.publish_diagnostic(diagnostics, None).with_logger(logger()).await,
                Err(e) => error!("error linting"; "error" => format!("{:?}", e), "path" => &path),
            }
        }
    }

    #[logging::with_trace_id]
    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        info!("saved document"; "uri" => params.text_document.uri.as_str());

        let path: NormalizedPathBuf = params.text_document.uri.into();
        match self.workspace_for_file(&path).await {
            Some(workspace) => {
                trace!("found workspace"; "root" => &workspace.root);

                workspace.update_sourcefile(&path, params.text.unwrap()).with_logger(logger()).await;

                match workspace.lint(&path).with_logger(logger()).await {
                    Ok(diagnostics) => self.publish_diagnostic(diagnostics, None).with_logger(logger()).await,
                    Err(e) => error!("error linting"; "error" => format!("{:?}", e), "path" => &path),
                }
            }
            None => warn!("no workspace found"; "path" => path),
        }
    }

    #[logging::with_trace_id]
    async fn execute_command(&self, params: ExecuteCommandParams) -> tower_lsp::jsonrpc::Result<Option<Value>> {
        match params.command.as_str() {
            // "graphDot" => {
            //     let document_path: NormalizedPathBuf = params.arguments.first().unwrap().try_into().unwrap();
            //     let manager = self.workspace_manager.lock().with_logger(logger()).await;
            //     let workspace = manager.find_workspace_for_file(&document_path).unwrap();
            //     let graph = workspace.graph.lock().with_logger(logger()).await;
            //     commands::graph_dot::run(&workspace.root, &graph).map_err(|e| Error {
            //         code: ErrorCode::InternalError,
            //         message: format!("{:?}", e),
            //         data: None,
            //     })
            // }
            "virtualMerge" => {
                let document_path: NormalizedPathBuf = params.arguments.first().unwrap().try_into().unwrap();
                let workspace = self.workspace_for_file(&document_path).await.unwrap();
                let mut workspace_view = workspace.workspace_view.lock().with_logger(logger()).await;

                let mut roots = workspace_view.trees_for_entry(&document_path).unwrap();
                let root = roots.next().unwrap();
                if roots.next().is_some() {
                    return Err(Error {
                        code: ErrorCode::InternalError,
                        message: "unexpected >1 root".into(),
                        data: None,
                    });
                }

                let sources = root
                    .unwrap()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .filter(|res| !matches!(res, Err(workspace::TreeError::FileNotFound { .. })))
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();

                commands::merged_includes::run(&document_path, &sources)
                    .with_logger(logger())
                    .await
                    .map_err(|e| Error {
                        code: ErrorCode::InternalError,
                        message: format!("{:?}", e),
                        data: None,
                    })
            }
            // "parseTree",
            _ => Err(Error {
                code: ErrorCode::InternalError,
                message: "command doesn't exist".into(),
                data: None,
            }),
        }
        .inspect_err(|e| {
            futures::executor::block_on(async {
                self.client
                    .lock()
                    .with_logger(logger())
                    .await
                    .send_notification::<ShowMessage>(ShowMessageParams {
                        typ: MessageType::ERROR,
                        message: format!("Failed to execute `{}`: {}.", params.command, e),
                    })
                    .with_logger(logger())
                    .await;
            });
        })
        .inspect(|_| {
            futures::executor::block_on(async {
                self.client
                    .lock()
                    .with_logger(logger())
                    .await
                    .send_notification::<ShowMessage>(ShowMessageParams {
                        typ: MessageType::INFO,
                        message: format!("Command `{}` executed successfully.", params.command),
                    })
                    .with_logger(logger())
                    .await;
            });
        })
    }

    async fn goto_definition(&self, _params: GotoDefinitionParams) -> tower_lsp::jsonrpc::Result<Option<GotoDefinitionResponse>> {
        /* logging::slog_with_trace_id(|| {
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
        } */
        Ok(None)
    }

    async fn references(&self, _params: ReferenceParams) -> tower_lsp::jsonrpc::Result<Option<Vec<Location>>> {
        /* logging::slog_with_trace_id(|| {
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
        }); */
        Ok(None)
    }

    async fn document_symbol(&self, _params: DocumentSymbolParams) -> tower_lsp::jsonrpc::Result<Option<DocumentSymbolResponse>> {
        /* logging::slog_with_trace_id(|| {
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

            match parser_ctx.list_document_symbols(&path) {
                Ok(symbols) => completable.complete(Ok(DocumentSymbolResponse::from(symbols.unwrap_or_default()))),
                Err(e) => {
                    return completable.complete(Err(MethodError {
                        code: 42069,
                        message: format!("error finding definitions: error={}, path={:?}", e, path),
                        data: (),
                    }))
                }
            }
        }); */
        Ok(None)
    }

    async fn document_link(&self, _params: DocumentLinkParams) -> tower_lsp::jsonrpc::Result<Option<Vec<DocumentLink>>> {
        /* logging::slog_with_trace_id(|| {
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
        }); */
        Ok(None)
    }

    #[logging_macro::with_trace_id]
    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        #[derive(Deserialize)]
        struct Configuration {
            #[serde(alias = "logLevel")]
            log_level: String,
        }

        let config: Configuration = from_value(params.settings.as_object().unwrap().get("mcglsl").unwrap().to_owned()).unwrap();

        info!("got updated configuration"; "config" => params.settings.as_object().unwrap().get("mcglsl").unwrap().to_string());

        match logging::Level::from_str(config.log_level.as_str()) {
            Ok(level) => logging::set_level(level),
            Err(_) => error!("got unexpected log level from config"; "level" => &config.log_level),
        }
    }
}

impl<G, F> Server<G, F>
where
    G: opengl::ShaderValidator + Send,
    F: Fn() -> G,
{
    fn capabilities() -> ServerCapabilities {
        ServerCapabilities {
            definition_provider: Some(OneOf::Left(false)),
            references_provider: Some(OneOf::Left(false)),
            document_symbol_provider: Some(OneOf::Left(false)),
            document_link_provider: /* Some(DocumentLinkOptions {
                resolve_provider: None,
                work_done_progress_options: WorkDoneProgressOptions { work_done_progress: None },
            }), */
            None,
            execute_command_provider: Some(ExecuteCommandOptions {
                commands: vec!["graphDot".into(), "virtualMerge".into(), "parseTree".into()],
                work_done_progress_options: WorkDoneProgressOptions { work_done_progress: None },
            }),
            text_document_sync: Some(TextDocumentSyncCapability::Options(TextDocumentSyncOptions {
                open_close: Some(true),
                will_save: None,
                will_save_wait_until: None,
                change: Some(TextDocumentSyncKind::FULL),
                save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions { include_text: Some(true) })),
            })),
            workspace: Some(WorkspaceServerCapabilities {
                workspace_folders: Some(WorkspaceFoldersServerCapabilities{
                    supported: Some(true),
                    change_notifications: Some(OneOf::Left(false)),
                }),
                file_operations: None,
            }),
            semantic_tokens_provider: Some(
                SemanticTokensOptions {
                    work_done_progress_options: WorkDoneProgressOptions { work_done_progress: None },
                    legend: SemanticTokensLegend {
                        token_types: vec![SemanticTokenType::COMMENT],
                        token_modifiers: vec![],
                    },
                    range: None,
                    full: Some(SemanticTokensFullOptions::Bool(true)),
                }
                .into(),
            ),
            ..ServerCapabilities::default()
        }
    }

    pub async fn gather_workspaces(&self, root: &NormalizedPathBuf) {
        let options = MatchOptions {
            case_sensitive: true,
            ..MatchOptions::default()
        };

        let glob = root.join("**").join("shaders.properties");
        info!("banana"; "glob" => &glob);

        for entry in glob_with(&glob.to_string(), options).unwrap() {
            match entry {
                Ok(path)
                    if path.file_name().and_then(OsStr::to_str) == Some("shaders.properties")
                        && path.parent().and_then(Path::file_name).and_then(OsStr::to_str) == Some("shaders") =>
                {
                    match path.parent().and_then(Path::parent).map(Into::into) {
                        Some(shader_root) => self.add_workspace(&shader_root).with_logger(logger()).await,
                        None => todo!(),
                    }
                }
                Ok(path)
                    if path.file_name().and_then(OsStr::to_str) == Some("shaders.properties")
                        && path
                            .parent()
                            .and_then(Path::file_name)
                            .and_then(OsStr::to_str)
                            .map_or(false, |f| f.starts_with("world"))
                        && path
                            .parent()
                            .and_then(Path::parent)
                            .and_then(Path::file_name)
                            .and_then(OsStr::to_str)
                            == Some("shaders") =>
                {
                    match path.parent().and_then(Path::parent).and_then(Path::parent).map(Into::into) {
                        Some(shader_root) => self.add_workspace(&shader_root).with_logger(logger()).await,
                        None => todo!(),
                    }
                }
                Ok(path) => {
                    let path: NormalizedPathBuf = path.into();
                    error!("shaders.properties found outside ./shaders or ./worldX dir"; "path" => path)
                }
                Err(e) => error!("error iterating glob entries"; "error" => format!("{:?}", e)),
            }
        }

        let glob = root.join("**").join("shaders");
        for entry in glob_with(&glob.to_string(), options).unwrap() {
            match entry {
                Ok(path)
                    if !walkdir::WalkDir::new(path.clone()).into_iter().any(|p| {
                        p.as_ref()
                            .ok()
                            .map(|p| p.file_name())
                            .and_then(|f| f.to_str())
                            .map_or(false, |f| f == "shaders.properties")
                    }) =>
                {
                    match path.parent().map(Into::into) {
                        Some(shader_root) => self.add_workspace(&shader_root).with_logger(logger()).await,
                        None => todo!(),
                    }
                }
                Ok(path) => {
                    let path: NormalizedPathBuf = path.into();
                    info!("skipping as already existing"; "path" => path)
                }
                Err(e) => error!("error iterating glob entries"; "error" => format!("{:?}", e)),
            }
        }
    }

    async fn add_workspace(&self, root: &NormalizedPathBuf) {
        let mut search = self.workspaces.lock().with_logger(logger()).await;
        // let mut workspaces = self.workspaces.lock().with_logger(logger()).await;

        if !search.contains_key(&root.to_string()) {
            info!("adding workspace"; "root" => &root);
            let opengl_context = (self.gl_factory)();
            let workspace = Workspace::new(root.clone(), opengl_context);
            workspace.build().with_logger(logger()).await;
            // workspaces.push(workspace);
            // search.insert(&root.to_string(), WorkspaceIndex(workspaces.len() - 1));
            search.insert(&root.to_string(), Arc::new(workspace));
        }
    }

    async fn publish_diagnostic(&self, diagnostics: HashMap<Url, Vec<Diagnostic>>, document_version: Option<i32>) {
        let client = self.client.lock().with_logger(logger()).await;
        let mut handles = Vec::with_capacity(diagnostics.len());
        for (url, diags) in diagnostics {
            handles.push(client.publish_diagnostics(url, diags, document_version));
        }
        join_all(handles).with_logger(logger()).await;
    }

    pub async fn workspace_for_file(&self, file: &NormalizedPathBuf) -> Option<Arc<Workspace<G>>> {
        let search = self.workspaces.lock().with_logger(logger()).await;
        // let workspaces = self.workspaces.lock().with_logger(logger()).await;

        let file = file.to_string();
        let prefix = search.longest_prefix(&file);
        if prefix.is_empty() {
            return None;
        }

        search.get(prefix).cloned()
    }
}

#[allow(unused)]
#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;
    use std::sync::Arc;

    use filesystem::NormalizedPathBuf;
    use logging::warn;
    use opengl::MockShaderValidator;
    use pretty_assertions::assert_eq;

    use serde_json::json;
    use serde_json::Value;
    use tempdir::TempDir;

    use fs_extra::{copy_items, dir};

    use tower_lsp::lsp_types::Diagnostic;
    use tower_lsp::ClientSocket;
    use tower_lsp::LanguageServer;
    use tower_lsp::LspService;
    use tower_test::mock::Spawn;
    use url::Url;

    use crate::server::Server;

    // implements a noop client for testing sake
    pub struct Client;

    impl Client {
        pub async fn send_notification<N>(&self, _: N::Params)
        where
            N: tower_lsp::lsp_types::notification::Notification,
        {
        }

        pub async fn publish_diagnostics(&self, uri: Url, diags: Vec<Diagnostic>, version: Option<i32>) {}
    }

    pub fn new_temp_server<F>(gl_factory: F) -> Server<MockShaderValidator, F>
    where
        F: Fn() -> MockShaderValidator + Send + Sync,
    {
        Server::new(Client {}, gl_factory)
    }

    #[macro_export]
    macro_rules! assert_exchange {
        ($service:expr, $request:expr, $response:expr, $method:path) => {
            assert_eq!($method($service, $request).await, $response);
        };
    }

    fn copy_to_tmp_dir(test_path: &str) -> (TempDir, PathBuf) {
        let tmp_dir = TempDir::new("mcshader").unwrap();
        fs::create_dir(tmp_dir.path().join("shaders")).unwrap();

        {
            let test_path = Path::new(test_path)
                .canonicalize()
                .unwrap_or_else(|_| panic!("canonicalizing '{}'", test_path));
            let opts = &dir::CopyOptions::new();
            let files = fs::read_dir(test_path)
                .unwrap()
                .map(|e| String::from(e.unwrap().path().to_str().unwrap()))
                .collect::<Vec<String>>();
            copy_items(&files, tmp_dir.path().join("shaders"), opts).unwrap();
        }

        let tmp_path = tmp_dir.path().to_str().unwrap().into();

        (tmp_dir, tmp_path)
    }

    #[tokio::test]
    #[logging_macro::scope]
    async fn test_empty_initialize() {
        let mut server = new_temp_server(MockShaderValidator::new);

        let tmp_dir = TempDir::new("mcshader").unwrap();
        let tmp_path = tmp_dir.path();

        let init_req = initialize::request(tmp_path);
        let init_resp = Ok(initialize::response());
        assert_exchange!(&server, init_req, init_resp, Server::initialize);

        assert_eq!(server.workspaces.lock().await.len(), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[logging_macro::scope]
    async fn test_01_initialize() {
        let mut server = new_temp_server(MockShaderValidator::new);

        let (_tmp_dir, tmp_path) = copy_to_tmp_dir("../testdata/01");

        let init_req = initialize::request(&tmp_path);
        let init_resp = Ok(initialize::response());
        assert_exchange!(&server, init_req, init_resp, Server::initialize);

        assert_eq!(
            server
                .workspaces
                .lock()
                .await
                .iter()
                .map(|(_, w)| w.root.to_string())
                .collect::<Vec<String>>(),
            vec![tmp_path.to_str().unwrap()]
        );

        // let workspace = workspaces.first().unwrap();
        // let graph = workspace.graph.lock().await;
        // // Assert there is one edge between two nodes
        // assert_eq!(graph.inner().edge_count(), 1);

        // let edge = graph.inner().edge_indices().next().unwrap();
        // let (node1, node2) = graph.inner().edge_endpoints(edge).unwrap();

        // // Assert the values of the two nodes in the tree
        // assert_eq!(graph.inner()[node1], tmp_path.join("shaders").join("final.fsh").into());
        // assert_eq!(graph.inner()[node2], tmp_path.join("shaders").join("common.glsl").into());

        // assert_eq!(graph.inner().edge_weight(edge).unwrap().line, 2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[logging_macro::scope]
    async fn test_05_initialize() {
        let mut server = new_temp_server(MockShaderValidator::new);

        let (_tmp_dir, tmp_path) = copy_to_tmp_dir("../testdata/05");

        let init_req = initialize::request(&tmp_path);
        let init_resp = Ok(initialize::response());
        assert_exchange!(&server, init_req, init_resp, Server::initialize);

        assert_eq!(
            server
                .workspaces
                .lock()
                .await
                .iter()
                .map(|(_, w)| w.root.to_string())
                .collect::<Vec<String>>(),
            vec![tmp_path.to_str().unwrap()]
        );

        // let workspace = workspaces.first().unwrap();
        // let graph = workspace.graph.lock().await;

        // // Assert there is one edge between two nodes
        // assert_eq!(graph.inner().edge_count(), 3);

        // assert_eq!(graph.inner().node_count(), 4);

        // let pairs: HashSet<(NormalizedPathBuf, NormalizedPathBuf)> = vec![
        //     (
        //         tmp_path.join("shaders").join("final.fsh").into(),
        //         tmp_path.join("shaders").join("common.glsl").into(),
        //     ),
        //     (
        //         tmp_path.join("shaders").join("final.fsh").into(),
        //         tmp_path.join("shaders").join("test").join("banana.glsl").into(),
        //     ),
        //     (
        //         tmp_path.join("shaders").join("test").join("banana.glsl").into(),
        //         tmp_path.join("shaders").join("test").join("burger.glsl").into(),
        //     ),
        // ]
        // .into_iter()
        // .collect();

        // for edge in graph.inner().edge_indices() {
        //     let endpoints = graph.inner().edge_endpoints(edge).unwrap();
        //     let first = &graph[endpoints.0];
        //     let second = &graph[endpoints.1];
        //     let contains = pairs.contains(&(first.clone(), second.clone()));
        //     assert!(contains, "doesn't contain ({:?}, {:?})", first, second);
        // }
    }

    #[macro_export]
    macro_rules! from_request {
        ($($json:tt)+) => {
            {
                use tower_lsp::jsonrpc;
                use serde_json::{json, Value};
                serde_json::from_value::<jsonrpc::Request>(json!($($json)+))
                    .map(|msg| msg.params().unwrap().clone())
                    .map(|value| serde_json::from_value(value))
                    .unwrap()
                    .unwrap()
            }
        };
    }

    #[macro_export]
    macro_rules! from_response {
        ($($json:tt)+) => {
            {
                use tower_lsp::jsonrpc;
                use serde_json::{json, Value};
                serde_json::from_value::<jsonrpc::Response>(json!($($json)+))
                    .map(|msg| msg.result().unwrap().clone())
                    .map(|value| serde_json::from_value(value))
                    .unwrap()
                    .unwrap()
            }
        };
    }

    pub mod exit {
        use serde_json::{json, Value};

        pub fn notification() -> Value {
            json!({
                "jsonrpc": "2.0",
                "method": "exit",
            })
        }
    }

    pub mod initialize {
        use std::path::Path;

        use tower_lsp::{jsonrpc, lsp_types};
        use url::Url;

        pub fn request(root: &Path) -> lsp_types::InitializeParams {
            from_request!({
                "jsonrpc": "2.0",
                "method": "initialize",
                "params": {
                    "rootUri": Url::from_directory_path(root).unwrap(),
                    "capabilities":{},
                },
                "id": 1,
            })
        }

        pub fn response() -> lsp_types::InitializeResult {
            use crate::server::Server;
            use opengl::MockShaderValidator;

            from_response!({
                "jsonrpc": "2.0",
                "result": {
                    "capabilities": Server::<MockShaderValidator, fn() -> MockShaderValidator>::capabilities(),
                },
                "id": 1,
            })
        }
    }

    pub mod initialized {
        use tower_lsp::lsp_types;

        pub fn notification() -> lsp_types::InitializedParams {
            from_request!({
                "jsonrpc": "2.0",
                "method": "initialized",
                "params": {},
            })
        }
    }

    /* pub mod text_document {
        pub mod did_change {

            pub mod notification {
                use serde_json::{json, Value};
                use tower_lsp::lsp_types::*;

                pub fn entire<S: AsRef<str>>(uri: &Url, text: S) -> Value {
                    json!({
                        "jsonrpc": "2.0",
                        "method": "textDocument/didChange",
                        "params": {
                            "textDocument": {
                                "uri": uri,
                            },
                            "contentChanges": [
                                {
                                    "text": text.as_ref(),
                                }
                            ],
                        },
                    })
                }
            }
        }

        pub mod did_close {
            use serde_json::{json, Value};
            use tower_lsp::lsp_types::*;

            pub fn notification(uri: &Url) -> Value {
                json!({
                    "jsonrpc": "2.0",
                    "method": "textDocument/didClose",
                    "params": {
                        "textDocument": {
                            "uri": uri,
                        },
                    },
                })
            }
        }

        pub mod did_open {
            use serde_json::{json, Value};
            use tower_lsp::lsp_types::*;

            pub fn notification<S: AsRef<str>, T: AsRef<str>>(uri: &Url, language_id: S, version: i64, text: T) -> Value {
                json!({
                    "jsonrpc": "2.0",
                    "method": "textDocument/didOpen",
                    "params": {
                        "textDocument": {
                            "uri": uri,
                            "languageId": language_id.as_ref(),
                            "version": version,
                            "text": text.as_ref(),
                        },
                    },
                })
            }
        }

        pub mod document_symbol {
            use serde_json::{json, Value};
            use tower_lsp::lsp_types::*;

            pub fn request(uri: &Url) -> Value {
                json!({
                    "jsonrpc": "2.0",
                    "method": "textDocument/documentSymbol",
                    "params": {
                        "textDocument": {
                            "uri": uri,
                        },
                    },
                    "id": 1,
                })
            }

            pub fn response(response: DocumentSymbolResponse) -> Value {
                json!({
                    "jsonrpc": "2.0",
                    "result": response,
                    "id": 1,
                })
            }
        }

        pub mod hover {
            use serde_json::{json, Value};
            use tower_lsp::lsp_types::*;

            pub fn request(uri: &Url, position: Position) -> Value {
                json!({
                    "jsonrpc": "2.0",
                    "method": "textDocument/hover",
                    "params": {
                        "textDocument": {
                            "uri": uri,
                        },
                        "position": position,
                    },
                    "id": 1,
                })
            }

            pub fn response() -> Value {
                json!({
                    "jsonrpc": "2.0",
                    "result": {
                    },
                    "id": 1,
                })
            }
        }

        pub mod publish_diagnostics {
            use serde_json::{json, Value};
            use tower_lsp::lsp_types::*;

            pub fn notification(uri: &Url, diagnostics: &[Diagnostic]) -> Value {
                json!({
                    "jsonrpc": "2.0",
                    "method": "textDocument/publishDiagnostics",
                    "params": {
                        "uri": uri,
                        "diagnostics": diagnostics,
                    },
                })
            }
        }
    } */
}
