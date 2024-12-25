const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const libhello = b.addStaticLibrary(.{
        .name = "hello",
        .root_source_file = b.path("hello.zig"),
        .target = target,
        .optimize = optimize,
    });

    switch (optimize) {
        .Debug, .ReleaseSafe => libhello.bundle_compiler_rt = true,
        .ReleaseFast, .ReleaseSmall => {},
    }

    b.installArtifact(libhello);
}
