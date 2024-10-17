load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_proc_macro")
load("utils.bzl", "suite_run" )

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
    deps = [
        ":runner",
        "//:engine_base"
    ],
    proc_macro_deps = [":rig_macros"],
)

rust_library(
    name = "runner",
    srcs = glob(["runner/*.rs"]),
    deps = [
        "@crates//:indicatif",
        "@crates//:console",
        "@crates//:rayon",
        "@crates//:crossbeam-channel",
        "@crates//:anyhow",
    ],
)

suite_run(
    name = "simple_engine",
    deps = [
        ":rig",
        ":runner",
        "//:engine_base",
        "//:simple_engine",
    ],
    suite = "rig::engine_suite",
    constructor = "simple_engine::SimpleEngine::new()",
    timeout = 5,
)
