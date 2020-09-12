load("@io_bazel_rules_rust//rust:rust.bzl", "rust_binary")
load("@io_bazel_rules_rust//:aspects.bzl", "rust_analyzer")

package(default_visibility = ["//visibility:public"])

rust_analyzer(
    name = "mcshader-lsp-ra",
    targets = ["//server"],
    exec_root = "bazel-rules_rust"
)

rust_binary(
    name = "mcshader-lsp",
    srcs = glob(["src/*.rs"]),
    deps = [
        "//server/cargo:anyhow",
        "//server/cargo:bit_set",
        "//server/cargo:chan",
        "//server/cargo:fs_extra",
        "//server/cargo:hamcrest2",
        "//server/cargo:lazy_static",
        "//server/cargo:percent_encoding",
        "//server/cargo:petgraph",
        "//server/cargo:regex",
        "//server/cargo:rust_lsp",
        "//server/cargo:serde",
        "//server/cargo:serde_json",
        "//server/cargo:tempdir",
        "//server/cargo:thiserror",
        "//server/cargo:url",
        "//server/cargo:walkdir",
        "//server/cargo:log"
    ],
)