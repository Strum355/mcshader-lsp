'use strict';

import * as vscode from 'vscode'
import GLSLProvider from './linter/glslProvider'
import * as shell from 'shelljs'

let glslProv: GLSLProvider;

export function activate(context: vscode.ExtensionContext) {
    glslProv = new GLSLProvider(context.subscriptions)
    vscode.languages.registerCodeActionsProvider('glsl', glslProv)
}

export function deactivate() {
    try {
        console.log('[MC-GLSL] disposing')
        shell.rm('-rf', glslProv.getConfig().tmpdir)
      } catch(e) {
        console.log(e)
      }
}