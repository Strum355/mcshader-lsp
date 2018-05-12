'use strict';

import * as vscode from 'vscode'

export default class GLSLProvider implements vscode.CodeActionProvider {
  private diagnosticCollection?: vscode.DiagnosticCollection

  public activate() {
    this.diagnosticCollection = vscode.languages.createDiagnosticCollection()

    vscode.workspace.onDidOpenTextDocument(this.lint, this)
    vscode.workspace.onDidSaveTextDocument(this.lint, this)
    
    vscode.workspace.onDidChangeTextDocument(this.docChange, this)

    vscode.workspace.onDidChangeConfiguration(this.configChange, this)

    vscode.workspace.textDocuments.forEach(this.lint, this)

    console.log(vscode.workspace.textDocuments)
  }

  public dispose() {
    if(this.diagnosticCollection != null) {
      this.diagnosticCollection.clear()
      this.diagnosticCollection.dispose()
    }
  }

  private configChange(e: vscode.ConfigurationChangeEvent) {
    if (e.affectsConfiguration('mcglsl')) {

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