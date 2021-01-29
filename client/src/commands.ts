import * as vscode from 'vscode'
import * as lsp from 'vscode-languageclient'
import { Extension } from './extension'
import { log } from './log'

export type Command = (...args: any[]) => unknown

export function generateGraphDot(e: Extension): Command {
  return async () => {
    await e.lspClient.sendRequest(lsp.ExecuteCommandRequest.type.method, {
      command: 'graphDot',
      arguments: [vscode.workspace.workspaceFolders[0].uri.path],
    })
  }
}

export function restartExtension(e: Extension): Command {
  return async () => {
    vscode.window.showInformationMessage('Reloading Minecraft GLSL language server...')
    await e.deactivate()
    await e.activate(e.context).catch(log.error)
  }
}

export function virtualMergedDocument(e: Extension): Command {
  const getVirtualDocument = async (path: string): Promise<string | null> => {
    let content: string = ''
    try {
      content = await e.lspClient.sendRequest<string>(lsp.ExecuteCommandRequest.type.method, {
        command: 'virtualMerge',
        arguments: [path]
      })
    } catch(e) {}

    return content
  }

  const docProvider = new class implements vscode.TextDocumentContentProvider {
    onDidChangeEmitter = new vscode.EventEmitter<vscode.Uri>();
    onDidChange = this.onDidChangeEmitter.event;

    provideTextDocumentContent(uri: vscode.Uri, __: vscode.CancellationToken): vscode.ProviderResult<string> {
      return getVirtualDocument(uri.path)
    }
  }

  e.context.subscriptions.push(vscode.workspace.registerTextDocumentContentProvider('mcglsl', docProvider))

  return async () => {
    const uri = vscode.window.activeTextEditor.document.uri
    const path = vscode.Uri.parse('mcglsl:' + uri.path)
    const doc = await vscode.workspace.openTextDocument(path)
    await vscode.window.showTextDocument(doc, {preview: true})
  }
}