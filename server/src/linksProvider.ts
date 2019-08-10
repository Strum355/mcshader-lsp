import * as vsclang from 'vscode-languageserver'
import { linkLog } from './logging'
import { formatURI } from './utils'

export function getDocumentLinks(file: string): vsclang.DocumentLink[] {
  linkLog.debug(() => formatURI(file) + ' ' + file)
  return [vsclang.DocumentLink.create(vsclang.Range.create(8, 0, 8, 32), 'file:///home/noah/.minecraft/shaderpacks/robobo1221Shaders-7.9.01/shaders/lib/utilities.glsl')]
}