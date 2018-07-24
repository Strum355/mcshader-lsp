import { connection, documents, onEvent } from './server'
import fetch from 'node-fetch'
import { platform } from 'os'
import { createWriteStream, chmodSync, createReadStream, unlinkSync } from 'fs'
import * as unzip from 'unzip'
import { postError } from './utils'
import { execSync } from 'child_process'

const url = {
  'win32': 'https://github.com/KhronosGroup/glslang/releases/download/master-tot/glslang-master-windows-x64-Release.zip',
  'linux': 'https://github.com/KhronosGroup/glslang/releases/download/master-tot/glslang-master-linux-Release.zip',
  'darwin': 'https://github.com/KhronosGroup/glslang/releases/download/master-tot/glslang-master-osx-Release.zip'
}

export interface Config {
  readonly shaderpacksPath: string
  readonly glslangPath: string
}

export let conf: Config = {shaderpacksPath: '', glslangPath: ''}

export const onConfigChange = async (change) => {
  const temp = change.settings.mcglsl as Config
  conf = {shaderpacksPath: temp['shaderpacksPath'].replace(/\\/g, '/'), glslangPath: temp['glslangValidatorPath'].replace(/\\/g, '/')}

  if (!execSync(conf.glslangPath).toString().startsWith('Usage')) {
    documents.all().forEach(onEvent)
  } else {
    promptDownloadGlslang()
  }
}

async function promptDownloadGlslang() {
  const chosen = await connection.window.showErrorMessage(
    `[mc-glsl] glslangValidator not found at: '${conf.glslangPath}'.`,
    {title: 'Download'},
    {title: 'Cancel'}
  )

  if (!chosen || chosen.title !== 'Download') return

  if (conf.shaderpacksPath === '') {
    connection.window.showErrorMessage('Please set mcglsl.shaderpacksPath as this is where glslangValidator will be saved to.')
    return
  }

  downloadGlslang()
}

async function downloadGlslang() {
  const res = await fetch(url[platform()])

  try {
    const zip = createWriteStream(conf.shaderpacksPath + '/glslangValidator.zip')
    res.body.pipe(zip)

    zip.on('finish', async () => {
      createReadStream(conf.shaderpacksPath + '/glslangValidator.zip')
        .pipe(unzip.Parse())
        .on('entry', entry => {
          if (entry.path === 'bin/glslangValidator') {
            entry.pipe(createWriteStream(conf.shaderpacksPath + '/glslangValidator'))
            return
          }
          entry.autodrain()
        })
        .on('close', () => {
          chmodSync(conf.shaderpacksPath + '/glslangValidator', 0o775)
          unlinkSync(conf.shaderpacksPath + '/glslangValidator.zip')
          connection.sendNotification('update-config', conf.shaderpacksPath + '/glslangValidator')
          connection.window.showInformationMessage('glslangValidator has been downloaded to ' + conf.shaderpacksPath + '/glslangValidator')
        })
    })
  } catch (e) {
    postError(e)
  }
}