import { inspect } from 'util'
import * as vscode from 'vscode'

// from rust-analyzer https://github.com/rust-analyzer/rust-analyzer/blob/ef223b9e6439c228e0be49861efd2067c0b22af4/editors/code/src/util.ts
export const log = new class {
  readonly output = vscode.window.createOutputChannel('Minecraft Shaders');

  // Hint: the type [T, ...T[]] means a non-empty array
  debug(...msg: [unknown, ...unknown[]]): void {
      log.write('DEBUG', ...msg)
  }

  info(...msg: [unknown, ...unknown[]]): void {
      log.write('INFO ', ...msg)
  }

  warn(...msg: [unknown, ...unknown[]]): void {
      log.write('WARN ', ...msg)
  }

  error(...msg: [unknown, ...unknown[]]): void {
      log.write('ERROR', ...msg)
  }

  write(label: string, ...messageParts: unknown[]): void {
      const message = messageParts.map(log.stringify).join(' ')
      const dateTime = new Date().toLocaleString()
      log.output.appendLine(`${label} [${dateTime}]: ${message}`)
  }

  private stringify(val: unknown): string {
      if (typeof val === 'string') return val
      return inspect(val, {
          colors: false,
          depth: 6, // heuristic
      })
  }
}

export const lspExceptionLogger = new class implements vscode.OutputChannel {
  name: string

  append(value: string): void {
    log.write('LSP-F', value)
  }

  appendLine(value: string): void {
    log.write('LSP-F', value)
  }
  
  clear(): void {
    log.output.clear()
  }
  
  show(column?: any, preserveFocus?: any) {
    log.output.show(column, preserveFocus)
  }

  hide(): void {
    log.output.hide()
  }

  dispose(): void {
    log.output.dispose()
  }
}
