import { connection, documents } from './server'
import { readFileSync } from 'fs'
import { conf } from './config'

export function postError(e: Error) {
    connection.window.showErrorMessage(e.message)
    console.log(e)
}

export const formatURI = (uri: string) => uri.replace(/^file:\/\//, '').replace(/^(?:\/)c%3A/, 'C:').replace(/\\/g, '/')

export function getDocumentContents(uri: string): string {
  if (documents.keys().includes('file://' + uri)) return documents.get('file://' + uri).getText()
  else return readFileSync(uri).toString()
}

export function trimPath(path: string): string {
  return path.replace(conf.shaderpacksPath, '')
}