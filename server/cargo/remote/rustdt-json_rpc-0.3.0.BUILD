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
# Unsupported target "example" with type "test" omitted
alias(
  name = "rustdt_json_rpc",
  actual = ":jsonrpc",
  tags = ["cargo-raze"],
)

rust_library(
    name = "jsonrpc",
    crate_type = "lib",
    deps = [
        "@server__futures__0_1_29//:futures",
        "@server__log__0_4_11//:log",
        "@server__rustdt_util__0_2_3//:rustdt_util",
        "@server__serde__1_0_116//:serde",
        "@server__serde_json__1_0_57//:serde_json",
    ],
    srcs = glob(["**/*.rs"]),
    crate_root = "src/jsonrpc.rs",
    edition = "2015",
    rustc_flags = [
        "--cap-lints=allow",
    ],
    version = "0.3.0",
    tags = ["cargo-raze"],
    crate_features = [
    ],
)

# Unsupported target "tests_sample_types" with type "test" omitted
