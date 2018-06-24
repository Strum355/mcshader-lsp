import * as vsclang from 'vscode-languageserver'
import { TextDocumentChangeEvent } from 'vscode-languageserver-protocol';
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
  //onEvent(event)
})

documents.onDidSave((event) => {
  onEvent(event)
})

documents.onDidChangeContent((event) => {
  //onEvent(event)
})

function onEvent(event: TextDocumentChangeEvent) {
  preprocess(event.document, true, [event.document.uri.replace(/^file:\/\//, '')])
}

connection.onDidChangeConfiguration((change) => {
  const temp = change.settings.mcglsl as Config
  conf = new Config(temp['shaderpacksPath'], temp['glslangValidatorPath'])
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