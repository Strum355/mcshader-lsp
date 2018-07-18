import { TextDocument, Diagnostic, DiagnosticSeverity, Range } from 'vscode-languageserver'
import { connection, documents } from './server'
import { execSync } from 'child_process'
import * as path from 'path'
import { readFileSync, existsSync } from 'fs'
import { conf } from './config'

const reDiag = /^(ERROR|WARNING): ([^?<>:*|"]+?):(\d+): (?:'.*?' : )?(.+)$/
const reVersion = /#version [\d]{3}/
const reInclude = /^(?:\s)*?(?:#include) "((?:\/?[^?<>:*|"]+?)+?\.(?:[a-zA-Z]+?))"$/
const reIncludeExt = /#extension GL_GOOGLE_include_directive ?: ?require/
const include = '#extension GL_GOOGLE_include_directive : require'

const filters = [
  /(No code generated)/,
  /(compilation terminated)/,
  /Could not process include directive for header name:/
]

const files = new Map<string, number>()

type IncludeObj = {
  lineNum: number,
  lineNumTopLevel: number,
  path: string,
  parent: string,
  match: RegExpMatchArray
}

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

export function isInComment(line: string, state: Comment): Comment {
  const indexOf = line.indexOf('#include')
  if (indexOf > -1 && line.indexOf('//') < indexOf) {
    return Comment.No
  }
  return Comment.No
}

export function preprocess(lines: string[], docURI: string) {
  if (lines.find((value: string, i: number, obj: string[]): boolean => reIncludeExt.test(value)) == undefined) {
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]
      if (reVersion.test(line)) {
        lines.splice(i + 1, 0, include)
        break
      }
      if (i === lines.length - 1) {
        lines.splice(0, 0, include)
        break
      }
    }
  }

  const incStack = [docURI]
  const allIncludes: IncludeObj[] = []
  processIncludes(lines, incStack, allIncludes)

  try {
    lint(docURI, lines, new Map<string, IncludeObj>(allIncludes.map(obj => [obj.path, obj]) as [string, IncludeObj][]))
  } catch (e) {
    console.log(e)
  }
}

function processIncludes(lines: string[], incStack: string[], allIncludes: IncludeObj[]) {
  const includes = getIncludes(incStack[0], lines)
  allIncludes.push(...includes)
  if (includes.length > 0) {
    includes.reverse().forEach(inc => {
      mergeInclude(inc, lines, incStack)
    })
    // recursively check for more includes to be merged
    processIncludes(lines, incStack, allIncludes)
  }
}

// TODO no
export function getIncludes(uri: string, lines: string[]) {
  const count = [-1] // for each file we need to track the line number
  const parStack = [uri] // for each include we need to track its parent

  let total = -1 // current line number overall
  let comment = Comment.No

  return lines.reduce<IncludeObj[]>((out: IncludeObj[], line: string, i, l): IncludeObj[] => {
    comment = isInComment(line, comment)
    count[count.length - 1]++
    total++
    if (comment) return out
    if (line.startsWith('#line')) {
      const inc = line.slice(line.indexOf('"') + 1, line.lastIndexOf('"'))

      if (inc === parStack[parStack.length - 2]) {
        count.pop()
        parStack.pop()
      } else {
        count.push(-1)
        parStack.push(inc)
      }
      return out
    }

    const match = line.match(reInclude)

    if (match) {
      out.push({
        path: absPath(parStack[parStack.length - 1], match[1]),
        lineNum: count[count.length - 1],
        lineNumTopLevel: total,
        parent: parStack[parStack.length - 1],
        match
      })
    }
    return out
  }, [])
}

function mergeInclude(inc: IncludeObj, lines: string[], incStack: string[]) {
  if (!existsSync(inc.path)) {
    const range = calcRange(inc.lineNumTopLevel, incStack[0])
    // TODO this needs to be aggregated and passed to the lint function, else theyre overwritten
    connection.sendDiagnostics({uri: 'file://' + inc.parent, diagnostics: [{
      severity: DiagnosticSeverity.Error,
      range,
      message: `${inc.path} is missing.`,
      source: 'mc-glsl'
    }]})
    lines[inc.lineNumTopLevel] = ''
    return
  }
  const dataLines = readFileSync(inc.path).toString().split('\n')
  incStack.push(inc.path)

  // TODO deal with the fact that includes may not be the sole text on a line
  // add #line indicating we are entering a new include block
  lines[inc.lineNumTopLevel] = `#line 0 "${inc.path}"`
  // merge the lines of the file into the current document
  lines.splice(inc.lineNumTopLevel + 1, 0, ...dataLines)
  // add the closing #line indicating we're re-entering a block a level up
  lines.splice(inc.lineNumTopLevel + 1 + dataLines.length, 0, `#line ${inc.lineNum} "${inc.parent}"`)
}

function lint(uri: string, lines: string[], includes: Map<string, IncludeObj>) {
  console.log(lines.join('\n'))
  //return
  let out: string = ''
  try {
    execSync(`${conf.glslangPath} --stdin -S ${ext.get(path.extname(uri))}`, {input: lines.join('\n')})
  } catch (e) {
    out = e.stdout.toString()
  }

  const diagnostics = new Map([[uri, Array<Diagnostic>()]])
  includes.forEach(obj => diagnostics.set(obj.path, []))

  filterMatches(out).forEach((match) => {
    let [whole, type, file, line, msg] = match

    let diag: Diagnostic = {
      severity: errorType(type),
      range: calcRange(parseInt(line) - 1, file.length - 1 ? file : uri),
      message: replaceWord(msg),
      source: 'mc-glsl'
    }

    diagnostics.get(file.length - 1 ? file : uri).push(diag)

    // if is an include, highlight an error in the parents line of inclusion
    while (file !== '0' && file !== uri) {
      // TODO what if we dont know the top level parent? Feel like thats a non-issue given that we have uri
      // TODO prefix error with filename
      diag = {
        severity: errorType(type),
        range: calcRange(includes.get(file).lineNum, includes.get(file).parent),
        message: replaceWord(msg),
        source: 'mc-glsl'
      }

      diagnostics.get(includes.get(file).parent).push(diag)

      file = includes.get(file).parent
    }
  })

  daigsArray(diagnostics).forEach(d => {
    connection.sendDiagnostics({uri: d.uri, diagnostics: d.diag})
  })
}

export const replaceWord = (msg: string) => Array.from(tokens.entries()).reduce((acc, [key, value]) => acc.replace(key, value), msg)

const errorType = (error: string) => error === 'ERROR' ? DiagnosticSeverity.Error : DiagnosticSeverity.Warning

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

export const formatURI = (uri: string) => uri.replace(/^file:\/\//, '')

export function absPath(currFile: string, includeFile: string): string {
  if (!currFile.startsWith(conf.shaderpacksPath) || conf.shaderpacksPath === '') {
    connection.window.showErrorMessage(`Shaderpacks path may not be correct. Current file is in '${currFile}' but the path is set to '${conf.shaderpacksPath}'`)
    return
  }

  // TODO add explanation comment
  if (includeFile.charAt(0) === '/') {
    const shaderPath = currFile.replace(conf.shaderpacksPath, '').split('/').slice(0, 3).join('/')
    return path.join(conf.shaderpacksPath, shaderPath, includeFile)
  }
  return path.join(path.dirname(currFile), includeFile)
}