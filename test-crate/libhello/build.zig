const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});
    const hello_module = b.createModule(.{
        .root_source_file = b.path("hello.zig"),
        .target = target,
        .optimize = optimize,
    });

    const libhello = b.addLibrary(.{ .name = "hello", .linkage = .static, .root_module = hello_module });

    switch (optimize) {
        .Debug, .ReleaseSafe => libhello.bundle_compiler_rt = true,
        .ReleaseFast, .ReleaseSmall => {},
    }

    libhello.pie = true;
    if (@hasField(@TypeOf(hello_module.*), "link_libc")) {
        hello_module.link_libc = true;
    } else {
        libhello.linkLibC();
    }

    b.installArtifact(libhello);
}
