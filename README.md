# vscode-mc-shader

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

- Visual Studio Code of course
- Not MacOSX. Not that you're making MC Shaders on/for MacOSX anyways...right?
- The [Shader languages support for VS Code](https://marketplace.visualstudio.com/items?itemName=slevesque.shader) extension. This should automatically install when you install this extension
- The [OpenGL / OpenGL ES Reference Compiler](https://cvs.khronos.org/svn/repos/ogl/trunk/ecosystem/public/sdk/tools/glslang/Install/) (for convenience, put it in your PATH)

## Extension Settings

- `mcglsl.glslangValidatorPath` : The path to the glslangValidator executable. If not provided, it assumes its in your `PATH`.

## Known Issues

I'll fill this in once this actually gets released.

## Release Notes

None yet.
