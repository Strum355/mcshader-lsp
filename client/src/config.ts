import * as vscode from 'vscode'
import { execSync } from 'child_process'
import * as vscodeLang from 'vscode-languageclient'

export class Config {
  public readonly shaderpacksPath: string
  public readonly glslangPath: string

  constructor(shaderpacksPath: string, glslangPath: string) {
    this.shaderpacksPath = shaderpacksPath
    this.glslangPath = glslangPath || 'glslangValidator'
  }
}

let conf = new Config('', '')

export async function configChangeHandler(langServer: vscodeLang.LanguageClient, event?: vscode.ConfigurationChangeEvent) {
  if (event && !event.affectsConfiguration('mcglsl')) return

  const temp = vscode.workspace.getConfiguration('mcglsl')
  conf = new Config(temp.get('shaderpacksPath'), temp.get('glslangValidatorPath'))

  try {
    execSync(conf.glslangPath)
    langServer.sendNotification(vscodeLang.DidChangeConfigurationNotification.type)
  } catch (e) {
    if (e.status !== 1) {
      const selected = await vscode.window.showErrorMessage(
        `[mc-glsl] glslangValidator not found at: '${conf.glslangPath}' or returned non-0 code`,
        'Download',
        'Cancel'
      )

      if (selected === 'Download') {
        //TODO can i use the python script?
      }
    }
  }
}