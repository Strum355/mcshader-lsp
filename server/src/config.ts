import { connection, documents, onEvent } from './server'
import fetch from 'node-fetch'
import { platform } from 'os'
import { createWriteStream, chmodSync, createReadStream, unlinkSync, read } from 'fs'
import * as unzip from 'unzip-stream'
import { postError } from './utils'
import { execSync } from 'child_process'
import { serverLog } from './logging'
import { dirname } from 'path'
import { DidChangeConfigurationParams } from 'vscode-languageserver/lib/main'
import { win } from './linter'

const url = {
  'win32': 'https://github.com/KhronosGroup/glslang/releases/download/master-tot/glslang-master-windows-x64-Release.zip',
  'linux': 'https://github.com/KhronosGroup/glslang/releases/download/master-tot/glslang-master-linux-Release.zip',
  'darwin': 'https://github.com/KhronosGroup/glslang/releases/download/master-tot/glslang-master-osx-Release.zip'
}

export let glslangReady = false

export interface Config {
  readonly shaderpacksPath: string
  readonly glslangPath: string
}

export let conf: Config = {shaderpacksPath: '', glslangPath: ''}

let supress = false

export async function onConfigChange(change: DidChangeConfigurationParams) {
  const temp = change.settings.mcglsl as Config
  if (temp.shaderpacksPath === conf.shaderpacksPath && temp.glslangPath === conf.glslangPath) return
  conf = {shaderpacksPath: temp['shaderpacksPath'].replace(/\\/g, '/'), glslangPath: temp['glslangValidatorPath'].replace(/\\/g, '/')}
  serverLog.debug(() => 'new config: ' + JSON.stringify(temp))
  serverLog.debug(() => 'old config: ' + JSON.stringify(conf))

  if (conf.shaderpacksPath === '' || conf.shaderpacksPath.replace(dirname(conf.shaderpacksPath), '') !== '/shaderpacks') {
    if (supress) return
    serverLog.error(() => 'shaderpack path not set or doesn\'t end in \'shaderpacks\'', null)
    supress = true
    const clicked = await connection.window.showErrorMessage(
      'mcglsl.shaderpacksPath is not set or doesn\'t end in \'shaderpacks\'. Please set it in your settings.',
      {title: 'Supress'}
    )
    supress = (clicked && clicked.title === 'Supress') ? true : false
    return
  }

  try {
    if (!execSync(conf.glslangPath).toString().startsWith('Usage')) {
      documents.all().forEach(onEvent)
      glslangReady = true
    } else {
      promptDownloadGlslang()
    }
  } catch (e) {
    if ((e.stdout.toString() as string).startsWith('Usage')) {
      documents.all().forEach(onEvent)
      glslangReady = true
    } else {
      promptDownloadGlslang()
    }
  }
}

async function promptDownloadGlslang() {
  const chosen = await connection.window.showErrorMessage(
    `[mc-glsl] glslangValidator not found at: '${conf.glslangPath}'.`,
    {title: 'Download'},
    {title: 'Cancel'}
  )

  if (!chosen || chosen.title !== 'Download') return

  downloadGlslang()
}

async function downloadGlslang() {
  connection.window.showInformationMessage('Downloading. Your settings will be updated automatically and you\'ll be notified when its done.')

  const res = await fetch(url[platform()])

  try {
    const zip = createWriteStream(conf.shaderpacksPath + '/glslangValidator.zip')
    res.body.pipe(zip)

    zip.on('finish', () => {
      createReadStream(conf.shaderpacksPath + '/glslangValidator.zip')
        .pipe(unzip.Parse())
        .on('entry', entry => {
          if (entry.path === 'bin/glslangValidator' + win ? '.exe' : '') {
            entry.pipe(createWriteStream(conf.shaderpacksPath + '/glslangValidator' + win ? '.exe' : ''))
            return
          }
          entry.autodrain()
        })
        .on('close', () => {
          chmodSync(conf.shaderpacksPath + '/glslangValidator' + win ? '.exe' : '', 0o775)
          unlinkSync(conf.shaderpacksPath + '/glslangValidator.zip')
          connection.sendNotification('update-config', conf.shaderpacksPath + '/glslangValidator' + win ? '.exe' : '')
          connection.window.showInformationMessage('glslangValidator has been downloaded to ' + conf.shaderpacksPath + '/glslangValidator. Your config should be updated automatically.')
          glslangReady = true
        })
    })
  } catch (e) {
    postError(e)
  }
}