# Change Log

All notable changes to the "vscode-mc-shader" extension will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)

## [0.9.8]

### Fixed

- NVIDIA diagnostics line offset off-by-one due to confusion with erroneous (non-proper) GLSL files resulting in both -1 and -2 offsets appearing to be valid when only the former is.
- Non-toplevel files being treated as toplevel files when they have .fsh/.vsh/etc and not imported into a valid toplevel file.
- Fix issue in the depth-first-search iterator when a file is imported twice into another file with a different include in between.

## [0.9.7]

### Fixed

- Fixed bad release tag format
- Fixed extension silently failing on activation

## [0.9.6]

### Added

- MacOS M1 binary releases
- AMD OpenGL driver diagnostics output support. AMD linting is a-go 🚀
- Tree-sitter based go-to-definition/find-references/document symbols. Currently disabled until stabilized

### Fixed

- Another `#include` merging bug when a file is imported twice into another file at different lines

## [0.9.5]

### Added

- Filesystem watcher reads custom defined file associations

### Fixed

- Fixed `#include` merging for when file is merged twice that would normally be `#ifdef` guarded. Please see commit message of [551380a](https://github.com/Strum355/mcshader-lsp/commit/551380a6ed00709287460b7d8c88e7803956052c) for detailed explanation

## [0.9.4]

### Fixed

- `#include` merging when project consists of files with both CRLF and LF files
- Out-of-tree shader files are not linted or added to the dependency graph
- Client no longer attempts to bootstrap server when `MCSHADER_DEBUG=true`

## [0.9.3]

### Fixed

- Language server download for windows

## [0.9.2]

### Changed

- VSCode extension activation predicate to only when `shaders` folder exists at top level

### Added

- Additional client-side logging

## [0.9.1]

### Fixed

- Windows support in client not adding `.exe` to language server path
- Binary release CI

## [0.9.0]

### Changed

- Replaced in-process Typescript language server with Rust based language server

### Fixed

- Due to the above, `#include` directive handling is vastly improved

### Added

- Command to view read-only document representing a top-level file with all includes merged
- Command to generate a DOT graph file of the entire project
- Command to restart language server

### Removed

- `glslangValidatorPath` and `shaderpacksPath` config settings

## [0.8.5]

### Fixed

- Fixed for latest VSCode version

### Removed

- Filters from 0.8.4

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
