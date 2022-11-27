import { ChildProcess, spawn } from 'child_process'
import { ConfigurationTarget, workspace } from 'vscode'
import * as lsp from 'vscode-languageclient/node'
import { PublishDiagnosticsNotification, StreamInfo, TelemetryEventNotification } from 'vscode-languageclient/node'
import { Extension } from './extension'
import { log, lspOutputChannel, traceOutputChannel } from './log'
import { statusMethod, StatusParams } from './lspExt'

export class LanguageClient extends lsp.LanguageClient {
  private extension: Extension

  constructor(ext: Extension, lspBinary: string, filewatcherGlob: string) {
    const serverOptions = () => new Promise<ChildProcess>((resolve, reject) => {
      const childProcess = spawn(lspBinary, {
        env: {
          'RUST_BACKTRACE': 'true',
          ...process.env,
        }
      })
      childProcess.stderr.on('data', (data: Buffer) => {
        lspOutputChannel.appendLine(data.toString().trimRight())
      })
      childProcess.on('exit', (code, signal) => {
        lspOutputChannel.appendLine(`⚠️⚠️⚠️ Language server exited ` + (signal ? `from signal ${signal}` : `with exit code ${code}`) + ' ⚠️⚠️⚠️')
      })
      resolve(childProcess)
    })

    super('mcglsl', '', serverOptions, {
      traceOutputChannel: traceOutputChannel,
      diagnosticCollectionName: 'mcglsl',
      documentSelector: [{ scheme: 'file', language: 'glsl' }],
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
    this.extension.context.subscriptions.push(this.onNotification(TelemetryEventNotification.type, this.onStatusChange))

    await this.start()

    return this
  }

  onStatusChange = (params: {
    status: 'loading' | 'ready' | 'failed' | 'clear'
    message: string
    icon: string
  }) => {
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
}
