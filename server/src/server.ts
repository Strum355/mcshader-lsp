import * as vsclang from 'vscode-languageserver'
import { Config } from './config'
import { completions } from './completionProvider';
import { preprocess } from './linter';
import { exec } from 'child_process';

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

connection.onExit(() => {

})

documents.onDidOpen((event) => {
  preprocess(event.document, true, [event.document.uri.replace(/^file:\/\//, '')])
})

documents.onDidSave((event) => {
  preprocess(event.document, true, [event.document.uri.replace(/^file:\/\//, '')])
})

/* documents.onDidChangeContent((change) => {
  preprocess(change.document);
});*/

connection.onDidChangeConfiguration((change) => {
  const temp = change.settings.mcglsl as Config
  conf = new Config(temp['minecraftPath'], temp['glslangValidatorPath'])
  exec(conf.glslangPath, (error) => {
    if (error['code'] !== 1) {
      connection.window.showErrorMessage(`[mc-glsl] glslangValidator not found at: ${conf.glslangPath}`)
      return
    }
    documents.all().forEach((document) => preprocess(document, true, [document.uri.replace(/^file:\/\//, '')]));
  })
});

connection.onCompletion((textDocumentPosition: vsclang.TextDocumentPositionParams): vsclang.CompletionItem[] => {
  return completions
});

connection.onCompletionResolve((item: vsclang.CompletionItem): vsclang.CompletionItem => {
  return completions[item.data - 1]
});

connection.listen();