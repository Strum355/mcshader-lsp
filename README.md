# Minecraft GLSL Shaders Language Server
## mcshader-lsp

[![Marketplace Version](https://vsmarketplacebadge.apphb.com/version/strum355.vscode-mc-shader.svg)](https://marketplace.visualstudio.com/items?itemName=strum355.vscode-mc-shader) [![Installs](https://vsmarketplacebadge.apphb.com/installs/strum355.vscode-mc-shader.svg)](https://marketplace.visualstudio.com/items?itemName=strum355.vscode-mc-shader)
[![license](https://img.shields.io/github/license/Strum355/vscode-mc-shader.svg)](https://github.com/Strum355/mcshader-lsp)
[![Issues](https://img.shields.io/github/issues-raw/Strum355/mcshader-lsp.svg)](https://github.com/Strum355/mcshader-lsp/issues)
[![Build Status](https://img.shields.io/drone/build/Strum355/mcshader-lsp)](https://cloud.drone.io/Strum355/mcshader-lsp)

mcshader-lsp is a [Language Server](https://langserver.org/) and collection of editor extensions for developing Minecraft GLSL Shaders for [Optifine](http://optifine.net). It currently provides linting and syntax highlighting.

Currently supported editors:

- [Visual Studio Code](https://code.visualstudio.com/) with `vscode-mc-shader`

<img src="https://github.com/Strum355/mcshader-lsp/raw/rust-rewrite/logo.png" width="20%" height="20%">

## Features

- Linting
- Syntax highlighting
- Support for `#include` directives
- Displaying `#include` flattened file
- Generating Graphviz DOT `#include` dependency graph
<!-- - Auto-complete prompts (incomplete and rough) -->

## Requirements

- That you've only one shader folder open. Multiple workspaces aren't currently supported.
- The root folder of the workspace is the parent folder of `shaders` folder.

<!-- ## Extension Settings

| Option Name | Data Type | Description | Default Value |
| ----------- | --------- | ----------- | ------------- |
| `mcglsl.glslangValidatorPath` | string |  The path to the glslangValidator executable. | In your `PATH`.| -->

## Contributing

Please see [CONTRIBUTING.md](https://github.com/Strum355/mcshader-lsp/blob/master/CONTRIBUTING.md).

## Planned

- Multi-workspaces (currently only one is supported and using multiple is very undefined behaviour)
- Warnings for unused uniforms/varyings
- Lint for all #define value combinations
- Compute shader support
- Some cool `DRAWBUFFERS` stuff

Got a feature request? Chuck it into an Issue!

## Known Issues

Check the issues on Github [here](https://github.com/Strum355/mcshader-lsp/issues?q=is%3Aissue+is%3Aopen+sort%3Aupdated-desc+label%3Abug).

## Release Notes

Check [CHANGELOG.md](https://github.com/Strum355/mcshader-lsp/blob/master/CHANGELOG.md).

## License

This code is released under the [MIT License](https://github.com/Strum355/mcshader-lsp/blob/master/LICENSE). Copyright (c) 2021 Noah Santschi-Cooney
