use rust_lsp::lsp::*;
use rust_lsp::ls_types::*;
use rust_lsp::jsonrpc::*;
use rust_lsp::jsonrpc::method_types::*;

use std::io;

fn main() {
    let stdin = std::io::stdin();

    let endpoint_output = LSPEndpoint::create_lsp_output_with_output_stream(|| io::stdout());

    let langserver = MinecraftShaderLanguageServer{endpoint: endpoint_output.clone()};

    LSPEndpoint::run_server_from_input(&mut stdin.lock(), endpoint_output, langserver);
}

struct MinecraftShaderLanguageServer {
    endpoint: Endpoint,
}

impl MinecraftShaderLanguageServer {
    pub fn error_not_available<DATA>(data: DATA) -> MethodError<DATA> {
        let msg = "Functionality not implemented.".to_string();
        MethodError::<DATA> { code: 1, message: msg, data: data }
    }
}

impl LanguageServerHandling for MinecraftShaderLanguageServer {
    fn initialize(&mut self, _: InitializeParams, completable: MethodCompletable<InitializeResult, InitializeError>) {
        let mut capabilities = ServerCapabilities::default();
        capabilities.hover_provider = Some(true);

        completable.complete(Ok(InitializeResult { capabilities: capabilities }))
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