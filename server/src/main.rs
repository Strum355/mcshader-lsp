use rust_lsp::lsp::*;
use rust_lsp::lsp_types::*;
use rust_lsp::jsonrpc::*;
use rust_lsp::jsonrpc::method_types::*;

use walkdir;

use std::rc::Rc;
use std::cell::RefCell;
use std::ops::Add;
use std::io::{BufReader, BufRead};
use std::process;
use std::collections::HashMap;
use std::io;

use chan::WaitGroup;

use regex::Regex;

use lazy_static::lazy_static;

mod provider;
mod graph;
#[cfg(test)]
mod test;

lazy_static! {
    static ref RE_DIAGNOSTIC: Regex = Regex::new(r#"^(ERROR|WARNING): ([^?<>*|"]+?):(\d+): (?:'.*?' : )?(.+)\r?"#).unwrap();
    static ref RE_VERSION: Regex = Regex::new(r#"#version [\d]{3}"#).unwrap();
    static ref RE_INCLUDE: Regex = Regex::new(r#"^(?:\s)*?(?:#include) "(.+)"\r?"#).unwrap();
    static ref RE_INCLUDE_EXTENSION: Regex = Regex::new(r#"#extension GL_GOOGLE_include_directive ?: ?require"#).unwrap();
}

#[allow(dead_code)]
static INCLUDE_STR: &'static str = "#extension GL_GOOGLE_include_directive : require";

fn main() {
    let stdin = std::io::stdin();

    let endpoint_output = LSPEndpoint::create_lsp_output_with_output_stream(|| io::stdout());

    let cache_graph = graph::CachedStableGraph::new();

    let mut langserver = MinecraftShaderLanguageServer{
        endpoint: endpoint_output.clone(),
        graph: Rc::new(RefCell::new(cache_graph)),
        config: Configuration::default(),
        wait: WaitGroup::new(),
        root: None,
        command_provider: None,
    };

    langserver.command_provider = Some(provider::CustomCommandProvider::new(vec![
        ("graphDot", Box::new(provider::GraphDotCommand{
            graph: langserver.graph.clone(),
        }))
    ]));

    LSPEndpoint::run_server_from_input(&mut stdin.lock(), endpoint_output, langserver);
}

struct MinecraftShaderLanguageServer {
    endpoint: Endpoint,
    graph: Rc<RefCell<graph::CachedStableGraph>>,
    config: Configuration,
    wait: WaitGroup,
    root: Option<String>,
    command_provider: Option<provider::CustomCommandProvider>
}

#[derive(Default)]
struct Configuration {
    glslang_validator_path: String,
    shaderpacks_path: String,
}

impl Configuration {
    fn validate(&self) -> bool {
        if self.glslang_validator_path == "" || self.shaderpacks_path == "" {
            return false;
        }

        return true;
    }
}

struct GLSLFile {
    idx: petgraph::graph::NodeIndex,
    includes: Vec<String>,
}

impl MinecraftShaderLanguageServer {
    pub fn error_not_available<DATA>(data: DATA) -> MethodError<DATA> {
        let msg = "Functionality not implemented.".to_string();
        MethodError::<DATA> { code: 1, message: msg, data }
    }

    pub fn gen_initial_graph(&mut self, root: String) {
        self.endpoint.send_notification("status", vec!["$(loading~spin)", "Building project..."]).unwrap();
        let mut files = HashMap::new();

        eprintln!("root of project is {}", root);

        // filter directories and files not ending in any of the 3 extensions
        let file_iter = walkdir::WalkDir::new(root.clone()).into_iter().filter_map(|entry| {
            if !entry.is_ok() {
                return None;
            }

            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                return None;
            }

            let ext = path.extension().unwrap().to_str().unwrap();
            if ext != "vsh" && ext != "fsh" && ext != "glsl" && ext != "inc" {
                return None;
            }
            Some(String::from(path.to_str().unwrap()))
        });

        // iterate all valid found files, search for includes, add a node into the graph for each
        // file and add a file->includes KV into the map
        for entry_res in file_iter {
            let includes = self.find_includes(root.as_str(), entry_res.as_str());

            let stripped_path = String::from(String::from(entry_res));
            let idx = self.graph.borrow_mut().add_node(stripped_path.clone());

            //eprintln!("adding {} with\n{:?}", stripped_path.clone(), includes);
            
            files.insert(stripped_path, GLSLFile{idx, includes});
        }

        // Add edges between nodes, finding target nodes on weight (value)
        for (_, v) in files.into_iter() {
            for file in v.includes {
                //eprintln!("searching for {}", file);
                let idx = self.graph.borrow_mut().find_node(file.clone());
                if idx.is_none() {
                    eprintln!("couldn't find {} in graph for {}", file, self.graph.borrow().graph[v.idx]);
                    continue;
                }
                //eprintln!("added edge between\n\t{}\n\t{}", k, file);
                self.graph.borrow_mut().graph.add_edge(v.idx, idx.unwrap(), String::from("includes"));
            }
        }

        eprintln!("finished building project include graph");
        std::thread::sleep(std::time::Duration::from_secs(1));
        self.endpoint.send_notification("status", vec!["$(check)", "Finished building project!"]).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(3));
        self.endpoint.send_notification("clearStatus", None::<()>).unwrap();
    }

    pub fn find_includes(&self, root: &str, file: &str) -> Vec<String> {
        let mut includes = Vec::default();

        let buf = BufReader::new(std::fs::File::open(file).unwrap());
        buf.lines()
            .filter_map(|line| line.ok())
            .filter(|line| RE_INCLUDE.is_match(line.as_str()))
            .for_each(|line| {
                let caps = RE_INCLUDE.captures(line.as_str()).unwrap();
                //eprintln!("{:?}", caps);
                let mut path: String = String::from(caps.get(1).unwrap().as_str());
                if !path.starts_with("/") {
                    path.insert(0, '/');
                }
                let full_include = String::from(root).add("/shaders").add(path.as_str());
                includes.push(full_include.clone());
                //eprintln!("{} includes {}", file, full_include);
            });

        return includes;
    }

    pub fn lint(&self) {
        process::Command::new("program: S");
    }
}

impl LanguageServerHandling for MinecraftShaderLanguageServer {
    fn initialize(&mut self, params: InitializeParams, completable: MethodCompletable<InitializeResult, InitializeError>) {
        self.wait.add(1);

        let mut capabilities = ServerCapabilities::default();
        capabilities.hover_provider = Some(true);
        capabilities.document_link_provider = Some(DocumentLinkOptions{
            resolve_provider: None,
            work_done_progress_options: WorkDoneProgressOptions{
                work_done_progress: None,
            },
        });
        capabilities.execute_command_provider = Some(ExecuteCommandOptions{
            commands: vec![String::from("graphDot")],
            work_done_progress_options: WorkDoneProgressOptions{
                work_done_progress: None,
            },
        });
        capabilities.text_document_sync = Some(TextDocumentSyncCapability::Options(TextDocumentSyncOptions{
            open_close: Some(true),
            will_save: None,
            will_save_wait_until: None,
            change: Some(TextDocumentSyncKind::Full),
            save: Some(SaveOptions{
                include_text: Some(true),
            })
        }));

        self.root = Some(String::from(params.root_uri.unwrap().path()));

        completable.complete(Ok(InitializeResult{capabilities, server_info: None}));

        self.gen_initial_graph(self.root.clone().unwrap());
        self.lint();
    }
    
    fn shutdown(&mut self, _: (), completable: LSCompletable<()>) {
        completable.complete(Ok(()));
    }
    
    fn exit(&mut self, _: ()) {
        self.endpoint.request_shutdown();
    }
    
    fn workspace_change_configuration(&mut self, params: DidChangeConfigurationParams) {
        let config = params.settings.as_object().unwrap().get("mcglsl").unwrap();

        self.config.glslang_validator_path = String::from(config.get("glslangValidatorPath").unwrap().as_str().unwrap());
        self.config.shaderpacks_path = String::from(config.get("shaderpacksPath").unwrap().as_str().unwrap());

        if !self.config.validate() {
            self.endpoint.send_notification("badConfig", None::<()>).unwrap();
        }

        eprintln!("{:?}", params.settings.as_object().unwrap());

        self.wait.done();
    }
    
    fn did_open_text_document(&mut self, _: DidOpenTextDocumentParams) {}
    
    fn did_change_text_document(&mut self, params: DidChangeTextDocumentParams) {
        self.wait.wait();
        
        #[allow(unused_variables)]
        let text_change = params.content_changes.get(0).unwrap();
        //eprintln!("changed {} changes: {}", text_change., params.text_document.uri);
    }
    
    fn did_close_text_document(&mut self, _: DidCloseTextDocumentParams) {}

    fn did_save_text_document(&mut self, params: DidSaveTextDocumentParams) {
        self.wait.wait();
        eprintln!("saved {}", params.text_document.uri);
    }

    fn did_change_watched_files(&mut self, _: DidChangeWatchedFilesParams) {}
    
    fn completion(&mut self, _: TextDocumentPositionParams, completable: LSCompletable<CompletionList>) {
        completable.complete(Err(Self::error_not_available(())));
    }
    
    fn resolve_completion_item(&mut self, _: CompletionItem, completable: LSCompletable<CompletionItem>) {
        completable.complete(Err(Self::error_not_available(())));
    }
    fn hover(&mut self, _: TextDocumentPositionParams, completable: LSCompletable<Hover>) {
        self.wait.wait();
        self.endpoint.send_notification("sampleText", vec![1,2,3]).unwrap();
        completable.complete(Ok(Hover{
            contents: HoverContents::Markup(MarkupContent{
                kind: MarkupKind::Markdown,
                value: String::from("# Hello World"),
            }),
            range: None,
        }));
    }

    fn execute_command(&mut self, mut params: ExecuteCommandParams, completable: LSCompletable<WorkspaceEdit>) {
        params.arguments.push(serde_json::Value::String(self.root.clone().unwrap()));
        match self.command_provider.as_ref().unwrap().execute(params.command, params.arguments) {
            Ok(_) => completable.complete(Ok(WorkspaceEdit{
                changes: None,
                document_changes: None,
            })),
            Err(err) => completable.complete(Err(MethodError::new(32420, err, ())))
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
        let node = match self.graph.borrow_mut().find_node(params.text_document.uri.to_file_path().unwrap().as_os_str().to_str().unwrap().to_string()) {
            Some(n) => n,
            None => return,
        };

        let edges: Vec<DocumentLink> = self.graph.borrow().neighbors(node).into_iter().filter_map(|value| {
            let path = std::path::Path::new(&value);
            let url = match Url::from_file_path(path) {
                Ok(url) => url,
                Err(e) => {
                    eprintln!("error converting {:?} into url: {:?}", path, e);
                    return None;
                }
            };
            Some(DocumentLink{
                range: Range::new(Position::new(18, 0), Position::new(18, 20)),
                target: url,
                tooltip: None,
            })
        }).collect();
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