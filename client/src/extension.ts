import * as vscode from 'vscode'
import * as vscodeLang from 'vscode-languageclient'
import * as path from 'path'

export function activate(context: vscode.ExtensionContext) {
  const serverModule = context.asAbsolutePath(path.join('server', 'out', 'server.js'))

  const debugOpts = { execArgv: ['--nolazy', '--inspect=6009']}

  const serverOpts: vscodeLang.ServerOptions = {
    run: {
      module: serverModule, transport: vscodeLang.TransportKind.ipc
    },
    debug: {
      module: serverModule, transport: vscodeLang.TransportKind.ipc, options: debugOpts
    }
  }

  const clientOpts: vscodeLang.LanguageClientOptions = {
    documentSelector: [{scheme: 'file', language: 'glsl'}],
    synchronize: {
      configurationSection: 'mcglsl',
      fileEvents: vscode.workspace.createFileSystemWatcher('**/*.{fsh,gsh,vsh,glsl}')
    }
  }

  const langServer = new vscodeLang.LanguageClient('vscode-mc-shader', serverOpts, clientOpts)

  context.subscriptions.push(langServer.start())
}