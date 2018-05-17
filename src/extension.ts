'use strict';

import * as vscode from 'vscode'
import GLSLProvider from './linter/glslProvider'

export function activate(context: vscode.ExtensionContext) {
    vscode.languages.registerCodeActionsProvider('glsl', new GLSLProvider(context.subscriptions))
}