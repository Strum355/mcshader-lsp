import { ConfigProvider } from './config'
import { execSync } from 'child_process'
import { extensionMap, ShaderFileExtension } from './fileTypes'
import * as path from 'path'
import fetch from 'node-fetch';
import { platform } from 'os';
import * as unzip from 'adm-zip'
import { createWriteStream } from 'fs'
import { writeFileSync, fstat } from 'fs';
import { connection } from './server';
import { glslProviderLog as log } from './logging';


const url = {
  'win32': 'https://github.com/KhronosGroup/glslang/releases/download/master-tot/glslang-master-windows-x64-Release.zip',
  'linux': 'https://github.com/KhronosGroup/glslang/releases/download/master-tot/glslang-master-linux-Release.zip',
  'darwin': 'https://github.com/KhronosGroup/glslang/releases/download/master-tot/glslang-master-osx-Release.zip'
}

export class GLSLangProvider {
  private _config: ConfigProvider
  
  public constructor(c: ConfigProvider) {
    this._config = c
  }
  
  public lint(document: string, fileExtension: ShaderFileExtension): string {
    try {
      execSync(`${this._config.config.glslangValidatorPath} --stdin -S ${extensionMap.get(fileExtension)}`, {input: document})
    } catch (e) {
      return e.stdout.toString()
    }
  }

  public promptDownload = async () => {
    const chosen = await connection.window.showErrorMessage(
      `[mc-glsl] glslangValidator not found at: '${this._config.config.glslangValidatorPath}'.`,
      {title: 'Download'},
      {title: 'Cancel'}
    )
  
    if (!chosen || chosen.title !== 'Download') return

    await this.installExecutable()
  }

  public installExecutable = async () => {
    try {
      const glslangBin = '/glslangValidator' + (platform() === 'win32' ? '.exe' : '')
      const glslangPath = this._config.config.shaderpacksPath + glslangBin

      const response = await fetch(url[platform()])
      
      const zip = new unzip(await response.buffer())
      
      const bin = zip.readFile('bin' + glslangBin)
      
      writeFileSync(glslangPath, bin)

      if (!this.testExecutable()) {
        connection.window.showErrorMessage(
          'Unexpected error occurred. Please try again'
        )
        return
      }

      // why doesnt this work????????
      log.info(() => `successfully downloaded glslangValidator to ${glslangPath}`)
      connection.window.showInformationMessage(
        `glslangValidator has been downloaded to ${glslangPath}. Your config should be updated automatically.`
      )
      connection.sendNotification('update-config', glslangPath)
    } catch (e) {
      log.error(() => `failed downloading glslangValidator`, e)
      connection.window.showErrorMessage(
        `Failed to install glslangValidator: ${e}`
      )
    }
  }
  
  public testExecutable(glslangPath?: string): boolean {
    let success = false
    try {
      const stdout = execSync(glslangPath || this._config.config.glslangValidatorPath, {
        stdio: 'pipe',
      }).toString()
      success = stdout.startsWith('Usage')
    } catch (e) {
      success = (e.stdout.toString() as string).startsWith('Usage')
    }

    if (success) {
      log.info(() => `glslangValidator found at ${this._config.config.glslangValidatorPath}`) 
    } else {
      log.error(() => `glslangValidator not found at ${this._config.config.glslangValidatorPath}`, null)
    }

    return success
  }
}