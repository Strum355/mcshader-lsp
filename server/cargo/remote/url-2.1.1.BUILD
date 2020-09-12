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


# Unsupported target "data" with type "test" omitted
# Unsupported target "parse_url" with type "bench" omitted
# Unsupported target "unit" with type "test" omitted

rust_library(
    name = "url",
    crate_type = "lib",
    deps = [
        "@server__idna__0_2_0//:idna",
        "@server__matches__0_1_8//:matches",
        "@server__percent_encoding__2_1_0//:percent_encoding",
        "@server__serde__1_0_116//:serde",
    ],
    srcs = glob(["**/*.rs"]),
    crate_root = "src/lib.rs",
    edition = "2015",
    rustc_flags = [
        "--cap-lints=allow",
    ],
    version = "2.1.1",
    tags = ["cargo-raze"],
    crate_features = [
        "serde",
    ],
)

