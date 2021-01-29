local Pipeline(name, os) = {
  kind: "pipeline",
  name: name,
  platform: {
      os: os
      arch: "amd64"
  },
  steps: [
    {
      name: "test",
      image: "rust:1.49-slim-buster",
      commands: [
        "cargo build --verbose --all",
        "cargo test --verbose --all"
      ]
    }
  ]
}

[
    Pipeline("Linux", "linux"),
    Pipeline("Windows", "windows"),
    Pipeline("MacOS", "darwin")
],