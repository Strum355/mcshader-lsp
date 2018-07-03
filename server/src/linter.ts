import { conf, connection, documents } from './server'
import { TextDocument, Diagnostic, DiagnosticSeverity, Range } from 'vscode-languageserver'
import { execSync } from 'child_process'
import * as path from 'path'
import { readFileSync } from 'fs'

const reDiag = /^(ERROR|WARNING): ([^?<>:*|"]+?):(\d+): (?:'.*?' : )?(.+)$/
const reVersion = /#version [\d]{3}/
const reInclude = /^(?:\s)*?(?:#include) "((?:\/?[^?<>:*|"]+?)+?\.(?:[a-zA-Z]+?))"$/
const include = '#extension GL_GOOGLE_include_directive : require'

const filters = [
  /(No code generated)/,
  /(compilation terminated)/,
  /Could not process include directive for header name:/
]

const files = new Map<string, number>()

export const ext = new Map([
  ['.fsh', 'frag'],
  ['.gsh', 'geom'],
  ['.vsh', 'vert'],
])

const tokens = new Map([
  ['SEMICOLON', ';'],
  ['COMMA', ','],
  ['COLON', ':'],
  ['EQUAL', '='],
  ['LEFT_PAREN', '('],
  ['RIGHT_PAREN', ')'],
  ['DOT', '.'],
  ['BANG', '!'],
  ['DASH', '-'],
  ['TILDE', '~'],
  ['PLUS', '+'],
  ['STAR', '*'],
  ['SLASH', '/'],
  ['PERCENT', '%'],
  ['LEFT_ANGEL', '<'],
  ['RIGHT_ANGEL', '>'],
  ['VERICAL_BAR', '|'],
  ['CARET', '^'],
  ['AMPERSAND', '&'],
  ['QUESTION', '?'],
  ['[LEFT_BRACKET', '['],
  ['RIGHT_BRACKET', ']'],
  ['LEFT_BRACE', '{'],
  ['RIGHT_BRACE', '}'],
])

// TODO exclude exts not in ext
export function preprocess(lines: string[], docURI: string, topLevel: boolean, incStack: string[]) {
  if (topLevel) {
    let inComment = false
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]
      //TODO something better than this
      if (line.includes('/*')) inComment = true
      if (line.includes('*/')) inComment = false
      if (line.trim().startsWith('//')) continue
      if (!inComment && reVersion.test(line)) {
        lines.splice(i + 1, 0, include)
        break
      }
      if (i === lines.length - 1) {
        lines.splice(0, 0, include)
        break
      }
    }
  }

  const includes = getIncludes(incStack[0], lines)
  let addedLines = 0
  if (includes.length > 0) {
    includes.forEach((inc, i) => {
      const incPath = absPath(docURI, inc.match[1])
      const dataLines = readFileSync(incPath).toString().split('\n')
      lines[inc.lineNum + addedLines + i] = `#line 0 "${incPath}"`
      lines.splice(inc.lineNum + addedLines + i, 0, ...dataLines)
      addedLines += dataLines.length
      lines.splice(inc.lineNum + addedLines + i, 0, `#line ${inc.lineNum} "${docURI}"`)
      preprocess(lines, incPath, false, incStack)
    })
  }

  if (!topLevel) return

  try {
    lint(docURI, lines, includes)
  } catch (e) {
  }
}

export const formatURI = (uri: string) => uri.replace(/^file:\/\//, '')

//TODO not include in comments
/* const getIncludes = (lines: string[])  => lines
    .map((line, i) => ({num: i, line}))
    .filter(obj => reInclude.test(obj.line))
    .map(obj => ({lineNum: obj.num, match: obj.line.match(reInclude)})) */

function getIncludes(uri: string, lines: string[]) {
  const out: {lineNum: number, parent: string, match: RegExpMatchArray}[] = []
  let count = [1] // for each file we need to track the line number
  let parStack = [uri] // for each include we need to track its parent
  lines.forEach(line => {
    count[count.length - 1]++
    const match = line.match(reInclude)
    if (line.startsWith('#line')) {
      parStack.push(line.slice(line.indexOf('"') + 1, line.lastIndexOf('"')))
    } else if (match.length === 0) return
  })
  return out
}

function absPath(currFile: string, includeFile: string): string {
  if (!currFile.startsWith(conf.shaderpacksPath)) {
    connection.window.showErrorMessage(`Shaderpacks path may not be correct. Current file is in ${currFile} but the path is set to ${conf.shaderpacksPath}`)
    return
  }

  // TODO add explanation comment
  if (includeFile.charAt(0) === '/') {
    const shaderPath = currFile.replace(conf.shaderpacksPath, '').split('/').slice(0, 3).join('/')
    return path.join(conf.shaderpacksPath, shaderPath, includeFile)
  }
  return path.join(path.dirname(currFile), includeFile)
}

function lint(uri: string, lines: string[], includes: {lineNum: number, match: RegExpMatchArray}[]) {
  console.log(lines.join('\n'))
  let out: string = ''
  try {
    execSync(`${conf.glslangPath} --stdin -S ${ext[path.extname(uri)]}`, {input: lines.join('\n')})
  } catch (e) {
    out = e.stdout.toString()
  }

  const diagnostics: {[uri: string]: Diagnostic[]} = {uri: []}
  includes.forEach(obj => {
    diagnostics[absPath(uri, obj.match[1])] = []
  })

  const matches = filterMatches(out)
  matches.forEach((match) => {
    const [whole, type, file, line, msg] = match
    const diag = {
      severity: type === 'ERROR' ? DiagnosticSeverity.Error : DiagnosticSeverity.Warning,
      range: calcRange(parseInt(line) - 1, uri),
      message: replaceWord(msg),
      source: 'mc-glsl'
    }
    diagnostics[file ? uri : file].push(diag)
  })

  daigsArray(diagnostics).forEach(d => connection.sendDiagnostics({uri: d.uri, diagnostics: d.diag}))
}

const replaceWord = (msg: string) => Object.entries(tokens).reduce((acc, [key, value]) => acc.replace(key, value), msg)

const daigsArray = (diags: {[uri: string]: Diagnostic[]}) => Object.keys(diags).map(uri => ({uri: 'file://' + uri, diag: diags[uri]}))

const filterMatches = (output: string) => output
  .split('\n')
  .filter(s => s.length > 1 && !filters.some(reg => reg.test(s)))
  .map(s => s.match(reDiag))
  .filter(match => match && match.length === 5)

function calcRange(lineNum: number, uri: string): Range {
  const lines = documents.get('file://' + uri).getText().split('\n')
  const line = lines[lineNum]
  const startOfLine = line.length - line.trimLeft().length
  const endOfLine = line.slice(0, line.indexOf('//')).trimRight().length + 1
  return Range.create(lineNum, startOfLine, lineNum, endOfLine)
}