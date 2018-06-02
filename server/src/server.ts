import { IPCMessageReader, IPCMessageWriter, createConnection, IConnection,
  TextDocuments, TextDocument, Diagnostic, DiagnosticSeverity, InitializeResult, TextDocumentPositionParams, CompletionItem, CompletionItemKind
} from 'vscode-languageserver';

// Create a connection for the server. The connection uses Node's IPC as a transport
const connection: IConnection = createConnection(new IPCMessageReader(process), new IPCMessageWriter(process));

// Create a simple text document manager. The text document manager
// supports full document sync only
const documents: TextDocuments = new TextDocuments();
// Make the text document manager listen on the connection
// for open, change and close text document events
documents.listen(connection);

// After the server has started, the client sends an initialize request. The server receives
// in the passed params, the rootPath of the workspace plus the client capabilities.
connection.onInitialize((params): InitializeResult => {
  return {
    capabilities: {
      // Tell the client that the server works in FULL text document sync mode
      textDocumentSync: documents.syncKind,
      // Tell the client that the server supports code completion
      completionProvider: {
        resolveProvider: true
      }
    }
  };
});

// The content of a text document has changed. This event is emitted
// when the text document is first opened or when its content has changed.
documents.onDidChangeContent((change) => {
  validateTextDocument(change.document);
});

// The settings have changed. It is sent on server activation
// as well.
connection.onDidChangeConfiguration((change) => {
  documents.all().forEach(validateTextDocument);
});

function validateTextDocument(textDocument: TextDocument): void {
  const diagnostics: Diagnostic[] = [];
  const lines = textDocument.getText().split(/\r?\n/g);
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const index = line.indexOf('typescript');
    if (index >= 0) {
      diagnostics.push({
        severity: DiagnosticSeverity.Warning,
        range: {
          start: { line: i, character: index },
          end: { line: i, character: index + 10 }
        },
        message: `${line.substr(index, 10)} should be spelled TypeScript`,
        source: 'ex'
      });
    }
  }
  // Send the computed diagnostics to VS Code.
  connection.sendDiagnostics({ uri: textDocument.uri, diagnostics });
}

connection.onDidChangeWatchedFiles((change) => {
  // Monitored files have changed in VS Code
  connection.console.log('We received a file change event');
});

// This handler provides the initial list of the completion items.
connection.onCompletion((textDocumentPosition: TextDocumentPositionParams): CompletionItem[] => {
  // The passed parameter contains the position in the text document in
  // which code completion was requested. For this example, we ignore this
  // information and always provide the same completion items.
  return [
    {
      label: 'TypeScript',
      kind: CompletionItemKind.Text,
      data: 1
    },
    {
      label: 'JavaScript',
      kind: CompletionItemKind.Text,
      data: 2
    }
  ];
});

// This handler resolves additional information for the item selected in
// the completion list.
connection.onCompletionResolve((item: CompletionItem): CompletionItem => {
  if (item.data === 1) {
    item.detail = 'TypeScript details',
      item.documentation = 'TypeScript documentation'
  } else if (item.data === 2) {
    item.detail = 'JavaScript details',
      item.documentation = 'JavaScript documentation'
  }
  return item;
});

/*
connection.onDidOpenTextDocument((params) => {
	// A text document was opened in VS Code.
	// params.uri uniquely identifies the document. For documents stored on disk, this is a file URI.
	// params.text the initial full content of the document.
	connection.console.log(`${params.textDocument.uri} opened.`);
});
connection.onDidChangeTextDocument((params) => {
	// The content of a text document has changed in VS Code.
	// params.uri uniquely identifies the document.
	// params.contentChanges describe the content changes to the document.
	connection.console.log(`${params.textDocument.uri} changed: ${JSON.stringify(params.contentChanges)}`);
});
connection.onDidCloseTextDocument((params) => {
	// A text document was closed in VS Code.
	// params.uri uniquely identifies the document.
	connection.console.log(`${params.textDocument.uri} closed.`);
});
*/

// Listen on the connection
connection.listen();