const std = @import("std");

const Client = @import("./Client.zig").Client;

test "Test Client" {
    const alloc = std.testing.allocator;
    var c = try Client.init(alloc);
    defer c.deinit(); // Clean up resources

    const res = try c.me();
    _ = res;
}
