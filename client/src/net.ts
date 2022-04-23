import { log } from './log'
import fetch from 'node-fetch'
import * as vscode from 'vscode'
import * as stream from 'stream'
import * as fs from 'fs'
import * as util from 'util'

const pipeline = util.promisify(stream.pipeline)

interface GithubRelease {
  tag_name: string;
  assets: Array<{
    name: string;
    browser_download_url: string;
  }>;
}

export async function getReleaseInfo(releaseTag: string): Promise<GithubRelease> {
  log.info('fetching release info for tag', releaseTag)
  const response = await fetch(`https://api.github.com/repos/strum355/mcshader-lsp/releases/tags/${releaseTag}`, {
    headers: { Accept: 'application/vnd.github.v3+json' }
  })

  const isRelease = (obj: unknown): obj is GithubRelease => {
    return obj != null && typeof obj === 'object'
      && typeof (obj as GithubRelease).tag_name === 'string'
      && Array.isArray((obj as GithubRelease).assets)
      && (obj as GithubRelease).assets.every((a) => typeof a.name === 'string' && typeof a.browser_download_url === 'string')
  }

  const json = await response.json()
  if (!isRelease(json)) {
    throw new TypeError(`Received malformed request from Github Release API ${JSON.stringify(json)}`)
  }
  return json
}

export async function download(url: string, downloadDest: string) {
  await vscode.window.withProgress(
    {
      location: vscode.ProgressLocation.Notification,
      cancellable: false,
      title: `Downloading ${url}`
    },
    async (progress, _) => {
      let lastPercentage = 0
      await downloadFile(url, downloadDest, (readBytes, totalBytes) => {
        const newPercentage = Math.round((readBytes / totalBytes) * 100)
        if (newPercentage !== lastPercentage) {
          progress.report({
            message: `${newPercentage.toFixed(0)}%`,
            increment: newPercentage - lastPercentage
          })

          lastPercentage = newPercentage
        }
      })
    }
  )
}

async function downloadFile(
  url: string,
  destFilePath: fs.PathLike,
  onProgress: (readBytes: number, totalBytes: number) => void
): Promise<void> {
  const res = await fetch(url)
  if (!res.ok) {
    log.error(res.status, 'while downloading file from', url)
    log.error({ body: await res.text(), headers: res.headers })
    throw new Error(`Got response ${res.status} when trying to download ${url}.`)
  }

  const totalBytes = Number(res.headers.get('content-length'))

  log.debug('downloading file with', totalBytes, 'bytes size from', url, 'to', destFilePath)

  let readBytes = 0
  res.body.on('data', (chunk: Buffer) => {
    readBytes += chunk.length
    onProgress(readBytes, totalBytes)
  })

  const destFileStream = fs.createWriteStream(destFilePath, { mode: 0o755 })

  await pipeline(res.body, destFileStream)

  // Don't apply the workaround in fixed versions of nodejs, since the process
  // freezes on them, the process waits for no-longer emitted `close` event.
  // The fix was applied in commit 7eed9d6bcc in v13.11.0
  // See the nodejs changelog:
  // https://github.com/nodejs/node/blob/master/doc/changelogs/CHANGELOG_V13.md
  const [, major, minor] = /v(\d+)\.(\d+)\.(\d+)/.exec(process.version)!
  if (+major > 13 || (+major === 13 && +minor >= 11)) return

  await new Promise<void>(resolve => {
    destFileStream.on('close', resolve)
    destFileStream.destroy()
    // This workaround is awaiting to be removed when vscode moves to newer nodejs version:
    // https://github.com/rust-analyzer/rust-analyzer/issues/3167
  })
}