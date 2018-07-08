import * as vsclang from 'vscode-languageserver'
import { TextDocumentChangeEvent, TextDocument } from 'vscode-languageserver-protocol'
import { Config } from './config'
import { completions } from './completionProvider'
import { preprocess, ext, formatURI } from './linter'
import { exec, execSync } from 'child_process'
import { extname } from 'path'

export const connection = vsclang.createConnection(new vsclang.IPCMessageReader(process), new vsclang.IPCMessageWriter(process))

export const documents = new vsclang.TextDocuments()
documents.listen(connection)

export let conf = new Config('', '')

connection.onInitialize((params): vsclang.InitializeResult => {
  return {
    capabilities: {
      textDocumentSync: documents.syncKind,
      completionProvider: {
        resolveProvider: true
      },
    }
  }
})

connection.onExit(() => {})

documents.onDidOpen((event) => onEvent(event.document))

documents.onDidSave((event) => onEvent(event.document))

//documents.onDidChangeContent(onEvent)

function onEvent(document: TextDocument) {
  if (!ext.has(extname(document.uri))) return
  preprocess(document.getText().split('\n'), formatURI(document.uri), true, [document.uri.replace(/^file:\/\//, '')], 0)
}

connection.onDidChangeConfiguration((change) => {
  const temp = change.settings.mcglsl as Config
  conf = new Config(temp['shaderpacksPath'], temp['glslangValidatorPath'])
  try {
    execSync(conf.glslangPath)
    documents.all().forEach(document => onEvent(document))
  } catch (e) {
    if (e.status !== 1) {
      connection.window.showErrorMessage(`[mc-glsl] glslangValidator not found at: '${conf.glslangPath}' or returned non-0 code`)
    }
  }
})

connection.onCompletion((textDocumentPosition: vsclang.TextDocumentPositionParams) => completions)

connection.onCompletionResolve((item: vsclang.CompletionItem): vsclang.CompletionItem => completions[item.data - 1])

connection.listen()