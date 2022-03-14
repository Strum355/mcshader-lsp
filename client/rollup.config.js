import resolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import json from '@rollup/plugin-json';
import nodeBuiltins from 'builtin-modules';

/** @type { import('rollup').RollupOptions } */
export default {
    input: 'out/extension.js',
    plugins: [
        json(),
        resolve({
            preferBuiltins: true
        }),
        commonjs()
    ],
    external: [...nodeBuiltins, 'vscode'],
    output: {
        file: './out/extension.js',
        format: 'cjs'
    }
};