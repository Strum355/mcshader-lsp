import { connection } from './server'
import { serverLog as log } from './logging'
import { dirname } from 'path'
import { DidChangeConfigurationParams } from 'vscode-languageserver'
import { GLSLangProvider } from './glslangValidator'

const url = {
  'win32': 'https://github.com/KhronosGroup/glslang/releases/download/master-tot/glslang-master-windows-x64-Release.zip',
  'linux': 'https://github.com/KhronosGroup/glslang/releases/download/master-tot/glslang-master-linux-Release.zip',
  'darwin': 'https://github.com/KhronosGroup/glslang/releases/download/master-tot/glslang-master-osx-Release.zip'
}

export let glslangReady = false

export class ConfigProvider {
  private _config: Config
  private _onChange: (settings: Config) => void
  private _glslang: GLSLangProvider

  public constructor(func?: (confProv: ConfigProvider, settings: Config) => void) {
    this._config = {
      shaderpacksPath: '',
      glslangValidatorPath: ''
    }

    if (!func) {
      this._onChange = (settings: Config) => {
        onConfigChange(this, settings)
      }
    } else {
      this._onChange = (settings: Config) => {
        func(this, settings)
      }
    }
  }

  public set config(c: Config) {
    Object.assign(this._config, c)
  }

  public get config(): Config {
    return this._config
  }

  public set onChange(func: (confProv: ConfigProvider, settings: Config) => void) {
    this._onChange = (settings: Config) => {
      func(this, settings)
    }
  }

  public onConfigChange = (change: DidChangeConfigurationParams) => {
    this._onChange(change.settings.mcglsl as Config)
  }

  public set glslang(glslang: GLSLangProvider) {
    this._glslang = glslang
  }

  public get glslang(): GLSLangProvider {
    return this._glslang
  }
}

interface Config {
  shaderpacksPath: string
  glslangValidatorPath: string
}

let supress = false

async function onConfigChange(confProv: ConfigProvider, old: Config) {
  if (!confProv.config == undefined && 
    old.shaderpacksPath === confProv.config.shaderpacksPath && 
    old.glslangValidatorPath === confProv.config.glslangValidatorPath) return

  confProv.config = {shaderpacksPath: old['shaderpacksPath'], glslangValidatorPath: old['glslangValidatorPath']}
  log.debug(() => 'new config: ' + JSON.stringify(old))
  log.debug(() => 'old config: ' + JSON.stringify(confProv.config))

  if (confProv.config.shaderpacksPath === '' || confProv.config.shaderpacksPath.replace(dirname(confProv.config.shaderpacksPath), '') !== '/shaderpacks') {
    if (supress) return

    log.error(() => `shaderpack path '${confProv.config.shaderpacksPath.replace(dirname(confProv.config.shaderpacksPath), '')}' not set or doesn't end in 'shaderpacks'`, null)
    supress = true

    const clicked = await connection.window.showErrorMessage(
      'mcglsl.shaderpacksPath is not set or doesn\'t end in \'shaderpacks\'. Please set it in your settings.',
      {title: 'Supress'}
    )
    supress = (clicked && clicked.title === 'Supress') ? true : false
    return
  }

  if (!confProv.glslang.testExecutable()) {
    await confProv.glslang.promptDownload()
  } else {
    glslangReady = true
  }
}
