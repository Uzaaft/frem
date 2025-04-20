const std = @import("std");
const Client = @import("Client.zig").Client;

test {
    std.testing.log_level = .debug;
    _ = @import("./test.zig");
}
