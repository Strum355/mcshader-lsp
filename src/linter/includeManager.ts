import * as vscode from 'vscode'
import * as fs from 'fs'
import GLSLProvider from './glslProvider';

export class IncludeManager {
  private readonly queue: null[] = []
  private readonly path: string;

  constructor(path: string) {
    this.path = path
  }

  public push() {
    this.queue.push(null)
    if (this.queue.length === 1) {
      this.mergeFiles()
    }
  }

  private async mergeFiles() {
    while (this.queue.length > 0) {
      // WIP
      /* this.queue.pop()
      let text = ''
      console.log('yes')
      await fs.readFile(GLSLProvider.getTempFilePath(this.path), (err: NodeJS.ErrnoException, data: Buffer) => {
        if (err) {
          resolve(err)
          console.log(err)
          return
        }
        text += data
      })
      console.log(text) */
    }
  }
}

export class IncludeHolder {
  public managers: { [file: string]: IncludeManager } = {}

  public add(file: vscode.Uri) {
    if (!this.managers.hasOwnProperty(file.path)) {
      this.managers[file.path] = new IncludeManager(file.path)
    }
  }

  public get = (file: vscode.Uri) => this.managers[file.path]
}