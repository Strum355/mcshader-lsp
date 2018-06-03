import * as vsclang from 'vscode-languageserver'
import { Config } from './config'

const connection = vsclang.createConnection(new vsclang.IPCMessageReader(process), new vsclang.IPCMessageWriter(process));

const documents = new vsclang.TextDocuments();

documents.listen(connection);

const conf = new Config('', '')

connection.onInitialize((params): vsclang.InitializeResult => {
  return {
    capabilities: {
      textDocumentSync: vsclang.TextDocumentSyncKind.Incremental,
      completionProvider: {
        resolveProvider: true
      },
    }
  };
});

documents.onDidChangeContent((change) => {
  validateTextDocument(change.document);
});

connection.onDidChangeConfiguration((change) => {
  conf.onChange(change.settings as Config)
  documents.all().forEach(validateTextDocument);
});

connection.onDidChangeTextDocument((param) => {
  console.log(param.contentChanges)
})

connection.onDidOpenTextDocument((param) => {
  console.log(param)
})

connection.onDidCloseTextDocument((param) => {
  console.log(param)
})

function validateTextDocument(textDocument: vsclang.TextDocument) {
  const diagnostics: vsclang.Diagnostic[] = [];
  const lines = textDocument.getText().split(/\r?\n/g);
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const index = line.indexOf('typescript');
    if (index >= 0) {
      diagnostics.push({
        severity: vsclang.DiagnosticSeverity.Warning,
        range: {
          start: { line: i, character: index },
          end: { line: i, character: index + 10 }
        },
        message: `blah blah Todd Howard`,
        source: 'mcglsl'
      });
    }
  }
  // Send the computed diagnostics to VS Code.
  connection.sendDiagnostics({ uri: textDocument.uri, diagnostics });
}

connection.onCompletion((textDocumentPosition: vsclang.TextDocumentPositionParams): vsclang.CompletionItem[] => {
  return [
    {
      label: 'heldItemId',
      kind: vsclang.CompletionItemKind.Variable,
      data: 0,
    },{
      label: 'hot',
      kind: vsclang.CompletionItemKind.Property,
      data: 1
    }];
});

connection.onCompletionResolve((item: vsclang.CompletionItem): vsclang.CompletionItem => {
  if (item.data === 0) {
    item.documentation = 'blyat man'
    item.detail = 'Held item ID (main hand)'
  } else if (item.data === 1) {
    item.documentation = 'random'
    item.detail = 'something'
  }
  return item
});

connection.listen();