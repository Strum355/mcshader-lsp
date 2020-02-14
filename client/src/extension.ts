import * as path from 'path'
import * as vscode from 'vscode'
import * as vscodeLang from 'vscode-languageclient'

export async function activate(context: vscode.ExtensionContext) {
  const outputChannel = vscode.window.createOutputChannel('vscode-mc-shader')

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

  langServer.onNotification('sampleText', (...nums: number[]) => {
    outputChannel.appendLine(`got notif: ${nums.join(' ')}`)
  })

  langServer.onNotification('update-config', (dir: string) => {
    vscode.workspace.getConfiguration().update('mcglsl.glslangValidatorPath', dir, vscode.ConfigurationTarget.Global)
  })

  outputChannel.appendLine('language server started!')
}