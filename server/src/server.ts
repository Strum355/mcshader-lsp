import * as vsclang from 'vscode-languageserver'
import * as vsclangproto from 'vscode-languageserver-protocol'
import { completions } from './completionProvider'
import { preprocess, ext, includeGraph } from './linter'
import { extname } from 'path'

const reVersion = /#version [\d]{3}/

export let connection: vsclang.IConnection
connection = vsclang.createConnection(new vsclang.IPCMessageReader(process), new vsclang.IPCMessageWriter(process))

import { onConfigChange } from './config'
import { formatURI, postError, getDocumentContents } from './utils'

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

// dont do this for include files, for non-include files, clear diags for all its includes. Cache this maybe
documents.onDidClose((event) => connection.sendDiagnostics({uri: event.document.uri, diagnostics: []}))

//documents.onDidChangeContent(onEvent)

export function onEvent(document: vsclangproto.TextDocument) {
  const uri = formatURI(document.uri)
  if (includeGraph.get(uri).parents.size > 0) {
    lintBubbleDown(uri, document)
    return
  }

  if (!ext.has(extname(document.uri))) return

  try {
    preprocess(document.getText().split('\n'), uri)
  } catch (e) {
    postError(e)
  }
}

function lintBubbleDown(uri: string, document: vsclangproto.TextDocument) {
  includeGraph.get(uri).parents.forEach((parent, parentURI) => {
    if (parent.second.parents.size > 0) {
      lintBubbleDown(parentURI, document)
    } else {
      const lines = getDocumentContents(parentURI).split('\n')
      if (lines.filter(l => reVersion.test(l)).length > 0) {
        preprocess(lines, parentURI)
      }
    }
  })
}

connection.onDidChangeConfiguration(onConfigChange)

connection.onCompletion((textDocumentPosition: vsclang.TextDocumentPositionParams) => completions)

connection.onCompletionResolve((item: vsclang.CompletionItem): vsclang.CompletionItem => completions[item.data - 1])

connection.listen()