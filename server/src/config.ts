import { dirname } from 'path'
import { DidChangeConfigurationParams } from 'vscode-languageserver'
import { GLSLangProvider } from './glslangValidator'
import { configLog as log, loggers } from './logging'
import { connection } from './server'

const url = {
  'win32': 'https://github.com/KhronosGroup/glslang/releases/download/master-tot/glslang-master-windows-x64-Release.zip',
  'linux': 'https://github.com/KhronosGroup/glslang/releases/download/master-tot/glslang-master-linux-Release.zip',
  'darwin': 'https://github.com/KhronosGroup/glslang/releases/download/master-tot/glslang-master-osx-Release.zip'
}

export let glslangReady = false

// Maps the JSON settings from VSCode to an object
interface Config {
  shaderpacksPath: string
  glslangValidatorPath: string
  logLevel: 'error' | 'warn' | 'info' | 'debug'
}

export class ConfigProvider {
  private _config: Config
  private _glslang: GLSLangProvider

  public constructor() {
    this._config = {
      shaderpacksPath: '',
      glslangValidatorPath: '',
      logLevel: 'info'
    }
  }

  public set config(c: Config) {
    Object.assign(this._config, c)
  }

  public get config(): Config {
    return this._config
  }

  public onConfigChange = (change: DidChangeConfigurationParams) => {
    onConfigChange(this, change.settings.mcglsl as Config)
  }

  public set glslang(glslang: GLSLangProvider) {
    this._glslang = glslang
  }

  public get glslang(): GLSLangProvider {
    return this._glslang
  }
}

let supress = false

async function onConfigChange(confProv: ConfigProvider, current: Config) {
  if (!confProv.config == undefined &&
    current.shaderpacksPath === confProv.config.shaderpacksPath &&
    current.glslangValidatorPath === confProv.config.glslangValidatorPath &&
    current.logLevel === confProv.config.logLevel) return

    log.debug('new config: ' + JSON.stringify(current, Object.keys(current).sort()))
    log.debug('old config: ' + JSON.stringify(confProv.config || {}, Object.keys(confProv.config).sort()))
    confProv.config = {
      shaderpacksPath: current['shaderpacksPath'], 
      glslangValidatorPath: current['glslangValidatorPath'],
      logLevel: current['logLevel'],
    }

  // handle config.shaderpacksPath
  {
    if (confProv.config.shaderpacksPath === '' || confProv.config.shaderpacksPath.replace(dirname(confProv.config.shaderpacksPath), '') !== '/shaderpacks') {
      if (supress) return
  
      log.error(`shaderpack path '${confProv.config.shaderpacksPath.replace(dirname(confProv.config.shaderpacksPath), '')}' not set or doesn't end in 'shaderpacks'`)
      supress = true
  
      const clicked = await connection.window.showErrorMessage(
        'mcglsl.shaderpacksPath is not set or doesn\'t end in \'shaderpacks\'. Please set it in your settings.',
        {title: 'Supress'}
      )
      supress = (clicked && clicked.title === 'Supress') ? true : false
      return
    }
  }

  // handle config.logLevel
  {
    for(let logger of loggers) {
      logger.level = current.logLevel
    }
  }

  // handle config.glslang
  {
    if (!confProv.glslang.testExecutable()) {
      await confProv.glslang.promptDownload()
    } else {
      glslangReady = true
    }
  }
}
