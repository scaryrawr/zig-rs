const std = @import("std");

export fn greet(name: [*:0]const u8) ?*const u8 {
    // Leaks memory, but that's fine for this test.
    const allocator = std.heap.page_allocator;
    const greeting = "Hello, ";
    const ending = "!";
    const name_len = std.mem.len(name);
    const buffer = allocator.alloc(u8, greeting.len + name_len + ending.len + 1) catch return null;
    @memcpy(buffer[0..greeting.len], greeting);
    @memcpy(buffer[greeting.len..(greeting.len + name_len)], name);
    @memcpy(buffer[(greeting.len + name_len)..(greeting.len + name_len + ending.len)], ending);
    buffer[greeting.len + name_len + ending.len] = 0;
    return &buffer[0];
}
