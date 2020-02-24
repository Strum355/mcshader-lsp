use rust_lsp::lsp::*;
use rust_lsp::ls_types::*;
use rust_lsp::jsonrpc::*;
use rust_lsp::jsonrpc::method_types::*;

use walkdir;

use std::ops::Add;
/* use std::fs::OpenOptions;
use std::io::prelude::*; */
use std::io;
use std::io::{BufReader, BufRead};
use std::process;
/* use std::path::Path; */
use std::collections::HashMap;

/* use petgraph::visit::Dfs;
use petgraph::dot; */
use petgraph::graph::Graph;


use chan::WaitGroup;

use regex::Regex;

use lazy_static::lazy_static;

lazy_static! {
    static ref RE_DIAGNOSTIC: Regex = Regex::new(r#"^(ERROR|WARNING): ([^?<>*|"]+?):(\d+): (?:'.*?' : )?(.+)\r?"#).unwrap();
    static ref RE_VERSION: Regex = Regex::new(r#"#version [\d]{3}"#).unwrap();
    static ref RE_INCLUDE: Regex = Regex::new(r#"^(?:\s)*?(?:#include) "(.+)"\r?"#).unwrap();
    static ref RE_INCLUDE_EXTENSION: Regex = Regex::new(r#"#extension GL_GOOGLE_include_directive ?: ?require"#).unwrap();
}

static INCLUDE_STR: &'static str = "#extension GL_GOOGLE_include_directive : require";

fn main() {
    let stdin = std::io::stdin();

    let endpoint_output = LSPEndpoint::create_lsp_output_with_output_stream(|| io::stdout());

    /* let file = OpenOptions::new()
        .write(true)
        .open("/home/noah/TypeScript/vscode-mc-shader/graph.dot")
        .unwrap(); */

    let langserver = MinecraftShaderLanguageServer{
        endpoint: endpoint_output.clone(),
        graph: Graph::default(),
        config: Configuration::default(),
        wait: WaitGroup::new(),
        /* file: file, */
    };

    LSPEndpoint::run_server_from_input(&mut stdin.lock(), endpoint_output, langserver);
}

struct MinecraftShaderLanguageServer {
    endpoint: Endpoint,
    graph: Graph<String, String>,
    config: Configuration,
    wait: WaitGroup,
    /* file: std::fs::File, */
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
        MethodError::<DATA> { code: 1, message: msg, data: data }
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
            if ext != "vsh" && ext != "fsh" && ext != "glsl" {
                return None;
            }
            Some(String::from(path.to_str().unwrap()))
        });

        // iterate all valid found files, search for includes, add a node into the graph for each
        // file and add a file->includes KV into the map
        for entry_res in file_iter {
            let includes = self.find_includes(root.as_str(), entry_res.as_str());

            let stripped_path = String::from(String::from(entry_res));
            let idx = self.graph.add_node(stripped_path.clone());

            //eprintln!("adding {} with\n{:?}", stripped_path.clone(), includes);
            
            files.insert(stripped_path, GLSLFile{idx, includes});
        }

        // Add edges between nodes, finding target nodes on weight (value)
        for (_, v) in files.into_iter() {
            for file in v.includes {
                let mut iter = self.graph.node_indices();
                //eprintln!("searching for {}", file);
                let idx = iter.find(|i| self.graph[*i] == file);
                if idx.is_none() {
                    eprintln!("couldn't find {} in graph", file);
                    continue;
                }
                //eprintln!("added edge between\n\t{}\n\t{}", k, file);
                self.graph.add_edge(v.idx, idx.unwrap(), String::from("includes"));
            }
        }

        /* self.file.seek(std::io::SeekFrom::Start(0))?;
        self.file.write_all(dot::Dot::new(&self.graph).to_string().as_bytes())?;
        self.file.flush()?;
        self.file.seek(std::io::SeekFrom::Start(0))?; */

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
                let full_include = String::from(root).add("/shaders").add(caps.get(1).unwrap().as_str());
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
        capabilities.text_document_sync = Some(TextDocumentSyncCapability::Options(TextDocumentSyncOptions{
            open_close: Some(true),
            will_save: None,
            will_save_wait_until: None,
            change: Some(TextDocumentSyncKind::Full),
            save: Some(SaveOptions{
                include_text: Some(true),
            })
        }));

        completable.complete(Ok(InitializeResult{capabilities}));

        self.gen_initial_graph(params.root_path.unwrap());
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
    fn document_link(&mut self, _params: DocumentLinkParams, completable: LSCompletable<Vec<DocumentLink>>) {
        completable.complete(Err(Self::error_not_available(())));
    }
    fn document_link_resolve(&mut self, _params: DocumentLink, completable: LSCompletable<DocumentLink>) {
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