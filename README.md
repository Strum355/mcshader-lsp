# Minecraft GLSL Shaders Language Server
## mcshader-lsp

[![Marketplace Version](https://vsmarketplacebadge.apphb.com/version/strum355.vscode-mc-shader.svg)](https://marketplace.visualstudio.com/items?itemName=strum355.vscode-mc-shader) [![Installs](https://vsmarketplacebadge.apphb.com/installs/strum355.vscode-mc-shader.svg)](https://marketplace.visualstudio.com/items?itemName=strum355.vscode-mc-shader)
[![license](https://img.shields.io/github/license/Strum355/vscode-mc-shader.svg)](https://github.com/Strum355/mcshader-lsp)
[![Issues](https://img.shields.io/github/issues-raw/Strum355/mcshader-lsp.svg)](https://github.com/Strum355/mcshader-lsp/issues)
[![Build Status](https://img.shields.io/drone/build/Strum355/mcshader-lsp)](https://cloud.drone.io/Strum355/mcshader-lsp)

mcshader-lsp is a [Language Server](https://langserver.org/) and collection of editor extensions for developing Minecraft GLSL Shaders for [Optifine](http://optifine.net). It currently provides linting and syntax highlighting.

Currently supported editors:

- [Visual Studio Code](https://code.visualstudio.com/)

<img src="https://github.com/Strum355/mcshader-lsp/raw/master/logo.png" width="20%" height="20%">

## Features

- Linting
- Syntax highlighting (by extension dependency)
- Support for `#include` directives
<!-- - Auto-complete prompts (incomplete and rough) -->

## Installation (assumes installing from VSCode extension tab)

- After reloading, open a shaderpack's folder.
- You should be prompted to set your shaderpacks folder e.g. `"mcglsl.shaderpacksPath": "C:/Users/Noah/AppData/Roaming/.minecraft/shaderpacks"`
- You should then be prompted saying `glslangValidator` isn't installed. Hit the download button and wait for a notification saying that it's been installed.
- You should now be good to go!

## Requirements

- Visual Studio Code (v1.43.0 or higher - minimum requirement untested).
- The [Shader languages support for VS Code](https://marketplace.visualstudio.com/items?itemName=slevesque.shader) extension. This should automatically install when you install this extension.
- That you've only one shader folder open. Multiple workspaces aren't currently supported.

<!-- ## Extension Settings

| Option Name | Data Type | Description | Default Value |
| ----------- | --------- | ----------- | ------------- |
| `mcglsl.glslangValidatorPath` | string |  The path to the glslangValidator executable. | In your `PATH`.| -->

## Contributing

Please see [CONTRIBUTING.md](https://github.com/Strum355/mcshader-lsp/blob/master/CONTRIBUTING.md).

## Planned

- Multi-workspaces (currently only one is supported and using multiple is very undefined behaviour)
- Warnings for unused uniforms/varyings
- Some cool `DRAWBUFFERS` stuff

Got a feature request? Chuck it into an Issue!

## Known Issues

Check the issues on Github [here](https://github.com/Strum355/mcshader-lsp/issues?q=is%3Aissue+is%3Aopen+sort%3Aupdated-desc+label%3Abug).

## Release Notes

Check [CHANGELOG.md](https://github.com/Strum355/mcshader-lsp/blob/master/CHANGELOG.md).

## License

This code is released under the [MIT License](https://github.com/Strum355/mcshader-lsp/blob/master/LICENSE). Copyright (c) 2018 Noah Santschi-Cooney
