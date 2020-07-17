import * as vscode from 'vscode'
import * as lsp from 'vscode-languageclient'
import * as commands from './commands'
import { promptDownload, testExecutable } from './glslangValidator'
import { log } from './log'
import { LanguageClient } from './lspClient'

export const glslConfigParam = 'mcglsl.glslangValidatorPath'

export let statusBarItem: vscode.StatusBarItem | null = null

export class Extension {
  private extensionContext: vscode.ExtensionContext | null = null
  private client: lsp.LanguageClient
  
  public get context() : vscode.ExtensionContext {
    return this.extensionContext
  }

  public get lspClient() : lsp.LanguageClient {
    return this.client
  }
  
  public activate = async (context: vscode.ExtensionContext) => {
    this.extensionContext = context

    this.registerCommand('graphDot', commands.generateGraphDot)
    this.registerCommand('restart', commands.restartExtension)
    
    if (!testExecutable(vscode.workspace.getConfiguration().get(glslConfigParam) as string)) {
      if(!await promptDownload(this)) return
    }
  
    log.info('starting language server...')
  
    this.client = await new LanguageClient(this).startServer()
  
    log.info('language server started!')
  
  }

  registerCommand = (name: string, f: (e: Extension) => commands.Command) => {
    const cmd = f(this)
    this.context.subscriptions.push(vscode.commands.registerCommand('mcshader.'+name, cmd))
  }

  public deactivate = async () => {
    await this.lspClient.stop()
    while(this.context.subscriptions.length > 0) {
      this.context.subscriptions.pop()?.dispose()
    }
  }
  
  public updateStatus = (icon: string, text: string) => {
    statusBarItem?.dispose()
    statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left)
    statusBarItem.text = icon + ' [mc-shader] ' + text
    statusBarItem.show()
    this.context.subscriptions.push(statusBarItem)
  }
  
  public clearStatus = () => {
    statusBarItem?.dispose()
  }
}


export const activate = new Extension().activate