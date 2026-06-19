const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});
    const math_module = b.createModule(.{
        .root_source_file = b.path("math.zig"),
        .target = target,
        .optimize = optimize,
    });

    const lib = b.addLibrary(.{
        .name = "zig_math",
        .linkage = .static,
        .root_module = math_module,
    });

    b.installArtifact(lib);
}
