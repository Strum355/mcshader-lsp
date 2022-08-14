use std::{collections::HashMap, marker::Sync, sync::Arc};

use filesystem::NormalizedPathBuf;
// use futures::future::join_all;
use logging::{error, info, logger, trace, warn, FutureExt};
use serde_json::Value;

use tokio::sync::Mutex;

// #[cfg(test)]
// use test::Client;
// #[cfg(not(test))]
use tower_lsp::Client;

use tower_lsp::{
    jsonrpc::{Error, ErrorCode, Result},
    lsp_types::{
        notification::{ShowMessage, TelemetryEvent},
        *,
    },
    LanguageServer,
};

use workspace::WorkspaceManager;

// use crate::commands;

pub struct Server<G: 'static, F: 'static>
where
    G: opengl::ShaderValidator + Send,
    F: Fn() -> G,
{
    pub client: Arc<Mutex<Client>>,
    workspace_manager: Arc<Mutex<WorkspaceManager<G, F>>>,
}

impl<G, F> Server<G, F>
where
    G: opengl::ShaderValidator + Send,
    F: Fn() -> G,
{
    pub fn new(client: Client, gl_factory: F) -> Self {
        Server {
            client: Arc::new(Mutex::new(client)),
            workspace_manager: Arc::new(Mutex::new(WorkspaceManager::new(gl_factory))),
        }
    }

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

    async fn publish_diagnostic(&self, diagnostics: HashMap<Url, Vec<Diagnostic>>, document_version: Option<i32>) {
        let client = self.client.lock().with_logger(logger()).await;
        // let mut handles = Vec::with_capacity(diagnostics.len());
        for (url, diags) in diagnostics {
            eprintln!("publishing to {:?} {:?}", &url, diags);
            /* handles.push( */
            client.publish_diagnostics(url, diags, document_version).with_logger(logger()).await;
            client
                .log_message(MessageType::INFO, "PUBLISHING!")
                .with_logger(logger())
                .await;
            // client.send_notification::<PublishDiagnostics>(PublishDiagnosticsParams {
            //     ri: url,
            //     diagnostics: diags,
            //     // version: document_version,
            //     version: None,
            // }).await/* ) */;
        }
        // join_all(handles).with_logger(logger()).await;
        eprintln!("published")
    }
}

