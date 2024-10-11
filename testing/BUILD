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
    ],
)

rust_binary(
    name = "playground",
    srcs = glob(["playground/*.rs"]),
    deps = [
        ":rig",
        ":runner",
    ],
)

suite_run(
    name = "sanity",
    deps = [
        ":rig",
        ":runner"
    ],
    suite = "rig::sanity_suite::sanity::suite",
    constructor = "std::vec![1u64, 2u64].into_iter()"
)
