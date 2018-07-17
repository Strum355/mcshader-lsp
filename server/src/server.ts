import * as vsclang from 'vscode-languageserver'
import * as vsclangproto from 'vscode-languageserver-protocol'
import { completions } from './completionProvider'
import { preprocess, ext, formatURI } from './linter'
import { exec, execSync } from 'child_process'
import { extname } from 'path'
import fetch from 'node-fetch'
import { platform } from 'os'
import { createWriteStream, chmodSync, createReadStream, unlinkSync } from 'fs'
import * as unzip from 'unzip'

export let connection: vsclang.IConnection
connection = vsclang.createConnection(new vsclang.IPCMessageReader(process), new vsclang.IPCMessageWriter(process))

import { Config, onConfigChange } from './config'

export const documents = new vsclang.TextDocuments()
documents.listen(connection)

connection.onInitialize((params): vsclang.InitializeResult => (
  {
    capabilities: {
      textDocumentSync: documents.syncKind,
      completionProvider: {
        resolveProvider: true
      },
    }
  }
))

connection.onExit(() => {})

documents.onDidOpen((event) => onEvent(event.document))

documents.onDidSave((event) => onEvent(event.document))

documents.onDidClose((event) => connection.sendDiagnostics({uri: event.document.uri, diagnostics: []}))

//documents.onDidChangeContent(onEvent)

export function onEvent(document: vsclangproto.TextDocument) {
  if (!ext.has(extname(document.uri))) return
  try {
    preprocess(document.getText().split('\n'), formatURI(document.uri))
  } catch (e) {
    connection.window.showErrorMessage(`[mc-glsl] ${e.message}`)
  }
}

connection.onDidChangeConfiguration(onConfigChange)

connection.onCompletion((textDocumentPosition: vsclang.TextDocumentPositionParams) => completions)

connection.onCompletionResolve((item: vsclang.CompletionItem): vsclang.CompletionItem => completions[item.data - 1])

connection.listen()