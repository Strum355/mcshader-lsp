import * as path from 'path'
import { ConfigurationTarget, workspace } from 'vscode'
import * as lsp from 'vscode-languageclient'
import { Extension } from './extension'
import { lspOutputChannel } from './log'
import { ConfigUpdateParams, statusMethod, StatusParams, updateConfigMethod } from './lspExt'

export class LanguageClient extends lsp.LanguageClient {
  private extension: Extension

  constructor(ext: Extension) {
    super('vscode-mc-shader', 'VSCode MC Shader', {
      command: process.env['MCSHADER_DEBUG'] ? 
      ext.context.asAbsolutePath(path.join('server', 'target', 'debug', 'mcshader-lsp')) + 
        (process.platform === 'win32' ? '.exe' : '') :
        path.join(ext.context.globalStoragePath, 'mcshader-lsp')
    }, {
      documentSelector: [{scheme: 'file', language: 'glsl'}],
      outputChannel: lspOutputChannel,
      synchronize: {
        configurationSection: 'mcglsl',
        fileEvents: workspace.createFileSystemWatcher('**/*.{fsh,gsh,vsh,glsl,inc}')
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
