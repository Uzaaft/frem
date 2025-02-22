const std = @import("std");

pub fn list() void {
    std.debug.print("<Issue list>: Not Implemeted!\n", .{});
}

pub fn view() void {
    std.debug.print("<Issue view>: Not Implemeted!\n", .{});
}

pub fn unrecognized() void {
    std.debug.print("<Issue unrecognized command>\n", .{});
}
