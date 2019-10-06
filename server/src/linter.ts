import { Diagnostic } from 'vscode-languageserver'
import * as vsclangproto from 'vscode-languageserver-protocol'
import { ConfigProvider } from './config'
import { Graph } from './graph'
import { linterLog } from './logging'
import { URI } from './uri'

// global include tree
const includeTree = new Graph()

const include = '#extension GL_GOOGLE_include_directive : require'
const reDiag = /^(ERROR|WARNING): ([^?<>*|"]+?):(\d+): (?:'.*?' : )?(.+)\r?/
const reVersion = /#version [\d]{3}/
const reInclude = /^(?:\s)*?(?:#include) "(.+)"\r?/
const reIncludeExt = /#extension GL_GOOGLE_include_directive ?: ?require/

interface LinterState {
  hadIncludeDirective: boolean
  eventDocument: vsclangproto.TextDocument
  lines: string[]
  diagnostics: Map<String, Diagnostic>
  topLevelFiles: string[]
}

export class Linter {
  private state: LinterState = {
    hadIncludeDirective: true,
    eventDocument: undefined,
    diagnostics: new Map(),
    lines: [],
    topLevelFiles: [],
  }

  constructor(document: vsclangproto.TextDocument) {
    this.state.eventDocument = document
    this.state.lines = document.getText().split('\n')
  }

  public static do(document: vsclangproto.TextDocument, config: ConfigProvider) {
    const uri = URI.fromFileURI(document.uri)
    linterLog.info(`running linter for ${uri}`)

    const linter = new Linter(document)
    linter.gatherTopLevelFiles(uri, uri)

    linter.addIncludeDirective()

    linterLog.debug(`top level files for ${URI.trimShaderpacksPath(uri, config.config.shaderpacksPath)}: ` + JSON.stringify(linter.state.topLevelFiles))
  }

  private addIncludeDirective() {
    if (this.state.lines.findIndex(x => reIncludeExt.test(x)) > -1) {
      linterLog.debug('include directive found')
      this.state.hadIncludeDirective = true
      return 
    }
  
    let hasDirective = true
    linterLog.debug('include directive not found')
    for (let i = 0; i < this.state.lines.length; i++) {
      const line = this.state.lines[i]
      if (reVersion.test(line)) {
        linterLog.debug('found version on line ' + (i + 1))
        this.state.lines.splice(i + 1, 0, include)
        break
      }
    }
    return
  }
  

  private gatherTopLevelFiles(uri: string, original: string): boolean {
    if(includeTree.get(uri).parents.size == 0) {
      // we've hit a parent with no further parents, add to top level files
      if(uri != original) this.state.topLevelFiles.push(uri)
      return false
    }

    includeTree.get(uri).parents.forEach((parent, parentURI) => {
      this.gatherTopLevelFiles(parentURI, original)
    })
    return true
  }

  
}