'use strict';

import * as vscode from 'vscode'
import * as os from 'os'
import * as cp from 'child_process'
import * as fs from 'fs'
import * as shell from 'shelljs'
import * as path from 'path'

interface config {
  glslangPath: string
  tmpdir: string
}

const extensions: { [id: string] : string } = {
  '.fsh':  '.frag',
  '.vsh':  '.vert',
  '.gsh':  '.geom',
  '.glsl': '.frag'
}

const filters: RegExp[] = [
  /(not supported for this version or the enabled extensions)/g
]

export default class GLSLProvider implements vscode.CodeActionProvider {
  private diagnosticCollection: vscode.DiagnosticCollection
  private config: config

  constructor(subs: vscode.Disposable[]) {
    this.diagnosticCollection = vscode.languages.createDiagnosticCollection()
    
    subs.push(this)
    this.config = this.initConfig()
    this.checkBinary()

    try {
      shell.mkdir('-p', `${this.config.tmpdir}`)
      console.log('[MC-GLSL] Successfully made temp directory', `${this.config.tmpdir}`)
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
    console.log('[MC-GLSL] temp directory root set to', path.join(os.tmpdir(), vscode.workspace.name!!, 'shaders'))

    return {
      glslangPath: c.get('glslangValidatorPath') as string,
      tmpdir: path.join(os.tmpdir(), vscode.workspace.name!!, 'shaders')
    }
  }

  private configChange(e: vscode.ConfigurationChangeEvent) {
    if (e.affectsConfiguration('mcglsl')) {
      console.log('[MC-GLSL] config changed')
      this.config = this.initConfig()
      this.checkBinary()
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

  private docChange(e: vscode.TextDocumentChangeEvent) {
    this.lint(e.document)
  }

  private matchesFilters(s: string): boolean {
    return filters.some((reg: RegExp, i: number, array: RegExp[]) => {
      let m = s.match(reg)
      console.log(s)
      console.log(m)
      return (m && m.length > 1)!!
    })
  }

  private lint(document: vscode.TextDocument) {
    if(document.languageId !== 'glsl') {
      return
    }

    let linkname = path.join(this.config.tmpdir, `${path.basename(document.fileName, path.extname(document.fileName))}${extensions[path.extname(document.fileName)]}`)

    if(!fs.existsSync(linkname)) {
      console.log(`[MC-GLSL] ${linkname} does not exist yet. Creating.`)
      shell.ln('-s', document.uri.fsPath, linkname)
    }

    let res = cp.spawnSync(this.config.glslangPath, [linkname]).output[1].toString()
    let lines = res.split(/(?:\n)/g)
      .filter((s: string) => { return s != '' })
      .slice(1, -1)
      //.filter((s: string) => { return !this.matchesFilters(s)} )

    if (lines.length < 1) {
      this.diagnosticCollection.set(document.uri, [])
      return
    }

    let diags: vscode.Diagnostic[] = []

    lines.forEach((line: string) => {
      // Default to error
      let matches = line.match(/WARNING:|ERROR:\s\d+:(\d+): (\W.*)/)
      if (!matches || (matches && matches.length < 3)) {
        return
      }

      let [lineNum, message] = matches.slice(1,3)

      let severity: vscode.DiagnosticSeverity = vscode.DiagnosticSeverity.Error
      if(!line.startsWith('ERROR:')) {
        // for now assume theres either errors or warnings. Maybe thats even the case!
        severity = vscode.DiagnosticSeverity.Warning
      }
      
      let range = new vscode.Range(parseInt(lineNum) -1, 0, parseInt(lineNum) - 1, 0)
      diags.push(new vscode.Diagnostic(range, message, severity))
    })
    this.diagnosticCollection.set(document.uri, diags)
  }

  public provideCodeActions(document: vscode.TextDocument, 
                            range: vscode.Range, 
                            context: vscode.CodeActionContext, 
                            token: vscode.CancellationToken): vscode.ProviderResult<vscode.Command[]> {
    throw new Error('Method not implemented.');
  }
}