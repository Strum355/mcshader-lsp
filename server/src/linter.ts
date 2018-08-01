import { Diagnostic, DiagnosticSeverity, Range } from 'vscode-languageserver'
import { connection, documents } from './server'
import { execSync } from 'child_process'
import * as path from 'path'
import { readFileSync, existsSync, statSync, Stats } from 'fs'
import { conf } from './config'
import { postError, formatURI, getDocumentContents } from './utils'
import { platform } from 'os'
import { Graph } from './graph'
import { Comment } from './comment'

const reDiag = /^(ERROR|WARNING): ([^?<>*|"]+?):(\d+): (?:'.*?' : )?(.+)\r?/
const reVersion = /#version [\d]{3}/
const reInclude = /^(?:\s)*?(?:#include) "(.+)"\r?/
const reIncludeExt = /#extension GL_GOOGLE_include_directive ?: ?require/
const include = '#extension GL_GOOGLE_include_directive : require'
const win = platform() === 'win32'

const filters = [
  /stdin/,
  /(No code generated)/,
  /(compilation terminated)/,
  /Could not process include directive for header name:/
]

export const includeGraph = new Graph()

export const allFiles = new Set<string>()

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

export function preprocess(lines: string[], docURI: string) {
  let hasDirective = true
  // wish there was an ignore keyword like Go
  if (lines.find((value: string, _, __): boolean => reIncludeExt.test(value)) == undefined) {
    hasDirective = false
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

  const allIncludes: IncludeObj[] = []
  const diagnostics = new Map<string, Diagnostic[]>()

  processIncludes(lines, [docURI], allIncludes, diagnostics, hasDirective)

  allIncludes.forEach(inc => allFiles.add(inc.path))

  const includeMap = new Map<string, IncludeObj>(allIncludes.map(obj => [obj.path, obj]) as [string, IncludeObj][])

  lint(docURI, lines, includeMap, diagnostics)
}

const buildIncludeGraph = (inc: IncludeObj) => includeGraph.setParent(inc.path, inc.parent)

function processIncludes(lines: string[], incStack: string[], allIncludes: IncludeObj[], diagnostics: Map<string, Diagnostic[]>, hasDirective: boolean) {
  const includes = getIncludes(incStack[0], lines)
  allIncludes.push(...includes)
  if (includes.length > 0) {
    includes.reverse().forEach(inc => {
      buildIncludeGraph(inc)
      mergeInclude(inc, lines, incStack, diagnostics, hasDirective)
    })
    // recursively check for more includes to be merged
    processIncludes(lines, incStack, allIncludes, diagnostics, hasDirective)
  }
}

// TODO no
export function getIncludes(uri: string, lines: string[]) {
  const count = [-1] // for each file we need to track the line number
  const parStack = [uri] // for each include we need to track its parent

  let total = -1 // current line number overall
  let comment = Comment.State.No

  return lines.reduce<IncludeObj[]>((out, line, i, l): IncludeObj[] => {
    const updated =  Comment.update(line, comment)
    comment = updated[0]
    line = updated[1]
    lines[i] = line

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
        path: formatURI(absPath(parStack[parStack.length - 1], match[1])),
        lineNum: count[count.length - 1],
        lineNumTopLevel: total,
        parent: formatURI(parStack[parStack.length - 1]),
        match
      })
    }
    return out
  }, [])
}

function ifInvalidFile(inc: IncludeObj, lines: string[], incStack: string[], diagnostics: Map<string, Diagnostic[]>) {
  const file = incStack[incStack.length - 1]
  diagnostics.set(
    inc.parent,
    [
      ...(diagnostics.get(inc.parent) || []),
      {
        severity: DiagnosticSeverity.Error,
        range: calcRange(inc.lineNum - (win ? 1 : 0), file),
        message: `${inc.path.replace(conf.shaderpacksPath, '')} is missing or an invalid file.`,
        source: 'mc-glsl'
      }
    ]
  )
  lines[inc.lineNumTopLevel] = ''
  // TODO fill in the actual data
  propogateDiagnostic(file, 'ERROR', inc.lineNum.toString(), `${inc.path.replace(conf.shaderpacksPath, '')} is missing or an invalid file.`, diagnostics, null)
}

function mergeInclude(inc: IncludeObj, lines: string[], incStack: string[], diagnostics: Map<string, Diagnostic[]>, hasDirective: boolean) {
  let stats: Stats
  try {
    stats = statSync(inc.path)
  } catch (e) {
    if (e.code === 'ENOENT') {
      ifInvalidFile(inc, lines, incStack, diagnostics)
      return
    }
    throw e
  }

  if (!stats.isFile()) {
    ifInvalidFile(inc, lines, incStack, diagnostics)
    return
  }

  const dataLines = readFileSync(inc.path).toString().split('\n')

  // if the includes parent is the top level (aka where the include directive is placed)
  // and we had to manually add the directive, - 1 the line number to account for the extra line
  if (inc.parent === incStack[0] && !hasDirective) inc.lineNum = inc.lineNum - 1

  incStack.push(inc.path)

  // TODO deal with the fact that includes may not be the sole text on a line
  // add #line indicating we are entering a new include block
  lines[inc.lineNumTopLevel] = `#line 0 "${formatURI(inc.path)}"`
  // merge the lines of the file into the current document
  // TODO do we wanna use a DLL here?
  lines.splice(inc.lineNumTopLevel + 1, 0, ...dataLines)
  // add the closing #line indicating we're re-entering a block a level up
  lines.splice(inc.lineNumTopLevel + 1 + dataLines.length, 0, `#line ${inc.lineNum + 1} "${inc.parent}"`)
}

function lint(docURI: string, lines: string[], includes: Map<string, IncludeObj>, diagnostics: Map<string, Diagnostic[]>) {
  console.log(lines.join('\n'))
  let out: string = ''
  try {
    execSync(`${conf.glslangPath} --stdin -S ${ext.get(path.extname(docURI))}`, {input: lines.join('\n')})
  } catch (e) {
    out = e.stdout.toString()
  }

  if (!diagnostics.has(docURI)) diagnostics.set(docURI, [])
  includes.forEach(obj => {
    if (!diagnostics.has(obj.path)) diagnostics.set(obj.path, [])
  })

  filterMatches(out).forEach((match) => {
    const [whole, type, file, line, msg] = match
    const diag: Diagnostic = {
      severity: errorType(type),
      // had to do - 2 here instead of - 1, windows only perhaps?
      range: calcRange(parseInt(line) - (win ? 2 : 1), file.length - 1 ? file : docURI),
      message: `Line ${line} ${replaceWords(msg)}`,
      source: 'mc-glsl'
    }

    diagnostics.get(file.length - 1 ? file : docURI).push(diag)

    // if is an include, highlight an error in the parents line of inclusion
    propogateDiagnostic(file, type, line, msg, diagnostics, includes)
  })

  daigsArray(diagnostics).forEach(d => {
    if (win) d.uri = d.uri.replace('file://C:', 'file:///c%3A')
    connection.sendDiagnostics({uri: d.uri, diagnostics: d.diag})
  })
}

function propogateDiagnostic(errorFile: string, type: string, line: string, msg: string, diagnostics: Map<string, Diagnostic[]>, includes: Map<string, IncludeObj>) {
  let nextFile = errorFile
  while (nextFile !== '0') {
    // TODO this doesnt deal with the fact that an include can have multiple parents :(
    const diag = {
      severity: errorType(type),
      range: calcRange(includes.get(nextFile).lineNum,  includes.get(nextFile).parent),
      message: `Line ${line} ${includes.get(errorFile).path.replace(conf.shaderpacksPath, '')} ${replaceWords(msg)}`,
      source: 'mc-glsl'
    }

    diagnostics.get(includes.get(nextFile).parent).push(diag)

    nextFile = includes.get(nextFile).parent
  }
}

export const replaceWords = (msg: string) => Array.from(tokens.entries()).reduce((acc, [key, value]) => acc.replace(key, value), msg)

const errorType = (error: string) => error === 'ERROR' ? DiagnosticSeverity.Error : DiagnosticSeverity.Warning

const daigsArray = (diags: Map<string, Diagnostic[]>) => Array.from(diags).map(kv => ({uri: 'file://' + kv[0], diag: kv[1]}))

const filterMatches = (output: string) => output
  .split('\n')
  .filter(s => s.length > 1 && !filters.some(reg => reg.test(s)))
  .map(s => s.match(reDiag))
  .filter(match => match && match.length === 5)

function calcRange(lineNum: number, uri: string): Range {
  const lines = getDocumentContents(uri).split('\n')
  const line = lines[lineNum]
  const startOfLine = line.length - line.trimLeft().length
  const endOfLine = line.trimRight().length + 1
  //const endOfLine = line.slice(0, line.indexOf('//')).trimRight().length + 2
  return Range.create(lineNum, startOfLine, lineNum, endOfLine)
}

export function absPath(currFile: string, includeFile: string): string {
  if (!currFile.startsWith(conf.shaderpacksPath) || conf.shaderpacksPath === '') {
    connection.window.showErrorMessage(`Shaderpacks path may not be correct. Current file is in '${currFile}' but the path is set to '${conf.shaderpacksPath}'`)
    return ''
  }

  // TODO add explanation comment
  if (includeFile.charAt(0) === '/') {
    const shaderPath = currFile.replace(conf.shaderpacksPath, '').split('/').slice(0, 3).join('/')
    return path.join(conf.shaderpacksPath, shaderPath, includeFile)
  }
  return path.join(path.dirname(currFile), includeFile)
}