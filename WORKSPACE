workspace(name="mcshader_lsp")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "bazel_skylib",
    sha256 = "97e70364e9249702246c0e9444bccdc4b847bed1eb03c5a3ece4f83dfe6abc44",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/bazel-skylib/releases/download/1.0.2/bazel-skylib-1.0.2.tar.gz",
        "https://github.com/bazelbuild/bazel-skylib/releases/download/1.0.2/bazel-skylib-1.0.2.tar.gz",
    ],
)

load("@bazel_skylib//:workspace.bzl", "bazel_skylib_workspace")
bazel_skylib_workspace()

http_archive(
    name = "io_bazel_rules_rust",
    sha256 = "ceee3ecc1bc134f42f448b88907649f1ab3930c52772d63d63e3f504b92cf8e9",
    strip_prefix = "rules_rust-6865219d2ddf7849bd8c98d8dc44715660452bde",
    urls = [
        # Master branch as of 2020-09-12
        "https://github.com/Strum355/rules_rust/archive/6865219d2ddf7849bd8c98d8dc44715660452bde.tar.gz",
    ],
)

load("@io_bazel_rules_rust//rust:repositories.bzl", "rust_repositories")
rust_repositories()

load("@io_bazel_rules_rust//:workspace.bzl", "bazel_version")
bazel_version(name = "bazel_version")

load("//server/cargo:crates.bzl", "server_fetch_remote_crates")
server_fetch_remote_crates()