import { mkdirSync, promises as fs } from 'fs'
import * as vscode from 'vscode'
import * as lsp from 'vscode-languageclient/node'
import * as commands from './commands'
import { log } from './log'
import { LanguageClient } from './lspClient'
import { download, getReleaseInfo } from './net'
import { PersistentState } from './persistent_state'
import * as path from 'path'

const platforms: { [key: string]: string } = {
  'x64 win32': 'x86_64-windows-msvc',
  'x64 linux': 'x86_64-unknown-linux-gnu',
  'x64 darwin': 'x86_64-apple-darwin',
  'arm64 darwin': 'aarch64-apple-darwin'
}

export class Extension {
  private statusBarItem: vscode.StatusBarItem | null = null
  private extensionContext: vscode.ExtensionContext | null = null
  private client: lsp.LanguageClient
  private state: PersistentState

  readonly extensionID = 'strum355.vscode-mc-shader'

  readonly package: {
    version: string
  } = vscode.extensions.getExtension(this.extensionID)!.packageJSON;

  public get context(): vscode.ExtensionContext {
    return this.extensionContext
  }

  public get lspClient(): lsp.LanguageClient {
    return this.client
  }

  public activate = async (context: vscode.ExtensionContext) => {
    this.extensionContext = context
    this.state = new PersistentState(context.globalState)

    if (!process.env['MCSHADER_DEBUG'] && !(vscode.workspace.getConfiguration('mcglsl').get('skipBootstrap') as boolean)) {
      await this.bootstrap()
    } else {
      log.info('skipping language server bootstrap')
    }

    this.registerCommand('graphDot', commands.generateGraphDot)
    this.registerCommand('restart', commands.restartExtension)
    this.registerCommand('virtualMerge', commands.virtualMergedDocument)
    this.registerCommand('parseTree', commands.parseTree)

    log.info('starting language server...')

    const lspBinary = process.env['MCSHADER_DEBUG'] ?
      this.context.asAbsolutePath(path.join('server', 'target', 'debug', 'mcshader-lsp')) +
      (process.platform === 'win32' ? '.exe' : '') :
      path.join(this.context.globalStorageUri.fsPath, 'mcshader-lsp')

    const filewatcherGlob = this.fileAssociationsToGlob(this.getGLSLFileAssociations())

    this.client = await new LanguageClient(this, lspBinary, filewatcherGlob).startServer()

    log.info('language server started!')
  }

  fileAssociationsToGlob = (associations: string[]): string => {
    return '**/*.{'.concat(
      associations.map(s => s.substring(s.indexOf('.'))).join(',')
    ) + '}'
  }

  getGLSLFileAssociations = (): string[] => {
    const exts = ['.fsh', '.vsh', '.gsh', '.glsl']
    const associations = vscode.workspace.getConfiguration('files').get('associations') as { [key: string]: string }

    Object.keys(associations).forEach((key) => {
      if (associations[key] === 'glsl') {
        exts.push(key.substring(key.indexOf('*') + 1))
      }
    })

    return exts
  }

  registerCommand = (name: string, f: (e: Extension) => commands.Command) => {
    const cmd = f(this)
    this.context.subscriptions.push(vscode.commands.registerCommand('mcglsl.' + name, cmd))
  }

  deactivate = async () => {
    await this.lspClient.stop()
    while (this.context.subscriptions.length > 0) {
      this.context.subscriptions.pop()?.dispose()
    }
  }

  public updateStatus = (icon: string, text: string) => {
    this.statusBarItem?.dispose()
    this.statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left)
    this.statusBarItem.text = icon + ' [mc-shader] ' + text
    this.statusBarItem.show()
    this.context.subscriptions.push(this.statusBarItem)
  }

  public clearStatus = () => {
    this.statusBarItem?.dispose()
  }

  private bootstrap = async () => {
    mkdirSync(this.extensionContext.globalStoragePath, { recursive: true })

    const dest = path.join(this.extensionContext.globalStoragePath, 'mcshader-lsp' + (process.platform === 'win32' ? '.exe' : ''))
    const exists = await fs.stat(dest).then(() => true, () => false)
    if (!exists) await this.state.updateServerVersion(undefined)

    const release = await getReleaseInfo(this.package.version)
    log.info('got release info from Github:\n\t', JSON.stringify(release))

    const platform = platforms[`${process.arch} ${process.platform}`]
    if (platform === undefined) {
      vscode.window.showErrorMessage('Unfortunately we don\'t ship binaries for your platform yet.')
      log.warn(`incompatible architecture/platform:\n\t${process.arch} ${process.platform}`)
      return
    }

    if (release.tag_name === this.state.serverVersion) {
      log.info('server version is same as extension:\n\t', this.state.serverVersion)
      return
    }

    const artifact = release.assets.find(artifact => artifact.name === `mcshader-lsp-${platform}${(process.platform === 'win32' ? '.exe' : '')}`)

    log.info(`artifact with url ${artifact.browser_download_url} found`)

    const userResponse = await vscode.window.showInformationMessage(
      this.state.serverVersion == undefined ?
        `Language server version ${this.package.version} is not installed.` :
        `An update is available. Upgrade from ${this.state.serverVersion} to ${release.tag_name}?`,
      'Download now'
    )
    if (userResponse !== 'Download now') {
      log.info('user chose not to download server...')
      return
    }

    await download(artifact.browser_download_url, dest)

    this.state.updateServerVersion(release.tag_name)
  }
}

export const activate = async (context: vscode.ExtensionContext) => {
  try {
    new Extension().activate(context)
  } catch (e) {
    log.error(`failed to activate extension: ${e}`)
    throw(e)
  }
}