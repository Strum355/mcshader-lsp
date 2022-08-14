import * as lsp from 'vscode-languageclient/node'

export type StatusParams = {
  status: 'loading' | 'ready' | 'failed' | 'clear'
  message: string
  icon: string
}

export const statusMethod = new lsp.NotificationType<StatusParams>('mc-glsl/status')

/* export const updateConfigMethod = 'mc-glsl/updateConfig'

export type ConfigUpdateParams = {
  kv: { key: string, value: string }[]
} */