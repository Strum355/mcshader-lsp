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
    sha256 = "f50600b8a56de5c70ad00ba7492dbd73262fadcf83354e69d10f8109e657324a",
    strip_prefix = "rules_rust-67dbb8939be30245bb33cb7a56335101dfc71e17",
    urls = [
        # Master branch as of 2020-09-12
        "https://github.com/Strum355/rules_rust/archive/67dbb8939be30245bb33cb7a56335101dfc71e17.tar.gz",
    ],
)

load("@io_bazel_rules_rust//rust:repositories.bzl", "rust_repositories")
rust_repositories()

load("@io_bazel_rules_rust//:workspace.bzl", "bazel_version")
bazel_version(name = "bazel_version")

load("//server/cargo:crates.bzl", "server_fetch_remote_crates")
server_fetch_remote_crates()