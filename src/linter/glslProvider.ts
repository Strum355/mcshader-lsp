'use strict';

import * as vscode from 'vscode'
import * as os from 'os'
import * as cp from 'child_process'
import * as sym from 'create-symlink'
import * as fs from 'fs'

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

    vscode.workspace.onDidOpenTextDocument(this.lint, this)
    vscode.workspace.onDidSaveTextDocument(this.lint, this)
    
    vscode.workspace.onDidChangeTextDocument(this.docChange, this)

    vscode.workspace.onDidChangeConfiguration(this.configChange, this)

    vscode.workspace.onDidCloseTextDocument((document: vscode.TextDocument) => {
       this.diagnosticCollection.delete(document.uri)
    }, null, subs)
  }

  private initConfig(): config {
    const c = vscode.workspace.getConfiguration('mcglsl')

    console.log('glslangValidatorPath set to', c.get('glslangValidatorPath'))
    console.log('temp dir set to', os.tmpdir())

    return {
      glslangPath: c.get('glslangValidatorPath') as string,
      tmpdir: os.tmpdir()
    }
  }

  public dispose() {
    this.diagnosticCollection.clear()
    this.diagnosticCollection.dispose()
  }

  private configChange(e: vscode.ConfigurationChangeEvent) {
    if (e.affectsConfiguration('mcglsl')) {
      console.log('config changed')
      this.config = this.initConfig()
    }
  }

  private docChange(e: vscode.TextDocumentChangeEvent) {
    this.lint(e.document)
  }

  private async lint(document: vscode.TextDocument) {
    if(document.languageId !== 'glsl') {
      return
    }

    fs.mkdirSync(`${this.config.tmpdir}/shaders`)
    sym(`${vscode.workspace.rootPath}/shaders/composite.frag`, `${this.config.tmpdir}/shaders/composite.banana`)
      .catch((err) => {console.log(err)})
  }

  public provideCodeActions(document: vscode.TextDocument, 
                            range: vscode.Range, 
                            context: vscode.CodeActionContext, 
                            token: vscode.CancellationToken): vscode.ProviderResult<vscode.Command[]> {
    throw new Error('Method not implemented.');
  }
}