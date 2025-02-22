const std = @import("std");

const Client = struct {
    client: std.http.Client,

    /// Initialize Linear Client
    pub fn init(allocator: std.mem.Allocator) !Client {
        const http_client = std.http.Client{
            .allocator = allocator,
        };

        return .{ .client = http_client };
    }
};
