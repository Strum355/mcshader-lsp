# Change Log

All notable changes to the "vscode-mc-shader" extension will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)

## [0.8.4]

### Fixed

- Filtering out `global const initializers must be constant`. "Something something non-standard shader extensions that GPU developers implicitly enable" - Dethraid

## [0.8.3]

### Fixed

- Filtering out gpu_shader4 in code

## [0.8.2]

### Added

- Support for #include directives
- Basic linting with highlighting with error propogation to all known parents of an include.
- Support for .fsh, .vsh, .glsl and .gsh files.
- Incomplete completion items