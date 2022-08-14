import { inspect } from 'util'
import * as vscode from 'vscode'

export const lspOutputChannel = vscode.window.createOutputChannel('Minecraft Shaders LSP - Server')
export const traceOutputChannel = vscode.window.createOutputChannel('Minecraft Shaders LSP - Trace')

// from rust-analyzer https://github.com/rust-analyzer/rust-analyzer/blob/ef223b9e6439c228e0be49861efd2067c0b22af4/editors/code/src/util.ts
export const log = new class {
  readonly output = vscode.window.createOutputChannel('Minecraft Shaders LSP - Client');

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

