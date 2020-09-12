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


# Unsupported target "bigint" with type "bench" omitted
# Unsupported target "bigint" with type "test" omitted
# Unsupported target "bigint_bitwise" with type "test" omitted
# Unsupported target "bigint_scalar" with type "test" omitted
# Unsupported target "biguint" with type "test" omitted
# Unsupported target "biguint_scalar" with type "test" omitted
# Unsupported target "build-script-build" with type "custom-build" omitted
# Unsupported target "factorial" with type "bench" omitted
# Unsupported target "gcd" with type "bench" omitted
# Unsupported target "modpow" with type "test" omitted

rust_library(
    name = "num_bigint",
    crate_type = "lib",
    deps = [
        "@server__num_integer__0_1_43//:num_integer",
        "@server__num_traits__0_2_12//:num_traits",
    ],
    srcs = glob(["**/*.rs"]),
    crate_root = "src/lib.rs",
    edition = "2015",
    rustc_flags = [
        "--cap-lints=allow",
    ],
    version = "0.2.6",
    tags = ["cargo-raze"],
    crate_features = [
        "std",
    ],
)

# Unsupported target "quickcheck" with type "test" omitted
# Unsupported target "rand" with type "test" omitted
# Unsupported target "roots" with type "bench" omitted
# Unsupported target "roots" with type "test" omitted
# Unsupported target "serde" with type "test" omitted
# Unsupported target "shootout-pidigits" with type "bench" omitted
# Unsupported target "torture" with type "test" omitted
