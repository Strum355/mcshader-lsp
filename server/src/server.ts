import * as vsclang from 'vscode-languageserver'
import * as vsclangproto from 'vscode-languageserver-protocol'
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

function onEvent(document: vsclangproto.TextDocument) {
  if (!ext.has(extname(document.uri))) return
  try {
    preprocess(document.getText().split('\n'), formatURI(document.uri))
  } catch (e) {
    connection.window.showErrorMessage(`[mc-glsl] ${e.message}`)
  }
}

connection.onDidChangeConfiguration(async (change) => {
  const temp = change.settings.mcglsl as Config
  conf = new Config(temp['shaderpacksPath'], temp['glslangValidatorPath'])
  try {
    execSync(conf.glslangPath)
    documents.all().forEach(document => onEvent)
  } catch (e) {
    if (e.status !== 1) {
      const chosen = await connection.window.showErrorMessage(
        `[mc-glsl] glslangValidator not found at: '${conf.glslangPath}' or returned non-0 code`,
        {title: 'Download'},
        {title: 'Cancel'}
      )
      console.log(chosen.title)
    }
  }
})

connection.onCompletion((textDocumentPosition: vsclang.TextDocumentPositionParams) => completions)

connection.onCompletionResolve((item: vsclang.CompletionItem): vsclang.CompletionItem => completions[item.data - 1])

connection.listen()