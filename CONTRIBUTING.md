# Contributing

Want to contribute? I sure want you to! Heres how you can help:

## Setting up environment

Development requirements (did I miss any? Submit a PR!):

`Git`, `NPM`, `NodeJS`, `VSCode`, `Rust`

Fork the repo and clone it (you are using [SSH keys](https://help.github.com/articles/connecting-to-github-with-ssh/), right?):

`git clone git@github.com:<your-username>/vscode-mc-shader.git`

Install dependencies:

`cd vscode-mc-shader/server && npm i && cd ../client && npm i`

Follow [this](https://code.visualstudio.com/docs/extensions/overview) link to learn your way around making extensions as well as [here](https://code.visualstudio.com/docs/extensions/example-language-server) to learn a bit about the Language Server Protocol.

To test out your changes, choose `Launch Client` in the debug menu or press `F5`.

## Submitting a Pull Request

Please adhere to the following guidelines before submitting a pull request:

- Passes eslint checks with the given config.
- Provide some comments in the code where applicable.
- Provide a good explanation of the changes provided. This helps me follow your code better.
