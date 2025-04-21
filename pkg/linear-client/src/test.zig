const std = @import("std");

const Client = @import("./Client.zig").Client;

test "Test Client" {
    const alloc = std.testing.allocator;
    var c = try Client.init(alloc);
    defer c.deinit(); // Clean up resources

    const issues = try c.issue();
    const teams = try c.teams();
    _ = issues;
    _ = teams;
}
