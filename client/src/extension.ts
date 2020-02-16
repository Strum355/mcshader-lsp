import * as path from 'path'
import * as vscode from 'vscode'
import * as vscodeLang from 'vscode-languageclient'
import { promptDownload, testExecutable } from './glslangValidator'

export const glslConfigParam = 'mcglsl.glslangValidatorPath'

export let outputChannel: vscode.OutputChannel

let statusBarItem: vscode.StatusBarItem

let globalContext: vscode.ExtensionContext

export async function activate(context: vscode.ExtensionContext) {
  outputChannel = vscode.window.createOutputChannel('vscode-mc-shader')
  globalContext = context

  {
    if (!testExecutable(vscode.workspace.getConfiguration().get(glslConfigParam))) {
      await promptDownload()
    } else {
      outputChannel.appendLine('glslangValidator found!')
    }
  }
  
  {

    const clientOpts: vscodeLang.LanguageClientOptions = {
      documentSelector: [{scheme: 'file', language: 'glsl'}],
      outputChannel: outputChannel,
      outputChannelName: 'vscode-mc-shader',
      synchronize: {
        configurationSection: 'mcglsl',
        fileEvents: vscode.workspace.createFileSystemWatcher('**/*.{fsh,gsh,vsh,glsl}')
      },
    }

    const serverOpts: vscodeLang.ServerOptions = {
      command: context.asAbsolutePath(path.join('server', 'target', 'debug', 'vscode-mc-shader')), 
    }

    outputChannel.appendLine('starting language server...')

    const langServer = new vscodeLang.LanguageClient('vscode-mc-shader', serverOpts, clientOpts)

    context.subscriptions.push(langServer.start())
    
    await langServer.onReady()

    langServer.onNotification('updateConfig', (dir: string) => {
      vscode.workspace.getConfiguration().update(glslConfigParam, dir, vscode.ConfigurationTarget.Global)
    })

    langServer.onNotification('status', updateStatus)

    langServer.onNotification('clearStatus', clearStatus)

    outputChannel.appendLine('language server started!')
  }
}

export function updateStatus(icon: string, text: string) {
  if(statusBarItem != null) statusBarItem.dispose()
  statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left)
  statusBarItem.text = icon + " [Minecraft Shaders] " + text
  statusBarItem.show()
  globalContext.subscriptions.push(statusBarItem)
}

export function clearStatus() {
  if(statusBarItem != null) statusBarItem.dispose()
}