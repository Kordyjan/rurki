load("@rules_rust//rust:defs.bzl", "rust_binary")

def _gen_testfile_impl(ctx):
    output = ctx.actions.declare_file("testfile.rs")
    ctx.actions.expand_template(
        template = ctx.file._template,
        output = output,
        substitutions = {
            "{cons}": ctx.attr.constructor,
            "{suite}": ctx.attr.suite,
            "{timeout}": str(ctx.attr.timeout),
        },
    )
    return [
        DefaultInfo(files = depset([output])),
    ]

gen_testfile = rule(
    implementation = _gen_testfile_impl,
    attrs = {
        "constructor": attr.string(mandatory=True),
        "suite": attr.string(mandatory=True),
        "_template": attr.label(
            allow_single_file = True,
            default = Label(":test_template.txt"),
            executable = False,
        ),
        "timeout": attr.int(default = 60),
    },
)

def suite_run(name, deps, constructor, suite, timeout = 60):
    gen_testfile(
        name = name + "_testfile",
        constructor = constructor,
        suite = suite,
        timeout = timeout,
    )

    rust_binary(
        name = name,
        srcs = [":" + name + "_testfile"],
        deps = deps,
    )
