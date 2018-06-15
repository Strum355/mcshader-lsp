import { conf, connection, documents } from './server'
import './global'
import { TextDocument, Diagnostic, DiagnosticSeverity, Range } from 'vscode-languageserver';
import { exec } from 'child_process'

const reDiag = /(ERROR|WARNING): (?:\d):(\d+): '(?:.*)' : (.+)/
const reVersion = /#version [\d]{3}/
const include = '#extension GL_GOOGLE_include_directive : require'

const filters = [
  /(No code generated)/,
  /(compilation terminated)/,
]

const tokens: {[key: string]: string} = {
  'SEMICOLON': ';',
  'COMMA': ',',
  'COLON': ':',
  'EQUAL': '=',
  'LEFT_PAREN': '(',
  'RIGHT_PAREN': ')',
  'DOT': '.',
  'BANG': '!',
  'DASH': '-',
  'TILDE': '~',
  'PLUS': '+',
  'STAR': '*',
  'SLASH': '/',
  'PERCENT': '%',
  'LEFT_ANGEL': '<',
  'RIGHT_ANGEL': '>',
  'VERICAL_BAR': '|',
  'CARET': '^',
  'AMPERSAND': '&',
  'QUESTION': '?',
  'LEFT_BRACKET': '[',
  'RIGHT_BRACKET': ']',
  'LEFT_BRACE': '{',
  'RIGHT_BRACE': '}'
}

const filterMatches = (output: string) => output
  .split('\n')
  .filter(s => s.length > 1 && !filters.some(reg => reg.test(s)))
  .map(s => s.match(reDiag))
  .filter(match => match && match.length === 4)

const replaceWord = (msg: string) => {
  for (const token of Object.keys(tokens)) {
    if (msg.includes(token)) {
      msg = msg.replace(token, tokens[token])
    }
  }
  return msg
}

export function preprocess(document: TextDocument) {
  const lines = document.getText().split('\n')
  let inComment = false
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]
    if (line.includes('/*')) inComment = true
    if (line.includes('*/')) inComment = false
    if (line.trim().startsWith('//')) break
    if (!inComment && reVersion.test(line)) {
      lines.splice(i + 1, 0, include)
      break
    }
    if (i === lines.length - 1) lines.splice(0, 0, include)
  }
  //console.log(lines.join('\n'))

  //const root = document.uri.replace(/^file:\/\//, '').replace(conf.minecraftPath, '').replace(path.basename(document.uri), '')
  lint(lines.join('\n'), document.uri)
}

function lint(text: string, uri: string) {
  const child = exec(`${conf.glslangPath} --stdin -S frag`, (error, out, err) => {
    if (error) return
    const diagnostics: Diagnostic[] = []
    const matches = filterMatches(out) as RegExpMatchArray[]
    matches.forEach((match) => {
      const [type, line, msg] = match.slice(1)
      diagnostics.push({
        severity: type === 'ERROR' ? DiagnosticSeverity.Error : DiagnosticSeverity.Warning,
        range: calcRange(parseInt(line) - 1, uri),
        message: replaceWord(msg),
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
  return Range.create(lineNum - 1, line.length - line.leftTrim().length, lineNum - 1, prepareLine(line).length + 1)
}

function prepareLine(line: string): string {
  return line.slice(0, line.indexOf('//')).rightTrim()
}