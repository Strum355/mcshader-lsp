import { dirname } from 'path'
import * as vsclang from 'vscode-languageserver'
import * as vsclangproto from 'vscode-languageserver-protocol'
import { completions } from './completionProvider'
import { conf, glslangReady, onConfigChange } from './config'
import { getDocumentLinks } from './linksProvider'
import { includeGraph, preprocess } from './linter'
import { formatURI, getDocumentContents, postError } from './utils'

const reVersion = /#version [\d]{3}/

export let connection = vsclang.createConnection(vsclang.ProposedFeatures.all)

console.log = connection.console.log.bind(connection.console)
console.error = connection.console.error.bind(connection.console)

export const documents = new vsclang.TextDocuments()
documents.listen(connection)

connection.onInitialize((params) => (
  {
    capabilities: {
      textDocumentSync: documents.syncKind,
      documentLinkProvider: {
        resolveProvider: true,
      },
      completionProvider: {
        resolveProvider: true
      },
    }
  }
))

connection.onExit(() => {})

documents.onDidOpen((event) => onEvent(event.document))

documents.onDidSave((event) => onEvent(event.document))

// what am i saying here
// dont do this for include files, for non-include files, clear diags for all its includes. Cache this maybe
documents.onDidClose((event) => {
  connection.sendDiagnostics({uri: event.document.uri, diagnostics: []})
})

//documents.onDidChangeContent(onEvent)

export function onEvent(document: vsclangproto.TextDocument) {
  if (conf.shaderpacksPath.replace(dirname(conf.shaderpacksPath), '') !== '/shaderpacks' || !glslangReady) return

  const uri = formatURI(document.uri)
  if (includeGraph.get(uri).parents.size > 0) {
    lintBubbleDown(uri)
    return
  }

  // i think we still need to keep this in case we havent found the root of this files include tree
  const lines = document.getText().split('\n')
  const hasVersion = lines.filter(l => reVersion.test(l)).length > 0
  if (!hasVersion) return

  try {
    preprocess(document.getText().split('\n'), uri)
  } catch (e) {
    postError(e)
  }
}

function lintBubbleDown(uri: string) {
  includeGraph.get(uri).parents.forEach((parent, parentURI) => {
    if (parent.second.parents.size > 0) {
      lintBubbleDown(parentURI)
    } else {
      const lines = getDocumentContents(parentURI).split('\n')
      // feel like we could perhaps do better? Hope no one puts #version at the top of their includes..
      if (lines.filter(l => reVersion.test(l)).length > 0) {
        try {
          preprocess(lines, parentURI)
        } catch (e) {
          postError(e)
        }
      }
    }
  })
}

connection.onDocumentLinks((params: vsclang.DocumentLinkParams)  => getDocumentLinks(params.textDocument.uri))

connection.onDidChangeConfiguration(onConfigChange)

connection.onCompletion((textDocumentPosition: vsclang.TextDocumentPositionParams) => completions)

connection.onCompletionResolve((item: vsclang.CompletionItem): vsclang.CompletionItem => completions[item.data - 1])

connection.listen()