import * as fs from 'fs'
import * as vscode from 'vscode'
import GLSLProvider from './linter/glslProvider'

export class DescriptorHolder {
  private holder: {[path: string]: number} = {}

  public add(path: vscode.Uri) {
    fs.open(GLSLProvider.getTempFilePath(path.path), 'r', (err, fd) => {
      this.holder[path.path] = fd
    })
  }

  public clear = () => {
    for (const path in this.holder) {
      if (this.holder.hasOwnProperty(path)) {
        fs.close(this.holder[path])
        delete this.holder[path]
      }
    }
  }
}