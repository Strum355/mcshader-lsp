'use strict'

import * as vscode from 'vscode'
import * as os from 'os'
import * as cp from 'child_process'
import * as fs from 'fs'
import * as shell from 'shelljs'
import * as path from 'path'
import '../global'

// glslangPath: Path to glslangValidator (assumed in PATH by default)
// tmpdir: the directory into which the symlinks are stored, should be the OS's temp dir
interface Config {
  readonly glslangPath: string
  readonly workDir: string
  readonly tmpdir: string
  readonly isWin: boolean
}

// These are used for symlinking as glslangValidator only accepts files in these formats
const extensions: { [id: string]: string } = {
  '.fsh':  '.frag',
  '.vsh':  '.vert',
  '.gsh':  '.geom',
  '.glsl': '.frag',
}

// These will be used to filter out error messages that are irrelevant/incorrect for us
// Lot of testing needed to find all the ones that we need to match
const filters: RegExp[] = [
  /(required extension not requested: GL_GOOGLE_include_directive)/,
  /('#include' : must be followed by a header name)/,
  /('#include' : unexpected include directive)/,
  /(No code generated)/,
  /(compilation terminated)/,
  /\/\w*.(vert|frag)$/
]

const syntaxError = /(syntax error)/
const outputMatch = /(WARNING:|ERROR:)\s\d+:(\d+): (\W.*)/
const include = /^(?: |\t)*(?:#include) "((?:\/[\w_-]+)+\.(?:glsl))"$/

export default class GLSLProvider implements vscode.CodeActionProvider {
  private diagnosticCollection: vscode.DiagnosticCollection // where errors/warnings/hints are pushed to be displayed
  private config: Config
  private onTypeDisposable?: vscode.Disposable

  constructor(subs: vscode.Disposable[], config?: Config) {
    this.diagnosticCollection = vscode.languages.createDiagnosticCollection()

    subs.push(this)

    // For if i ever get testing to work
    if (config !== null) {
      this.config = this.initConfig()
    } else {
      this.config = config
    }

    this.checkBinary()

    try {
      shell.mkdir('-p', `${this.config.tmpdir}`)
      console.log('[MC-GLSL] Successfully made temp directory', `${this.config.tmpdir}`)
    } catch (e) {
      console.error('[MC-GLSL] Error creating temp dir', e)
      vscode.window.showErrorMessage('[MC-GLSL] Error creating temp directory. Check developer tools for more info.')
    }

    vscode.workspace.onDidOpenTextDocument(this.lint, this)
    vscode.workspace.onDidSaveTextDocument(this.lint, this)

    vscode.workspace.onDidChangeConfiguration(this.configChange, this)

    vscode.workspace.textDocuments.forEach((doc: vscode.TextDocument) => this.lint(doc))
  }

  private initConfig(): Config {
    const c = vscode.workspace.getConfiguration('mcglsl')

    console.log('[MC-GLSL] glslangValidatorPath set to', c.get('glslangValidatorPath'))
    console.log('[MC-GLSL] temp directory root set to', path.join(os.tmpdir(), vscode.workspace.name!, 'shaders'))

    if (c.get('lintOnSave') as boolean) {
      this.onTypeDisposable = vscode.workspace.onDidChangeTextDocument(this.docChange, this)
      console.log('[MC-GLSL] linting on save')
    } else {
      if (this.onTypeDisposable) this.onTypeDisposable.dispose()
      console.log('[MC-GLSL] not linting on save')
    }

    return {
      glslangPath: c.get('glslangValidatorPath') as string,
      workDir: vscode.workspace.rootPath!,
      tmpdir: path.join(os.tmpdir(), vscode.workspace.name!, 'shaders'),
      isWin: require('os').platform() === 'win32',
    }
  }

  // Called when the config files are changed
  private configChange(e: vscode.ConfigurationChangeEvent) {
    if (e.affectsConfiguration('mcglsl')) {
      console.log('[MC-GLSL] config changed')
      this.config = this.initConfig()
      this.checkBinary()
    }
  }

  // Check if glslangValidator binary can be found
  public checkBinary() {
    const ret = shell.which(this.config.glslangPath)

    if (ret == null) {
      const msg = '[MC-GLSL] glslangValidator not found. Please check that you\'ve given the right path.'
      console.log(msg)
      vscode.window.showErrorMessage(msg)
      return
    }

    // Do we want this here? ¯\_(ツ)_/¯
    // vscode.window.showInformationMessage('[MC-GLSL] glslangValidator found!')
    return
  }

  public getConfig = () => this.config

  public dispose = () => this.diagnosticCollection.dispose()

  // Maybe only lint when files are saved...hmmm
  private docChange = (e: vscode.TextDocumentChangeEvent) => this.lint(e.document)

  // Returns true if the string matches any of the regex
  public matchesFilters = (s: string) => filters.some((reg: RegExp) => reg.test(s))

  // Split output by line, remove empty lines, remove the first and 2 trailing lines,
  // and then remove all lines that match any of the regex
  private filterMessages = (res: string) => res
      .split('\n')
      .filter((s: string) => s.length > 1 && !this.matchesFilters(s))
      .map((s: string) => s.match(outputMatch))
      .filter(match => match && match.length > 3)

  private filterPerLine(matches: RegExpMatchArray[], document: vscode.TextDocument) {
    return matches.filter((match) => {
      let line = document.lineAt(parseInt(match![2]))
      return !(syntaxError.test(match[0]) && line.text.leftTrim().startsWith('#include'))
    })
  }

  // The big boi that does all the shtuff
  private lint(document: vscode.TextDocument) {
    if (document.languageId !== 'glsl') return
    
    const linkname = path.join(this.config.tmpdir, `${path.basename(document.fileName, path.extname(document.fileName))}${extensions[path.extname(document.fileName)]}`)
    
    this.createSymlinks(linkname, document)

    const res = cp.spawnSync(this.config.glslangPath, [linkname]).output[1].toString()
    let messageMatches = this.filterPerLine(this.filterMessages(res) as RegExpMatchArray[], document)

    const diags: vscode.Diagnostic[] = []

    messageMatches.forEach((match) => {
      const [type, lineString, message] = match!.slice(1)
      const lineNum = parseInt(lineString)

      // Default to error
      const severity: vscode.DiagnosticSeverity = type !== 'ERROR:' ? vscode.DiagnosticSeverity.Warning : vscode.DiagnosticSeverity.Error

      const range = this.calcRange(document, lineNum)

      if (diags.length > 0 && range.isEqual(diags[diags.length - 1].range) && syntaxError.test(message)) return

      diags.push(new vscode.Diagnostic(range, message, severity))
    })

    this.diagnosticCollection.set(document.uri, diags)
  }

  private mergeIncludes(document: vscode.TextDocument) {
    const includes = this.findIncludes(document)
  }

  private findIncludes = (document: vscode.TextDocument) => this
      .filter(document, (line: vscode.TextLine) => include.test(line.text))
      .map((line: vscode.TextLine) => line.text.match(include) || [])

  private filter(document: vscode.TextDocument, f: (s: vscode.TextLine) => boolean): vscode.TextLine[] {
    const out: vscode.TextLine[] = []
    for (let i = 0; i < document.lineCount; i++) {
      if (f(document.lineAt(i))) out.push(document.lineAt(i))
    }
    return out
  }

  private calcRange(document: vscode.TextDocument, lineNum: number): vscode.Range {
    const line = document.lineAt(lineNum - 1).text
    const trimmed = line.toString().leftTrim()
    return new vscode.Range(lineNum - 1, line.length - trimmed.length, lineNum - 1, line.length - 1)
  }

  private createSymlinks(linkname: string, document: vscode.TextDocument) {
    if (!fs.existsSync(linkname)) {
      console.log(`[MC-GLSL] ${linkname} does not exist yet. Creating`, this.config.isWin ? 'hard link.' : 'soft link.')

      if (this.config.isWin) shell.ln(document.uri.fsPath, linkname)
      else shell.ln('-sf', document.uri.fsPath, linkname)

      if (shell.error()) {
        console.error(`[MC-GLSL] ${shell.error()}`)
        vscode.window.showErrorMessage('[MC-GLSL] Error creating symlink')
      }
    }
  }

  public provideCodeActions(document: vscode.TextDocument,
                            range: vscode.Range,
                            context: vscode.CodeActionContext,
                            token: vscode.CancellationToken): vscode.ProviderResult<vscode.Command[]> {
    throw new Error('Method not implemented.');
  }
}