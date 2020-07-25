import * as unzip from 'adm-zip'
import { execSync } from 'child_process'
import * as fs from 'fs'
import { writeFileSync } from 'fs'
import fetch from 'node-fetch'
import { platform } from 'os'
import * as vscode from 'vscode'
import { Extension } from './extension'
import { log } from './log'

const url = {
  'win32': 'https://github.com/KhronosGroup/glslang/releases/download/master-tot/glslang-master-windows-x64-Release.zip',
  'linux': 'https://github.com/KhronosGroup/glslang/releases/download/master-tot/glslang-master-linux-Release.zip',
  'darwin': 'https://github.com/KhronosGroup/glslang/releases/download/master-tot/glslang-master-osx-Release.zip'
}

const config = vscode.workspace.getConfiguration()

export async function bootstrapGLSLangValidator(e: Extension): Promise<boolean> {
  const glslangValidatorPath = config.get('mcglsl.glslangValidatorPath') as string
  if (!testExecutable(glslangValidatorPath)) {
    if(!await promptDownload(e, glslangValidatorPath)) return false
  }
  return true
}

async function promptDownload(e: Extension, glslangValidatorPath: string): Promise<boolean> {
  const chosen = await vscode.window.showErrorMessage(
    `[mc-glsl] glslangValidator not found at: '${glslangValidatorPath}'.`,
    {title: 'Download'},
    {title: 'Cancel'}
  )

  if (!chosen || chosen.title !== 'Download') return false

  return await tryInstallExecutable(e)
}

export async function tryInstallExecutable(e: Extension): Promise<boolean> {
  try {
    await installExecutable(e)
  } catch (e) {
    log.error(`failed downloading glslangValidator ${e}`)
    vscode.window.showErrorMessage(`Failed to install glslangValidator: ${e}`)
    e.clearStatus()
    return false
  }
  return true
}

async function installExecutable(e: Extension) {
  fs.mkdirSync(e.context.globalStoragePath, { recursive: true })
  
  e.updateStatus('$(cloud-download)', 'Downloading glslangValidator')

  const glslangBin = '/glslangValidator' + (platform() === 'win32' ? '.exe' : '')
  const glslangPath = e.context.globalStoragePath + glslangBin

  const response = await fetch(url[platform()])
  log.info('glslangValidator download response status: ' + response.status)

  const zip = new unzip(await response.buffer())

  const bin = zip.readFile('bin' + glslangBin)
  log.info('buffer length ' + bin.length)
  writeFileSync(glslangPath, bin, {encoding: null, mode: 0o755})

  // Make sure download was successful
  if (!testExecutable(glslangPath)) {
    throw new Error(`Unexpected error occurred checking for binary at ${glslangPath}. Please try again`)
  }

  // All done!
  log.info(`successfully downloaded glslangValidator to ${glslangPath}`)
  vscode.window.showInformationMessage(
    `glslangValidator has been downloaded to ${glslangPath}. Your config should be updated automatically.`
  )
  await config.update('mcglsl.glslangValidatorPath', glslangPath, vscode.ConfigurationTarget.Global)
  e.clearStatus()
}

function testExecutable(glslangPath: string): boolean {
  let stdout = ''
  try {
    stdout = execSync(glslangPath, {
      stdio: 'pipe',
    }).toString()
  } catch (e) {
    stdout = (e.stdout.toString() as string)
  }

  log.info('glslangValidator first line stdout: "' + stdout.slice(0, stdout.indexOf('\n')) + '"')
  const success = stdout.startsWith('Usage')

  if (success) {
    log.info(`glslangValidator found at ${glslangPath}`)
  } else {
    log.warn(`glslangValidator not found at ${glslangPath}`)
  }

  return success
}