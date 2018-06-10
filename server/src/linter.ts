import { conf, connection, documents } from './server'
import './global'
import { TextDocument, Diagnostic, DiagnosticSeverity, Range } from 'vscode-languageserver';
import { exec } from 'child_process'

const reDiag = /(ERROR|WARNING): (?:\d):(\d+): '(?:.*)' : (.+)/

const filters = [
  /(No code generated)/,
  /(compilation terminated)/,
]

const matchesFilters = (s: string) => filters.some(reg => reg.test(s))

const filterMatches = (output: string) => output
  .split('\n')
  .filter(s => s.length > 1 && !matchesFilters(s))
  .map(s => s.match(reDiag))
  .filter(match => match && match.length === 4)

export function preprocess(document: TextDocument) {
  if (conf.minecraftPath === 'shaderpacks') return

  //const root = document.uri.replace(/^file:\/\//, '').replace(conf.minecraftPath, '').replace(path.basename(document.uri), '')
  lint(document.getText(), document.uri)
}

function lint(text: string, uri: string) {
  const child = exec(`${conf.glslangPath} --stdin -S frag`, (error, out, err) => {
    const diagnostics: Diagnostic[] = []
    const matches = filterMatches(out) as RegExpMatchArray[]
    matches.forEach((match) => {
      const [type, line, msg] = match.slice(1)
      diagnostics.push({
        severity: type === 'ERROR' ? DiagnosticSeverity.Error : DiagnosticSeverity.Warning,
        range: calcRange(parseInt(line), uri),
        message: msg,
        source: 'mc-glsl'
      })
    })
    connection.sendDiagnostics({uri, diagnostics})
  })
  child.stdin.write(text)
  child.stdin.end()
}

function calcRange(lineNum: number, uri: string): Range {
  const line = documents.get(uri).getText().split('\n')[lineNum - 1]
  return Range.create(lineNum - 1, line.length - line.leftTrim().length, lineNum - 1, prepareLine(line).length)
}

function prepareLine(line: string): string {
  return line.slice(0, line.indexOf('//')).rightTrim()
}