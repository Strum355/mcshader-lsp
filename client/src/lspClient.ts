import * as path from 'path'
import { ConfigurationTarget, workspace } from 'vscode'
import * as lsp from 'vscode-languageclient'
import { Extension } from './extension'
import { lspExceptionLogger } from './log'
import { ConfigUpdateParams, statusMethod, StatusParams, updateConfigMethod } from './lspExt'

export class LanguageClient extends lsp.LanguageClient {
  private extension: Extension

  constructor(ext: Extension) {
    super('vscode-mc-shader', 'VSCode MC Shader', {
      command: ext.context.asAbsolutePath(path.join('server', 'target', 'debug', 'vscode-mc-shader')), 
    }, {
      documentSelector: [{scheme: 'file', language: 'glsl'}],
      outputChannel: lspExceptionLogger,
      synchronize: {
        configurationSection: 'mcglsl',
        fileEvents: workspace.createFileSystemWatcher('**/*.{fsh,gsh,vsh,glsl}')
      },
    })
    this.extension = ext
  }

  public startServer = async (): Promise<LanguageClient> => {
    this.extension.context.subscriptions.push(this.start())

    await this.onReady()
    
    this.onNotification(updateConfigMethod, this.onUpdateConfig)
    this.onNotification(statusMethod, this.onStatusChange)
    
    return this
  }

  onStatusChange = (params: StatusParams) => {
    switch (params.status) {
      case 'loading':
      case 'ready':
      case 'failed':
        this.extension.updateStatus(params.icon, params.message)
        break
      case 'clear':
        this.extension.clearStatus()
        break
    }
  }

  onUpdateConfig = (params: ConfigUpdateParams) => {
    for (const kv of params.kv) {
      workspace.getConfiguration().update('mcglsl.' + kv.key, kv.value, ConfigurationTarget.Global)
    }
  }
}
