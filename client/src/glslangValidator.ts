import * as unzip from 'adm-zip'
import { execSync } from 'child_process'
import { writeFileSync } from 'fs'
import fetch from 'node-fetch'
import { platform } from 'os'
import * as vscode from 'vscode'
import { clearStatus, glslConfigParam, outputChannel, updateStatus } from './extension'

const url = {
  'win32': 'https://github.com/KhronosGroup/glslang/releases/download/master-tot/glslang-master-windows-x64-Release.zip',
  'linux': 'https://github.com/KhronosGroup/glslang/releases/download/master-tot/glslang-master-linux-Release.zip',
  'darwin': 'https://github.com/KhronosGroup/glslang/releases/download/master-tot/glslang-master-osx-Release.zip'
}

const config = vscode.workspace.getConfiguration()

export async function promptDownload() {
  const chosen = await vscode.window.showErrorMessage(
    `[mc-glsl] glslangValidator not found at: '${config.get(glslConfigParam)}'.`,
    {title: 'Download'},
    {title: 'Cancel'}
  )

  if (!chosen || chosen.title !== 'Download') return

  await installExecutable()
}

async function installExecutable() {
  try {
    updateStatus('$(cloud-download)', 'Downloading glslangValidator')

    const glslangBin = '/glslangValidator' + (platform() === 'win32' ? '.exe' : '')
    const glslangPath = config.get('mcglsl.shaderpacksPath') + glslangBin

    const response = await fetch(url[platform()])
    outputChannel.appendLine('glslangValidator download response status: ' + response.status )

    const zip = new unzip(await response.buffer())

    const bin = zip.readFile('bin' + glslangBin)
    outputChannel.appendLine('buffer length ' + bin.length)
    writeFileSync(glslangPath, bin, {encoding: null, mode: 0o755})

    // Make sure download was successful
    if (!testExecutable(glslangPath)) {
      vscode.window.showErrorMessage(`Unexpected error occurred checking for binary at ${glslangPath}. Please try again`)
      clearStatus()
      throw new Error('failed to install glslangValidator')
    }

    // All done!
    outputChannel.appendLine(`successfully downloaded glslangValidator to ${glslangPath}`)
    vscode.window.showInformationMessage(
      `glslangValidator has been downloaded to ${glslangPath}. Your config should be updated automatically.`
    )
    config.update('mcglsl.glslangValidatorPath', glslangPath, vscode.ConfigurationTarget.Global)
    clearStatus()
  } catch (e) {
    outputChannel.appendLine(`failed downloading glslangValidator ${e}`)
    vscode.window.showErrorMessage(`Failed to install glslangValidator: ${e}`)
    clearStatus()
    throw e
  }

}

export function testExecutable(glslangPath?: string): boolean {
  glslangPath = glslangPath || config.get(glslConfigParam)
  let stdout = ''
  try {
    stdout = execSync(glslangPath, {
      stdio: 'pipe',
    }).toString()
  } catch (e) {
    stdout = (e.stdout.toString() as string)
  }

  outputChannel.appendLine('glslangValidator first line stdout: "' + stdout.split('\n')[0] + '"')
  const success = stdout.startsWith('Usage')

  if (success) {
    outputChannel.appendLine(`glslangValidator found at ${glslangPath}`)
  } else {
    outputChannel.appendLine(`glslangValidator not found at ${glslangPath}`)
  }

  return success
}