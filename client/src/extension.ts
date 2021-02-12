import { mkdirSync, promises as fs } from 'fs'
import * as vscode from 'vscode'
import * as lsp from 'vscode-languageclient'
import * as commands from './commands'
import { log } from './log'
import { LanguageClient } from './lspClient'
import { download, getReleaseInfo } from './net'
import { PersistentState } from './persistent_state'
import * as path from 'path' 

const platforms: { [key: string]: string } = {
  'x64 win32': 'x86_64-pc-windows-msvc',
  'x64 linux': 'x86_64-unknown-linux-gnu',
  'x64 darwin': 'x86_64-apple-darwin',
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

    await this.bootstrap()

    this.registerCommand('graphDot', commands.generateGraphDot)
    this.registerCommand('restart', commands.restartExtension)
    this.registerCommand('virtualMerge', commands.virtualMergedDocument)

    log.info('starting language server...')
  
    this.client = await new LanguageClient(this).startServer()
  
    log.info('language server started!')
  }

  registerCommand = (name: string, f: (e: Extension) => commands.Command) => {
    const cmd = f(this)
    this.context.subscriptions.push(vscode.commands.registerCommand('mcglsl.'+name, cmd))
  }

   deactivate = async () => {
    await this.lspClient.stop()
    while(this.context.subscriptions.length > 0) {
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

    const platform = platforms[`${process.arch} ${process.platform}`]
    if (platform === undefined) {
      vscode.window.showErrorMessage('Unfortunately we don\'t ship binaries for your platform yet.')
      return
    }
    
    if (release.tag_name === this.state.serverVersion) return

    const artifact = release.assets.find(artifact => artifact.name === `mcshader-lsp-${platform}${(process.platform === 'win32' ? '.exe' : '')}`)

    const userResponse = await vscode.window.showInformationMessage(
      this.state.serverVersion == undefined ?
      `Language server version ${this.package.version} is not installed.` :
      `An update is available. Upgrade from ${this.state.serverVersion} to ${release.tag_name}?`,
      'Download now'
    )
    if (userResponse !== 'Download now') return

    await download(artifact.browser_download_url, dest)
    
    this.state.updateServerVersion(release.tag_name)
  }
}

export const activate = new Extension().activate