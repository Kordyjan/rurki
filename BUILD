load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library")

rust_library(
    name = "engine_base",
    srcs = glob(["engine_base/**/*.rs"]),
    deps = [
        "@crates//:crossbeam-channel",
        "@crates//:rustc-hash",
    ],
    visibility = ["//visibility:public"],
)

rust_library(
    name = "simple_engine",
    srcs = glob(["simple_engine/**/*.rs"]),
    deps = [
        ":engine_base",
        "@crates//:crossbeam-channel",
        "@crates//:rustc-hash",
        "@crates//:typed-arena",
    ],
    visibility = ["//visibility:public"],
)
