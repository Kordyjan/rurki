load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library")

rust_library(
    name = "engine_base",
    srcs = glob(["engine_base/*.rs"]),
)
