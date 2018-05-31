import * as vscode from 'vscode'
import * as shell from 'shelljs'
import * as path from 'path'
import { runLinter } from '../asyncSpawn'
import '../global'
import { Config } from '../config'

// These are used for symlinking as glslangValidator only accepts files in these formats
const extensions: { [id: string]: string } = {
  '.fsh':  'frag',
  '.vsh':  'vert',
  '.gsh':  'geom',
  '.glsl': 'frag',
}

// These will be used to filter out error messages that are irrelevant/incorrect for us
// Lot of testing needed to find all the ones that we need to match
const filters: RegExp[] = [
  /(No code generated)/,
  /(compilation terminated)/,
  /\/\w*.(vert|frag)$/
]

const regInclude = /^(?: |\t)*(?:#include) "((?:\/[\S]+)+\.(?:glsl))"$/
export const regLinuxOutput = /^(WARNING|ERROR): ((?:\/[^/\n]*)+\/*):(\d+): ((?:'[\w\W]*'){1} :[\w ]+)/

export default class GLSLProvider implements vscode.CodeActionProvider {
  private diagnosticCollection: vscode.DiagnosticCollection // where errors/warnings/hints are pushed to be displayed
  private config: Config
  private onTypeDisposable?: vscode.Disposable

  constructor(subs: vscode.Disposable[]) {
    this.diagnosticCollection = vscode.languages.createDiagnosticCollection()

    subs.push(this)

    this.config = new Config()

    const c = vscode.workspace.getConfiguration('mcglsl')
    if (c.get('lintOnType') as boolean) {
      this.onTypeDisposable = vscode.workspace.onDidChangeTextDocument(this.docChange, this)
      console.log('[MC-GLSL] linting while typing.')
    } else {
      if (this.onTypeDisposable) this.onTypeDisposable.dispose()
      console.log('[MC-GLSL] not linting while typing.')
    }

    this.checkBinary()

    vscode.workspace.onDidOpenTextDocument(this.lint, this)
    vscode.workspace.onDidSaveTextDocument(this.lint, this)

    vscode.workspace.onDidChangeConfiguration((e: vscode.ConfigurationChangeEvent) => {
      this.config.onChange(e)
      this.checkBinary()
    }, this)

    vscode.workspace.textDocuments.forEach(doc => this.lint(doc))
  }

  // Check if glslangValidator binary can be found
  public checkBinary() {
    if (shell.which(this.config.glslangPath) == null) {
      const msg = '[MC-GLSL] glslangValidator not found. Please check that you\'ve given the right path.'
      console.log(msg)
      vscode.window.showErrorMessage(msg)
    }
  }

  public dispose = () => this.diagnosticCollection.dispose()

  // Maybe only lint when files are saved...hmmm
  private docChange = (e: vscode.TextDocumentChangeEvent) => this.lint(e.document)

  // Returns true if the string matches any of the regex
  public matchesFilters = (s: string) => filters.some(reg => reg.test(s))

  // Split output by line, remove empty lines and then remove all lines that match any of the regex
  private filterMessages = (res: string) => res
      .split('\n')
      .filter(s => s.length > 1 && !this.matchesFilters(s))
      .map(s => s.match(this.config.outputMatch))
      .filter(match => match && match.length > 3)

  // The big boi that does all the shtuff
  private async lint(document: vscode.TextDocument) {
    if (document.languageId !== 'glsl') return

    const ext = extensions[path.extname(document.fileName)]
    const root = '-I'
    let res = ''

    try {
      res = await runLinter(this.config.glslangPath, ['-E', '-S', ext, document.uri.path])
    } catch (e) {
      console.error(e)
      return
    }
    console.log(res)
    const messageMatches = this.filterMessages(res) as RegExpMatchArray[]
    console.log(messageMatches)

    const diags: vscode.Diagnostic[] = []

    messageMatches.forEach(match => {
      const [type, lineString, message] = match!.slice(1)
      const lineNum = parseInt(lineString)

      const severity: vscode.DiagnosticSeverity = type !== 'ERROR:' ? vscode.DiagnosticSeverity.Warning : vscode.DiagnosticSeverity.Error

      const range = this.calcRange(document, lineNum)

      //if (diags.length > 0 && range.isEqual(diags[diags.length - 1].range) && regSyntaxError.test(message)) return

      diags.push(new vscode.Diagnostic(range, '[mc-glsl] ' + message, severity))
    })

    this.findIncludes(document).forEach(include => {
      // path.join(this.config.workDir, match![1])
      if (include.text.includes('../')) {
        const trimmed = include.text.leftTrim()
        const offset = include.text.length - trimmed.length
        const range = new vscode.Range(include.lineNumber, offset, include.lineNumber, offset + trimmed.length)
        diags.push(new vscode.Diagnostic(range, '[mc-glsl] includes with .. directory movement will fail in zipped shaders.', vscode.DiagnosticSeverity.Warning))
      }
    })

    this.diagnosticCollection.set(document.uri, diags)
  }

  // Finds all lines that contain #include
  private findIncludes = (document: vscode.TextDocument) => this.filter(document, line => regInclude.test(line.text))

  private filter(document: vscode.TextDocument, f: (s: vscode.TextLine) => boolean): vscode.TextLine[] {
    const out: vscode.TextLine[] = []
    for (let i = 0; i < document.lineCount; i++) {
      if (f(document.lineAt(i))) out.push(document.lineAt(i))
    }
    return out
  }

  // Calculates the start and end character positions to underline
  private calcRange(document: vscode.TextDocument, lineNum: number): vscode.Range {
    const line = document.lineAt(lineNum - 1).text
    const trimmed = line.leftTrim()
    return new vscode.Range(lineNum - 1, line.length - trimmed.length, lineNum - 1, line.length - 1)
  }

  public provideCodeActions(document: vscode.TextDocument,
                            range: vscode.Range,
                            context: vscode.CodeActionContext,
                            token: vscode.CancellationToken): vscode.ProviderResult<vscode.Command[]> {
    throw new Error('Method not implemented.');
  }
}