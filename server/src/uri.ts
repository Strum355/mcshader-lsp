import { uriLog as log } from './logging'

export function formatURI(uri: string): string {
    const drive = uri[7]
    uri = uri.replace(`file:///${drive.toUpperCase()}%3A`, `file://${drive}:`)
    return uri.replace(/^file:\/\//, '').replace(/\\/, '/')
  }

export class URI {
    public static fromFileURI(uri: string): string {
      log.debug(`normalizing ${uri}`)
      if (URI.isNormalized(uri)) {
        log.debug(`already normalized ${uri}`)
        return uri
      }

      return ''
    }

    public static toFileURI(uri: string): string {
      let fileURI = uri

      if (!fileURI.startsWith('file:///')) {
        if (/^\\[a-zA-Z]/.test(fileURI)) {
          fileURI = 'file:///' + fileURI.substr(1)
        } else if (fileURI.startsWith('/')) {
          fileURI = 'file://' + fileURI
        }
      } else if (fileURI.startsWith('file://\\')) {
        fileURI = fileURI.replace('file://\\', 'file:///')
      } else if (/^file:\/\/[a-zA-Z]/.test(fileURI)) {
        fileURI = fileURI.replace('file://', 'file:///')
      }

      log.debug(`formatted '${uri}' to '${fileURI}'`)
      return fileURI
    }

    private static isNormalized(uri: string): boolean {
      if (uri.startsWith('file://') || uri.includes('%3A')) {
        return false
      }

      return true
    }
}
