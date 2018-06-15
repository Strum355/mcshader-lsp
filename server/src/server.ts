import * as vsclang from 'vscode-languageserver'
import { Config } from './config'
import { completions } from './completionProvider';
import { preprocess } from './linter';
import { stat } from 'fs';

export const connection = vsclang.createConnection(new vsclang.IPCMessageReader(process), new vsclang.IPCMessageWriter(process));

export const documents = new vsclang.TextDocuments();
documents.listen(connection);

export let conf = new Config('', '')

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

documents.onDidOpen((event) => {
  preprocess(event.document)
})

documents.onDidSave((event) => {
  preprocess(event.document)
})

/* documents.onDidChangeContent((change) => {
  preprocess(change.document);
});
 */
connection.onDidChangeConfiguration((change) => {
  const temp = change.settings.mcglsl as Config
  conf = new Config(temp.minecraftPath, temp.glslangPath)
  stat(conf.glslangPath, (error) => {
    if (error) {
      connection.window.showErrorMessage('glslangValidator not found')
    }
  })
  documents.all().forEach(preprocess);
});

connection.onCompletion((textDocumentPosition: vsclang.TextDocumentPositionParams): vsclang.CompletionItem[] => {
  return completions
});

connection.onCompletionResolve((item: vsclang.CompletionItem): vsclang.CompletionItem => {
  return completions[item.data - 1]
});

connection.listen();