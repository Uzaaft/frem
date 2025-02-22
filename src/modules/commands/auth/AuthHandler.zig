const std = @import("std");

pub fn login() void {
    std.debug.print("<Auth login>: Not Implemented!\n", .{});
}

pub fn logout() void {
    std.debug.print("<Auth logout>: Not Implemented!\n", .{});
}

pub fn unrecognized() void {
    std.debug.print("<Auth unrecognized command>\n", .{});
}
