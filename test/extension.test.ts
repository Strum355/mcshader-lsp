import * as assert from 'assert';
import * as vscode from 'vscode'
import * as provider from '../src/linter/glslProvider'
import * as fs from 'fs'
import * as shell from 'shelljs'

suite('Extension Tests', () => {
    const inst = new provider.default([], {
        glslangPath: 'glslangValidator',
        tmpdir: '',
        isWin: false
    })

    /* test('Check for binary', () => {
        inst.checkBinary()
        shell.which(inst.)
    }) */

    test('Check filters', () => {
        const strings = [
            'ERROR: 0:61: \'#include\' : required extension not requested: GL_GOOGLE_include_directive',
            'ERROR: 0:10: \'\' :  syntax error',
            'ERROR: 0:23: \'owo\' compilation terminated',
            'WARNING: 4:20: \'xd\' no code generated'
        ]

        strings.forEach((s: string) => {
            assert.equal(true, inst.matchesFilters(s))
        })
    })
});