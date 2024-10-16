play:
    bazel run demos:playground

clippy:
    bazel build //... --config=clippy

fmt:
    bazel run @rules_rust//:rustfmt

project:
    bazel run @rules_rust//tools/rust_analyzer:gen_rust_project

build:
    bazel build //...
