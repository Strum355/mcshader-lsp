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



rust_library(
    name = "idna",
    crate_type = "lib",
    deps = [
        "@server__matches__0_1_8//:matches",
        "@server__unicode_bidi__0_3_4//:unicode_bidi",
        "@server__unicode_normalization__0_1_13//:unicode_normalization",
    ],
    srcs = glob(["**/*.rs"]),
    crate_root = "src/lib.rs",
    edition = "2015",
    rustc_flags = [
        "--cap-lints=allow",
    ],
    version = "0.2.0",
    tags = ["cargo-raze"],
    crate_features = [
    ],
)

# Unsupported target "tests" with type "test" omitted
# Unsupported target "unit" with type "test" omitted
