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
  "notice", # MIT from expression "MIT"
])

load(
    "@io_bazel_rules_rust//rust:rust.bzl",
    "rust_library",
    "rust_binary",
    "rust_test",
)



rust_library(
    name = "lsp_types",
    crate_type = "lib",
    deps = [
        "@server__base64__0_12_3//:base64",
        "@server__bitflags__1_2_1//:bitflags",
        "@server__serde__1_0_116//:serde",
        "@server__serde_json__1_0_57//:serde_json",
        "@server__url__2_1_1//:url",
    ],
    srcs = glob(["**/*.rs"]),
    crate_root = "src/lib.rs",
    edition = "2018",
    proc_macro_deps = [
        "@server__serde_repr__0_1_6//:serde_repr",
    ],
    rustc_flags = [
        "--cap-lints=allow",
    ],
    version = "0.80.0",
    tags = ["cargo-raze"],
    crate_features = [
        "default",
    ],
)

