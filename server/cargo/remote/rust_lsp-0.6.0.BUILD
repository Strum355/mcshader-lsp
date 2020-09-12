"""
@generated
cargo-raze crate build file.

DO NOT EDIT! Replaced on runs of cargo-raze
"""
package(default_visibility = [
  # Public for visibility by "@raze__crate__version//" targets.
  #
  # Prefer access through "//server/cargo", which limits external
  # visibility to explicit Cargo.toml dependencies.
  "//visibility:public",
])

licenses([
  "notice", # Apache-2.0 from expression "Apache-2.0"
])

load(
    "@io_bazel_rules_rust//rust:rust.bzl",
    "rust_library",
    "rust_binary",
    "rust_test",
)


# Unsupported target "dummy" with type "test" omitted

rust_library(
    name = "rust_lsp",
    crate_type = "lib",
    deps = [
        "@server__log__0_4_11//:log",
        "@server__lsp_types__0_80_0//:lsp_types",
        "@server__rustdt_json_rpc__0_3_0//:rustdt_json_rpc",
        "@server__rustdt_util__0_2_3//:rustdt_util",
        "@server__serde__1_0_116//:serde",
        "@server__serde_json__1_0_57//:serde_json",
    ],
    srcs = glob(["**/*.rs"]),
    crate_root = "src/lib.rs",
    edition = "2015",
    rustc_flags = [
        "--cap-lints=allow",
    ],
    version = "0.6.0",
    tags = ["cargo-raze"],
    crate_features = [
    ],
)

