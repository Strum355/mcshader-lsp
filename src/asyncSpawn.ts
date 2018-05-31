import { spawn } from 'child_process'

//
export function runLinter(command: string, ...args: any[]) {
  const child = spawn(command, ...args)
  let stderr = ''

  child.stderr.on('data', data => {
    stderr += data
  })

  const promise = new Promise<string>((resolve, reject) => {
    child.on('error', () => reject(new Error('fatal error ${stderr}')))

    child.on('exit', (code, signal) => {
      switch (code) {
        case 0:
        case 2:
          resolve(stderr)
        default:
          reject(new Error('standard error ${signal} ${stderr}'))
      }
    })
  })

  return promise
}