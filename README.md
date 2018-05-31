# vscode-mc-shader

[![CI](https://ci.netsoc.co/api/badges/Strum355/vscode-mc-shader/status.svg?branch=master)](https://ci.netsoc.co/Strum355/vscode-mc-shader)
[![Issues](https://img.shields.io/github/issues-raw/Strum355/vscode-mc-shader.svg)](https://github.com/Strum355/vscode-mc-shader/issues)
[![license](https://img.shields.io/github/license/Strum355/vscode-mc-shader.svg)](https://github.com/Strum355/vscode-mc-shader)
[![Maintainability](https://api.codeclimate.com/v1/badges/c2c813cb0a42a8aad483/maintainability)](https://codeclimate.com/github/Strum355/vscode-mc-shader/maintainability)

This is an extension for [Visual Studio Code](https://code.visualstudio.com/) for developing Minecraft GLSL Shaders for [Optifine](http://optifine.net). It currently provides linting and syntax highlighting (by stef-levesque/vscode-shader dependency).

## Features

- Linting (unpolished)
- Syntax highlighting (by extension dependency)

## Planned

- Support for `#includes`
- Warnings for unused uniforms/varyings
- Some cool `DRAWBUFFERS` stuff
- Auto-complete prompts

Got a feature request? Chuck it into an Issue!

## Requirements

- Visual Studio Code (v1.17.0 or higher - untested)
- The [Shader languages support for VS Code](https://marketplace.visualstudio.com/items?itemName=slevesque.shader) extension. This should automatically install when you install this extension
- That the shader you're editing is in the `shaderpacks` folder in `.minecraft`.
- The [OpenGL / OpenGL ES Reference Compiler](https://cvs.khronos.org/svn/repos/ogl/trunk/ecosystem/public/sdk/tools/glslang/Install/) (for convenience, put it in your PATH, this is the assumed location if not specified). If, for some reason, you're using MacOS, there are no pre-compiled binaries of this.
- [Windows] An up to date version of Windows with Developer mode enabled for symlink support. (May not always work, I've gotten inconsistent results)
- [MacOS] Not MacOS. Not that you're making MC Shaders on/for MacOS anyways...right?

## Extension Settings

| Option Name | Data Type | Description | Default Value |
| ----------- | --------- | ----------- | ------------- |
| `mcglsl.glslangValidatorPath` | string |  The path to the glslangValidator executable. | In your `PATH`.|
| `mcglsl.lintOnType` | bool | Whether or not to lint while typing. Can decrease performance. | `false` |
| `mcglsl.minecraftPath` | string | The path to your Minecraft installation. | None |

## Contributing

Please see [CONTRIBUTING.md](https://github.com/Strum355/vscode-mc-shader/blob/master/CONTRIBUTING.md).

## Known Issues

I'll fill this in once this actually gets released.

## Release Notes

None yet.
