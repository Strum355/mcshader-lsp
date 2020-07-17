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