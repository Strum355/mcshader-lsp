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

enum Comment {
  No = 0,
  Single,
  Multi
}

export function preprocess(lines: string[], docURI: string) {
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

  const incStack = [docURI]
  processIncludes(lines, incStack)

  try {
    lint(docURI, lines, incStack)
  } catch (e) {
    console.log(e)
  }
}

function processIncludes(lines: string[], incStack: string[]) {
  const includes = getIncludes(incStack[0], lines)
  if (includes.length > 0) {
    includes.reverse().forEach(inc => {
      mergeInclude(inc, lines, incStack)
    })
    // recursively check for more includes to be merged
    processIncludes(lines, incStack)
  }
}

function mergeInclude(inc: {lineNum: number, lineNumParent: number, parent: string, match: RegExpMatchArray}, lines: string[], incStack: string[]) {
  const incPath = absPath(inc.parent, inc.match[1])
  const dataLines = readFileSync(incPath).toString().split('\n')
  incStack.push(incPath)

  // add #line indicating we are entering a new include block
  lines[inc.lineNumParent] = `#line 0 "${incPath}"`
  // merge the lines of the file into the current document
  lines.splice(inc.lineNumParent + 1, 0, ...dataLines)
  // add the closing #line indicating we're re-entering a block a level up
  lines.splice(inc.lineNumParent + 1 + dataLines.length, 0, `#line ${inc.lineNum} "${inc.parent}"`)
}

export const formatURI = (uri: string) => uri.replace(/^file:\/\//, '')

// TODO no
export function getIncludes(uri: string, lines: string[]) {
  const out: {lineNum: number, lineNumParent: number, parent: string, match: RegExpMatchArray}[] = []

  const count = [0] // for each file we need to track the line number
  const parStack = [uri] // for each include we need to track its parent

  let total = 0 // current line number overall
  let comment = Comment.No

  lines.forEach(line => {
    comment = isInComment(line, comment)
    if (!comment) {
      const match = line.match(reInclude)

      if (line.startsWith('#line')) {
        const inc = line.slice(line.indexOf('"') + 1, line.lastIndexOf('"'))

        if (inc === parStack[parStack.length - 2]) {
          count.pop()
          parStack.pop()
        } else {
          count.push(0)
          parStack.push(inc)
        }
      } else if (match) {
        out.push({
          lineNum: count[count.length - 1],
          lineNumParent: total,
          parent: parStack[parStack.length - 1],
          match
        })
      }
    }

    count[count.length - 1]++
    total++
  })
  return out
}

export function isInComment(line: string, state: Comment): Comment {
  const indexOf = line.indexOf('#include')
  if (indexOf > -1 && line.indexOf('//') < indexOf) {
    return Comment.No
  }
  return Comment.No
}

export function absPath(currFile: string, includeFile: string): string {
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

function lint(uri: string, lines: string[], includes: string[]) {
  //console.log(lines.join('\n'))
  //return
  let out: string = ''
  try {
    execSync(`${conf.glslangPath} --stdin -S ${ext.get(path.extname(uri))}`, {input: lines.join('\n')})
  } catch (e) {
    out = e.stdout.toString()
  }

  const diagnostics = new Map([[uri, Array<Diagnostic>()]])
  includes.forEach(obj => diagnostics.set(obj, []))

  const matches = filterMatches(out)
  matches.forEach((match) => {
    const [whole, type, file, line, msg] = match

    const diag: Diagnostic = {
      severity: type === 'ERROR' ? DiagnosticSeverity.Error : DiagnosticSeverity.Warning,
      range: calcRange(parseInt(line) - 1, file.length - 1 ? file : uri),
      message: replaceWord(msg),
      source: 'mc-glsl'
    }

    diagnostics.get(file.length - 1 ? file : uri).push(diag)
  })

  daigsArray(diagnostics).forEach(d => {
    connection.sendDiagnostics({uri: d.uri, diagnostics: d.diag})
  })
}

export const replaceWord = (msg: string) => Object.entries(tokens).reduce((acc, [key, value]) => acc.replace(key, value), msg)

const daigsArray = (diags: Map<string, Diagnostic[]>) => Array.from(diags).map(kv => ({uri: 'file://' + kv[0], diag: kv[1]}))

const filterMatches = (output: string) => output
  .split('\n')
  .filter(s => s.length > 1 && !filters.some(reg => reg.test(s)))
  .map(s => s.match(reDiag))
  .filter(match => match && match.length === 5)

function calcRange(lineNum: number, uri: string): Range {
  let lines = []

  // TODO better error handling maybe?
  if (documents.keys().includes('file://' + uri)) {
    lines = documents.get('file://' + uri).getText().split('\n')
  } else {
    lines = readFileSync(uri).toString().split('\n')
  }

  const line = lines[lineNum]
  const startOfLine = line.length - line.trimLeft().length
  const endOfLine = line.slice(0, line.indexOf('//')).trimRight().length + 1
  return Range.create(lineNum, startOfLine, lineNum, endOfLine)
}