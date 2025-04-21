const std = @import("std");
const models = @import("./models.zig");
const queries = @import("./queries.zig");
const response = @import("./Response.zig");

const LINEAR_URL = "https://api.linear.app/graphql";

// const LinearClientErrors = error{
//     // 400
//     BadRequest,
//     // 401
//     Unauthorized,
//     // 403
//     Forbidden,
//     // 404
//     NotFound
// };

pub const Client = struct {
    client: std.http.Client,
    token: []const u8,
    alloc: std.mem.Allocator,

    /// Initialize Linear Client
    pub fn init(
        allocator: std.mem.Allocator,
    ) !Client {
        const client = std.http.Client{
            .allocator = allocator,
        };

        return Client{ .client = client, .alloc = allocator, .token = "lin_api_hGYIq2S21hgJkUBNlaBmqLKduyQWlQkW1bLjz7jL" };
    }

    /// Deinit the Linear Client
    pub fn deinit(self: *Client) void {
        self.client.deinit();
        self.* = undefined;
    }

    /// Build a properly formatted GraphQL query payload
    fn buildQueryPayload(self: *Client, query: []const u8) ![]const u8 {
        var escaped_query = std.ArrayList(u8).init(self.alloc);
        defer escaped_query.deinit();

        try std.json.stringify(query, .{}, escaped_query.writer());

        return try std.fmt.allocPrint(self.alloc, "{{\"query\":{s}}}", .{escaped_query.items});
    }

    // Execute a request,
    // I believe payload could be a comptime generic.
    // That way, we can do the stringification inside this func body
    // Could we do seperate stuff for the graphQL schema, and the output?
    fn post(self: *Client, schema: []const u8) !std.ArrayList(u8) {
        std.log.info("Requesting data from endpoint: {s}", .{LINEAR_URL});

        const payload = try self.buildQueryPayload(schema);
        defer self.alloc.free(payload);

        std.log.info("Payload: {s}", .{payload});

        //We can set up any headers we want
        const headers = &[_]std.http.Header{
            .{ .name = "Authorization", .value = self.token },
            .{ .name = "Content-Type", .value = "application/json" },
        };

        var response_body = std.ArrayList(u8).init(self.alloc);
        // In the case where response_body doesnt get returned to the caller, deinit it.
        errdefer response_body.deinit();

        const res = try self.client.fetch(.{
            .method = .POST,
            .location = .{ .url = LINEAR_URL },
            .extra_headers = headers,
            .response_storage = .{ .dynamic = &response_body },
            .payload = payload,
        });

        if (res.status != std.http.Status.ok) {
            std.log.err("Response status: {d}\n", .{res.status});
            std.log.err("{s} \n", .{response_body.items});
            std.log.err("{s} \n", .{payload});
            return error.WrongStatusResponse;
        }
        std.log.info("Response Status: {d}\n ", .{
            res.status,
        });

        std.log.debug("{s} \n", .{response_body.items});

        return response_body;
    }

    pub fn issue(self: *Client) !response.IssueResponse(models.Issue) {
        const res = try self.post(queries.ISSUE);
        defer res.deinit();
        const result = try std.json.parseFromSlice(response.IssueResponse(models.Issue), self.alloc, res.items, .{ .ignore_unknown_fields = true });
        defer result.deinit();
        return result.value;
    }

    pub fn teams(self: *Client) !response.TeamsResponse(models.Teams) {
        const res = try self.post(queries.TEAMS);
        defer res.deinit();
        const result = try std.json.parseFromSlice(response.TeamsResponse(models.Teams), self.alloc, res.items, .{ .ignore_unknown_fields = true });
        defer result.deinit();
        return result.value;
    }
};
