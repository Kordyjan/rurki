load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library")

rust_library(
    name = "rig",
    srcs = glob(["rig/*.rs"]),
)

rust_binary(
    name = "playground",
    srcs = glob(["playground/*.rs"]),
    deps = [":rig"],
)
