const std = @import("std");

const Client = struct {
    client: std.http.Client,
    token: []const u8,
    alloc: std.mem.Allocator,

    /// Initialize Linear Client
    pub fn init(allocator: std.mem.Allocator) !Client {
        const http_client = std.http.Client{
            .allocator = allocator,
        };

        return .{ .client = http_client };
    }

    // Execute a request,
    fn post(self: Client, url: []const u8, body: []const u8) !std.ArrayList(u8) {
        std.log.info("Requesting data from endpoint: {}", .{url});

        //We can set up any headers we want
        const headers = &[_]std.http.Header{
            .{ .name = "Authorization", .value = self.token },
            .{ .name = "Content-Type", .value = "application/json" },
        };

        const response_body = std.ArrayList(u8).init(self.alloc);

        const res = try self.client.fetch(.{
            .method = .POST,
            .location = .{ .url = url },
            .extra_headers = headers,
            .response_storage = .{ .dynamic = &response_body },
            .payload = body,
        });

        std.log.info("Response Status: {d}\n Response Body:{s}\n", .{ res.status, response_body.items });

        return response_body;
    }
};
