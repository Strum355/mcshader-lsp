import * as lsp from 'vscode-languageclient'

export type StatusParams = {
  status: 'loading' | 'ready' | 'failed' | 'clear'
  message: string
  icon: string
}

export const statusMethod = 'mc-glsl/status'
export const status = new lsp.NotificationType<StatusParams>(statusMethod)

export const updateConfigMethod = 'mc-glsl/updateConfig'

export type ConfigUpdateParams = {
  kv: { key: string, value: string }[]
}