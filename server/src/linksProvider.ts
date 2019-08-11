import * as vsclang from 'vscode-languageserver'
import { linkLog as log} from './logging'
import { formatURI } from './utils'

export function getDocumentLinks(file: string): vsclang.DocumentLink[] {
  log.debug(() => formatURI(file) + ' ' + file)
  return [vsclang.DocumentLink.create(vsclang.Range.create(8, 0, 8, 32), 'file:///e:\\shaderpacks\\Sushi-Shader\\shaders\\composite1.vsh')]
}