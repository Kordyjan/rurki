load("@rules_rust//rust:defs.bzl", "rust_binary")

rust_binary(
    name = "playground",
    srcs = glob(["playground/*.rs"]),
    deps = [
        "//:engine_base",
        "//:simple_engine",
    ],
)
