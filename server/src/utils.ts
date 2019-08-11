import { readFileSync, FSWatcher, ReadStream } from 'fs'
import * as urL from 'url'
import { serverLog as log } from './logging'
import { connection, documents } from './server'
import { url } from 'inspector';
import { Readable } from 'stream';

export function postError(e: Error) {
    connection.window.showErrorMessage(e.message)
    log.error(() => e.message, null)
}

export function formatURI(uri: string): string {
  const drive = uri[7]
  uri = uri.replace(`file:///${drive.toUpperCase()}%3A`, `file://${drive}:`)
  return uri.replace(/^file:\/\//, '').replace(/\\/, '/')
}

export function getDocumentContents(uri: string): string {
  if (documents.keys().includes('file://' + uri)) return documents.get('file://' + uri).getText()
  else return readFileSync(uri).toString()
}

/* export function trimPath(path: string): string {
  return path.replace(conf.shaderpacksPath, '')
} */