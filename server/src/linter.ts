import { Diagnostic, DiagnosticSeverity, Range } from 'vscode-languageserver'
import { connection, documents } from './server'
import { execSync } from 'child_process'
import * as path from 'path'
import { readFileSync, existsSync } from 'fs'
import { conf } from './config'
import { postError, formatURI, getDocumentContents } from './utils'
import { platform } from 'os'

const reDiag = /^(ERROR|WARNING): ([^?<>*|"]+?):(\d+): (?:'.*?' : )?(.+)\r?/
const reVersion = /#version [\d]{3}/
const reInclude = /^(?:\s)*?(?:#include) "((?:\/?[^?<>:*|"]+?)+?\.(?:[a-zA-Z]+?))"\r?/
const reIncludeExt = /#extension GL_GOOGLE_include_directive ?: ?require/
const include = '#extension GL_GOOGLE_include_directive : require'
const win = platform() === 'win32'

const filters = [
  /stdin/,
  /(No code generated)/,
  /(compilation terminated)/,
  /Could not process include directive for header name:/
]

export const includeToParent = new Map<string, Set<string>>()

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
  // wish there was an ignore keyword like Go
  if (lines.find((value: string, _, __): boolean => reIncludeExt.test(value)) == undefined) {
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

  processIncludes(lines, [docURI], allIncludes, diagnostics)

  const includeMap = new Map<string, IncludeObj>(allIncludes.map(obj => [obj.path, obj]) as [string, IncludeObj][])

  lint(docURI, lines, includeMap, diagnostics)
}

function buildIncludeTree(inc: IncludeObj) {
  if (!includeToParent.has(inc.path)) includeToParent.set(inc.path, new Set([inc.parent]))
  else includeToParent.get(inc.path).add(inc.parent)
}

function processIncludes(lines: string[], incStack: string[], allIncludes: IncludeObj[], diagnostics: Map<string, Diagnostic[]>) {
  const includes = getIncludes(incStack[0], lines)
  allIncludes.push(...includes)
  if (includes.length > 0) {
    includes.reverse().forEach(inc => {
      buildIncludeTree(inc)
      mergeInclude(inc, lines, incStack, diagnostics)
    })
    // recursively check for more includes to be merged
    processIncludes(lines, incStack, allIncludes, diagnostics)
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

function mergeInclude(inc: IncludeObj, lines: string[], incStack: string[], diagnostics: Map<string, Diagnostic[]>) {
  if (!existsSync(inc.path)) {
    const range = calcRange(inc.lineNumTopLevel - (win ? 1 : 0), incStack[0])
    diagnostics.set(
      inc.parent,
      [
        ...(diagnostics.get(inc.parent) || []),
        {
          severity: DiagnosticSeverity.Error,
          range,
          message: `${inc.path.replace(conf.shaderpacksPath, '')} is missing.`,
          source: 'mc-glsl'
        }
      ]
    )
    lines[inc.lineNumTopLevel] = ''
    return
  }
  const dataLines = readFileSync(inc.path).toString().split('\n')
  incStack.push(inc.path)

  // TODO deal with the fact that includes may not be the sole text on a line
  // add #line indicating we are entering a new include block
  lines[inc.lineNumTopLevel] = `#line 0 "${formatURI(inc.path)}"`
  // merge the lines of the file into the current document
  // TODO do we wanna use a DLL here?
  lines.splice(inc.lineNumTopLevel + 1, 0, ...dataLines)
  // add the closing #line indicating we're re-entering a block a level up
  lines.splice(inc.lineNumTopLevel + 1 + dataLines.length, 0, `#line ${inc.lineNum} "${inc.parent}"`)
}

function lint(uri: string, lines: string[], includes: Map<string, IncludeObj>, diagnostics: Map<string, Diagnostic[]>) {
  console.log(lines.join('\n'))
  //return
  let out: string = ''
  try {
    execSync(`${conf.glslangPath} --stdin -S ${ext.get(path.extname(uri))}`, {input: lines.join('\n')})
  } catch (e) {
    out = e.stdout.toString()
  }

  if (!diagnostics.has(uri)) diagnostics.set(uri, [])
  includes.forEach(obj => {
    if (!diagnostics.has(obj.path)) diagnostics.set(obj.path, [])
  })

  filterMatches(out).forEach((match) => {
    const [whole, type, file, line, msg] = match
    let diag: Diagnostic = {
      severity: errorType(type),
      // had to do - 2 here instead of - 1, windows only perhaps?
      range: calcRange(parseInt(line) - (win ? 2 : 1), file.length - 1 ? file : uri),
      message: replaceWords(msg),
      source: 'mc-glsl'
    }

    diagnostics.get(file.length - 1 ? file : uri).push(diag)

    // if is an include, highlight an error in the parents line of inclusion
    let nextFile = file
    while (nextFile !== '0' && nextFile !== uri) {
      // TODO what if we dont know the top level parent? Feel like thats a non-issue given that we have uri
      diag = {
        severity: errorType(type),
        // TODO put -1 back when #11 is sorted
        range: calcRange(includes.get(nextFile).lineNum,  includes.get(nextFile).parent),
        message: includes.get(file).path.replace(conf.shaderpacksPath, '') + replaceWords(msg),
        source: 'mc-glsl'
      }

      diagnostics.get(includes.get(nextFile).parent).push(diag)

      nextFile = includes.get(nextFile).parent
    }
  })

  daigsArray(diagnostics).forEach(d => {
    if (win) d.uri = d.uri.replace('file://C:', 'file:///c%3A')
    connection.sendDiagnostics({uri: d.uri, diagnostics: d.diag})
  })
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
  // TODO better error handling maybe?
  const lines = getDocumentContents(uri).split('\n')
  console.log(uri, lineNum, lines.length)
  console.log(lines.slice(Math.max(lineNum - 3, 0), lineNum + 3).join('\n'))

  const line = lines[lineNum]
  const startOfLine = line.length - line.trimLeft().length
  const endOfLine = line.slice(0, line.indexOf('//')).trimRight().length + 1
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