#[tower_lsp::async_trait]
impl<G, F> LanguageServer for Server<G, F>
where
    G: opengl::ShaderValidator + Send,
    F: Fn() -> G + Send + Sync,
{
    #[logging::with_trace_id]
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
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

        let mut manager = self.workspace_manager.lock().with_logger(logger()).await;

        // self.client
        //     .lock()
        //     .with_logger(logger())
        //     .await
        //     .send_notification::<TelemetryEvent>(serde_json::json!({
        //         "status": "loading",
        //         "message": "Building dependency graph...",
        //         "icon": "$(loading~spin)",
        //     }))
        //     .with_logger(logger())
        //     .await;

        manager.gather_workspaces(&root).with_logger(logger()).await;

        // self.client
        //     .lock()
        //     .with_logger(logger())
        //     .await
        //     .send_notification::<TelemetryEvent>(serde_json::json!({
        //         "status": "ready",
        //         "message": "Project(s) initialized...",
        //         "icon": "$(check)",
        //     }))
        //     .with_logger(logger())
        //     .await;

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

    async fn shutdown(&self) -> Result<()> {
        warn!("shutting down language server...");
        Ok(())
    }

    #[logging::with_trace_id]
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .lock()
            .with_logger(logger())
            .await
                .log_message(MessageType::INFO, "OPENED!")
                .with_logger(logger())
                .await;
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
        info!("opened document"; "uri" => params.text_document.uri.as_str());

        let path: NormalizedPathBuf = params.text_document.uri.into();
        if let Some(workspace) = self
            .workspace_manager
            .lock()
            .with_logger(logger())
            .await
            .find_workspace_for_file(&path)
        {
            trace!("found workspace"; "root" => &workspace.root);

            workspace.refresh_graph_for_file(&path).with_logger(logger()).await;

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
        match self
            .workspace_manager
            .lock()
            .with_logger(logger())
            .await
            .find_workspace_for_file(&path)
        {
            Some(workspace) => {
                trace!("found workspace"; "root" => &workspace.root);

                workspace.refresh_graph_for_file(&path).with_logger(logger()).await;

                match workspace.lint(&path).with_logger(logger()).await {
                    Ok(diagnostics) => self.publish_diagnostic(diagnostics, None).with_logger(logger()).await,
                    Err(e) => error!("error linting"; "error" => format!("{:?}", e), "path" => &path),
                }
            }
            None => warn!("no workspace found"; "path" => path),
        }
    }

    #[logging::with_trace_id]
    async fn execute_command(&self, params: ExecuteCommandParams) -> Result<Option<Value>> {
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
            // "virtualMerge" => {
            //     let document_path: NormalizedPathBuf = params.arguments.first().unwrap().try_into().unwrap();
            //     let manager = self.workspace_manager.lock().with_logger(logger()).await;
            //     let workspace = manager.find_workspace_for_file(&document_path).unwrap();
            //     let mut graph = workspace.graph.lock().with_logger(logger()).await;
            //     commands::merged_includes::run(&document_path, &mut graph)
            //         .with_logger(logger())
            //         .await
            //         .map_err(|e| Error {
            //             code: ErrorCode::InternalError,
            //             message: format!("{:?}", e),
            //             data: None,
            //         })
            // }
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

    async fn goto_definition(&self, _params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
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

    async fn references(&self, _params: ReferenceParams) -> Result<Option<Vec<Location>>> {
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

    async fn document_symbol(&self, _params: DocumentSymbolParams) -> Result<Option<DocumentSymbolResponse>> {
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

    async fn document_link(&self, _params: DocumentLinkParams) -> Result<Option<Vec<DocumentLink>>> {
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

    async fn did_change_configuration(&self, _params: DidChangeConfigurationParams) {
        eprintln!("got notif");
        /* logging::slog_with_trace_id(|| {
            #[derive(Deserialize)]
            struct Configuration {
                #[serde(alias = "logLevel")]
                log_level: String,
            }

            let config: Configuration = from_value(params.settings.as_object().unwrap().get("mcglsl").unwrap().to_owned()).unwrap();

            info!("got updated configuration"; "config" => params.settings.as_object().unwrap().get("mcglsl").unwrap().to_string());

            configuration::handle_log_level_change(config.log_level, |level| {
                self.log_guard = None; // set to None so Drop is invoked
                self.log_guard = Some(logging::set_logger_with_level(level));
            })
        }); */
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
            let files = fs::read_dir(&test_path)
                .unwrap()
                .map(|e| String::from(e.unwrap().path().to_str().unwrap()))
                .collect::<Vec<String>>();
            copy_items(&files, &tmp_dir.path().join("shaders"), opts).unwrap();
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

        assert_eq!(server.workspace_manager.lock().await.workspaces().len(), 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    #[logging_macro::scope]
    async fn test_01_initialize() {
        let mut server = new_temp_server(MockShaderValidator::new);

        let (_tmp_dir, tmp_path) = copy_to_tmp_dir("../testdata/01");

        let init_req = initialize::request(&tmp_path);
        let init_resp = Ok(initialize::response());
        assert_exchange!(&server, init_req, init_resp, Server::initialize);

        let manager = server.workspace_manager.lock().await;
        let workspaces = manager.workspaces();
        assert_eq!(
            workspaces.iter().map(|w| w.root.to_string()).collect::<Vec<String>>(),
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

        let manager = server.workspace_manager.lock().await;
        let workspaces = manager.workspaces();
        assert_eq!(
            workspaces.iter().map(|w| w.root.to_string()).collect::<Vec<String>>(),
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
