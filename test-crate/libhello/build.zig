const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const libhello = if (b.args.target.contains("linux"))
        b.addSharedLibrary(.{
            .name = "hello",
            .root_source_file = b.path("hello.zig"),
            .target = target,
            .optimize = optimize,
        })
    else
        b.addStaticLibrary(.{
            .name = "hello",
            .root_source_file = b.path("hello.zig"),
            .target = target,
            .optimize = optimize,
        });

    b.installArtifact(libhello);
}
