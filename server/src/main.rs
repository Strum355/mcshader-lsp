use rust_lsp::lsp::*;
use rust_lsp::ls_types::*;
use rust_lsp::jsonrpc::*;
use rust_lsp::jsonrpc::method_types::*;

use walkdir;

use std::ops::Add;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io;
use std::path::Path;
use std::collections::HashMap;
use std::io::{BufReader, BufRead};

use petgraph::visit::Dfs;
use petgraph::dot;
use petgraph::graph::Graph;

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

    let file = OpenOptions::new()
        .write(true)
        .open("/home/noah/TypeScript/vscode-mc-shader/graph.dot")
        .unwrap();

    let langserver = MinecraftShaderLanguageServer{
        endpoint: endpoint_output.clone(),
        graph: Graph::default(),
        file: file,
    };

    LSPEndpoint::run_server_from_input(&mut stdin.lock(), endpoint_output, langserver);
}

struct MinecraftShaderLanguageServer {
    endpoint: Endpoint,
    graph: Graph<String, String>,
    file: std::fs::File,
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
        let mut files = HashMap::new();

        eprintln!("root of project is {}", root);
        for entry_res in walkdir::WalkDir::new(root.clone()).into_iter() {
            let entry = match entry_res {
                Ok(entry) => entry,
                Err(e) => {
                    eprintln!("error {} {:?}", e.path().unwrap_or(Path::new("")).display(), e);
                    break;
                },
            };

            let path = entry.path();

            if path.is_dir() {
                continue;
            }

            if !path.is_dir() {
                let ext = path.extension().unwrap().to_str().unwrap();
                if ext != "vsh" && ext != "fsh" && ext != "glsl" {
                    continue;
                }
            }

            let includes = self.find_includes(root.as_str(), path.to_str().unwrap());

            let stripped_path = String::from(String::from(path.to_str().unwrap()).trim_start_matches(root.as_str()));
            let idx = self.graph.add_node(stripped_path.clone());

            eprintln!("adding {} with\n{:?}", stripped_path.clone(), includes);
            
            files.insert(stripped_path, GLSLFile{
                idx: idx, includes: includes,
            });
        }

        // Add edges between nodes, finding target nodes on weight (value)
        for (k, v) in files.into_iter() {
            for file in v.includes {
                let mut iter = self.graph.node_indices();
                eprintln!("searching for {}", file);
                let idx = iter.find(|i| self.graph[*i] == file).unwrap();
                //eprintln!("added edge between\n\t{}\n\t{}", k, file);
                self.graph.add_edge(v.idx, idx, String::from("includes"));
            }
        }

        self.file.seek(std::io::SeekFrom::Start(0)).unwrap();
        self.file.write_all(dot::Dot::new(&self.graph).to_string().as_bytes()).unwrap();
        self.file.flush().unwrap();
        self.file.seek(std::io::SeekFrom::Start(0)).unwrap();
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
}

impl LanguageServerHandling for MinecraftShaderLanguageServer {
    fn initialize(&mut self, params: InitializeParams, completable: MethodCompletable<InitializeResult, InitializeError>) {
        let mut capabilities = ServerCapabilities::default();
        capabilities.hover_provider = Some(true);

        completable.complete(Ok(InitializeResult { capabilities: capabilities }));

        self.gen_initial_graph(params.root_path.unwrap());
    }
    fn shutdown(&mut self, _: (), completable: LSCompletable<()>) {
        completable.complete(Ok(()));
    }
    fn exit(&mut self, _: ()) {
        self.endpoint.request_shutdown();
    }
    
    fn workspace_change_configuration(&mut self, _: DidChangeConfigurationParams) {}
    fn did_open_text_document(&mut self, _: DidOpenTextDocumentParams) {}
    fn did_change_text_document(&mut self, _: DidChangeTextDocumentParams) {}
    fn did_close_text_document(&mut self, _: DidCloseTextDocumentParams) {}
    fn did_save_text_document(&mut self, _: DidSaveTextDocumentParams) {}
    fn did_change_watched_files(&mut self, _: DidChangeWatchedFilesParams) {}
    
    fn completion(&mut self, _: TextDocumentPositionParams, completable: LSCompletable<CompletionList>) {
        completable.complete(Err(Self::error_not_available(())));
    }
    fn resolve_completion_item(&mut self, _: CompletionItem, completable: LSCompletable<CompletionItem>) {
        completable.complete(Err(Self::error_not_available(())));
    }
    fn hover(&mut self, _: TextDocumentPositionParams, completable: LSCompletable<Hover>) {
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