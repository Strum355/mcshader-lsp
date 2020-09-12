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


# Unsupported target "all" with type "test" omitted
# Unsupported target "any" with type "test" omitted
# Unsupported target "anything" with type "test" omitted
# Unsupported target "boolean" with type "test" omitted
# Unsupported target "close_to" with type "test" omitted
# Unsupported target "compared_to" with type "test" omitted
# Unsupported target "contains" with type "test" omitted
# Unsupported target "empty" with type "test" omitted
# Unsupported target "equal_to" with type "test" omitted
# Unsupported target "err" with type "test" omitted

rust_library(
    name = "hamcrest2",
    crate_type = "lib",
    deps = [
        "@server__num__0_2_1//:num",
        "@server__regex__1_3_9//:regex",
    ],
    srcs = glob(["**/*.rs"]),
    crate_root = "src/lib.rs",
    edition = "2018",
    rustc_flags = [
        "--cap-lints=allow",
    ],
    version = "0.3.0",
    tags = ["cargo-raze"],
    crate_features = [
    ],
)

# Unsupported target "has" with type "test" omitted
# Unsupported target "len" with type "test" omitted
# Unsupported target "none" with type "test" omitted
# Unsupported target "ok" with type "test" omitted
# Unsupported target "path_exists" with type "test" omitted
# Unsupported target "regex" with type "test" omitted
# Unsupported target "some" with type "test" omitted
# Unsupported target "type_of" with type "test" omitted
