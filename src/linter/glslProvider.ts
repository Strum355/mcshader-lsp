'use strict';

import * as vscode from 'vscode'
import * as os from 'os'
import * as cp from 'child_process'
import * as fs from 'fs'
import * as shell from 'shelljs'

interface config {
  glslangPath: string
  tmpdir: string
}

export default class GLSLProvider implements vscode.CodeActionProvider {
  private diagnosticCollection: vscode.DiagnosticCollection
  private config: config

  constructor(subs: vscode.Disposable[]) {
    this.diagnosticCollection = vscode.languages.createDiagnosticCollection()
    
    subs.push(this)
    this.config = this.initConfig()
    this.checkBinary()

    try {
      shell.mkdir('-p', `${this.config.tmpdir}/shaders`)
      console.log('[MC-GLSL] Successfully made temp directory', `${this.config.tmpdir}/shaders`)
    } catch(e) {
      console.error('[MC-GLSL] Error creating temp dir', e)
      vscode.window.showErrorMessage('[MC-GLSL] Error creat ing temp directory. Check developer tools for more info.')
    }

    vscode.workspace.onDidOpenTextDocument(this.lint, this)
    vscode.workspace.onDidSaveTextDocument(this.lint, this)
    
    vscode.workspace.onDidChangeTextDocument(this.docChange, this)

    vscode.workspace.onDidChangeConfiguration(this.configChange, this)

    vscode.workspace.textDocuments.forEach((doc: vscode.TextDocument) => {
      this.lint(doc)
    })
  }

  private initConfig(): config {
    const c = vscode.workspace.getConfiguration('mcglsl')

    console.log('[MC-GLSL] glslangValidatorPath set to', c.get('glslangValidatorPath'))
    console.log('[MC-GLSL] temp directory root set to', `${os.tmpdir()}/${vscode.workspace.name}`)

    return {
      glslangPath: c.get('glslangValidatorPath') as string,
      tmpdir: `${os.tmpdir()}/${vscode.workspace.name}`
    }
  }

  private checkBinary() {
    let ret = shell.which(this.config.glslangPath)

    if (ret == null) {
      vscode.window.showErrorMessage(
        '[MC-GLSL] glslangValidator not found. Please check that you\'ve given the right path.' +
        ' Use the config option "mcglsl.glslangValidatorPath" to point to its location'
      )
    } else {
      vscode.window.showInformationMessage('[MC-GLSL] glslangValidator found!')
    }
  }

  public dispose() {
    this.diagnosticCollection.clear()
    this.diagnosticCollection.dispose()
  }

  private configChange(e: vscode.ConfigurationChangeEvent) {
    if (e.affectsConfiguration('mcglsl')) {
      console.log('[MC-GLSL] config changed')
      this.config = this.initConfig()
      this.checkBinary()
    }
  }

  private docChange(e: vscode.TextDocumentChangeEvent) {
    this.lint(e.document)
  }

  private lint(document: vscode.TextDocument) {
    if(document.languageId !== 'glsl') {
      return
    }
  }

  public provideCodeActions(document: vscode.TextDocument, 
                            range: vscode.Range, 
                            context: vscode.CodeActionContext, 
                            token: vscode.CancellationToken): vscode.ProviderResult<vscode.Command[]> {
    throw new Error('Method not implemented.');
  }
}