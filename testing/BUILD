load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_proc_macro")

rust_proc_macro(
    name = "rig_macros",
    srcs = glob(["rig_macros/*.rs"]),
    deps = [
        "@crates//:quote",
        "@crates//:syn",
        "@crates//:proc-macro2",
        "@crates//:anyhow",
    ],
)

rust_library(
    name = "rig",
    srcs = glob(["rig/*.rs"]),
    proc_macro_deps = [":rig_macros"],
)

rust_binary(
    name = "playground",
    srcs = glob(["playground/*.rs"]),
    deps = [":rig"],
)
