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
    this.checkBinary()

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

    console.log('glslangValidatorPath set to', c.get('glslangValidatorPath'))
    console.log('temp dir set to', os.tmpdir())

    return {
      glslangPath: c.get('glslangValidatorPath') as string,
      tmpdir: os.tmpdir()
    }
  }

  private checkBinary() {
    var isWin = require('os').platform().indexOf('win') > -1;

    var out = cp.execSync(`${isWin ? 'where' : 'whereis'} ${this.config.glslangPath}`);

    if (out.toString().split(' ')[1] == null) {
      vscode.window.showErrorMessage(
        'glslangValidator not found. Please check that you\'ve given the right path.' +
        ' Use the config option "mcglsl.glslangValidatorPath" to point to its location'
      )
    } else {
      vscode.window.showInformationMessage('glslangValidator found!')
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

/*     let diags: vscode.Diagnostic[] = []
    let diag = new vscode.Diagnostic(new vscode.Range(11, 0, 11, 0), 'stuff n things', vscode.DiagnosticSeverity.Error)
    diags.push(diag)
    this.diagnosticCollection.set(document.uri, diags) */
    /* fs.mkdirSync(`${this.config.tmpdir}/shaders`)
    sym(`${vscode.workspace.rootPath}/shaders/composite.frag`, `${this.config.tmpdir}/shaders/composite.banana`)
      .catch((err) => {console.log(err)}) */
  }

  public provideCodeActions(document: vscode.TextDocument, 
                            range: vscode.Range, 
                            context: vscode.CodeActionContext, 
                            token: vscode.CancellationToken): vscode.ProviderResult<vscode.Command[]> {
    throw new Error('Method not implemented.');
  }
}