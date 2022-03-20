import { ConfigurationTarget, workspace } from 'vscode'
import * as lsp from 'vscode-languageclient'
import { Extension } from './extension'
import { log, lspOutputChannel } from './log'
import { ConfigUpdateParams, statusMethod, StatusParams, updateConfigMethod } from './lspExt'

export class LanguageClient extends lsp.LanguageClient {
  private extension: Extension

  constructor(ext: Extension, lspBinary: string, filewatcherGlob: string) {
    super('vscode-mc-shader', 'VSCode MC Shader', {
      command: lspBinary
    }, {
      documentSelector: [{scheme: 'file', language: 'glsl'}],
      outputChannel: lspOutputChannel,
      synchronize: {
        configurationSection: 'mcglsl',
        fileEvents: workspace.createFileSystemWatcher(filewatcherGlob)
      },
    })
    this.extension = ext

    log.info('server receiving events for file glob:\n\t', filewatcherGlob)
    log.info('running with binary at path:\n\t', lspBinary)
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
