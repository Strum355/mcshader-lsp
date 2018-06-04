import * as vsclang from 'vscode-languageserver'
import { Config } from './config'
import { completions } from './completionProvider';

const connection = vsclang.createConnection(new vsclang.IPCMessageReader(process), new vsclang.IPCMessageWriter(process));

const documents = new vsclang.TextDocuments();

documents.listen(connection);

const conf = new Config('', '')

connection.onInitialize((params): vsclang.InitializeResult => {
  return {
    capabilities: {
      textDocumentSync: documents.syncKind,
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

function validateTextDocument(textDocument: vsclang.TextDocument): void {
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
        message: `bananas`,
        source: 'mcglsl'
      });
    }
  }
  // Send the computed diagnostics to VS Code.
  connection.sendDiagnostics({ uri: textDocument.uri, diagnostics });
}

connection.onCompletion((textDocumentPosition: vsclang.TextDocumentPositionParams): vsclang.CompletionItem[] => {
  return completions
});

connection.onCompletionResolve((item: vsclang.CompletionItem): vsclang.CompletionItem => {
  return completions[item.data - 1]
});

connection.listen();