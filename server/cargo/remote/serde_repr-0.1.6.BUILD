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
  "notice", # MIT from expression "MIT OR Apache-2.0"
])

load(
    "@io_bazel_rules_rust//rust:rust.bzl",
    "rust_library",
    "rust_binary",
    "rust_test",
)


# Unsupported target "compiletest" with type "test" omitted

rust_library(
    name = "serde_repr",
    crate_type = "proc-macro",
    deps = [
        "@server__proc_macro2__1_0_21//:proc_macro2",
        "@server__quote__1_0_7//:quote",
        "@server__syn__1_0_40//:syn",
    ],
    srcs = glob(["**/*.rs"]),
    crate_root = "src/lib.rs",
    edition = "2018",
    rustc_flags = [
        "--cap-lints=allow",
    ],
    version = "0.1.6",
    tags = ["cargo-raze"],
    crate_features = [
    ],
)

# Unsupported target "test" with type "test" omitted
