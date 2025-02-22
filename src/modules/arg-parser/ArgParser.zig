const std = @import("std");

// FIXME: Decouple these from here
// and move into a "router"
const AuthHandler = @import("../commands/auth/AuthHandler.zig");
const IssueHandler = @import("../commands/issue/IssueHandler.zig");

// FIXME: Create the command structure
// Should look like: <noun> <verb> <?params> <?flags>
// Example: issue list 18

// FIXME: Move these away from here
// They should be defined in their respective modules
const Commands = enum {
    auth,
    issue,
};

const AuthSubCommands = enum {
    login,
    logout,
    unrecognized,
};

const IssueSubCommands = enum {
    list,
    view,
    unrecognized,
};

pub fn parse(args: [][:0]u8) !void {
    const command_arg = if (args.len > 1) args[1] else "";
    const subcommand_arg = if (args.len > 2) args[2] else "unrecognized";

    if (command_arg.len == 0) {
        std.debug.print("<No Command>: Print help", .{});
        return;
    }

    const command = std.meta.stringToEnum(Commands, command_arg) orelse {
        return error.InvalidChoice;
    };

    // FIXME: Turn into a comptime router
    // and move away from here
    switch (command) {
        .auth => {
            const subcommand = std.meta.stringToEnum(AuthSubCommands, subcommand_arg) orelse {
                return error.InvalidChoice;
            };
            switch (subcommand) {
                .login => {
                    AuthHandler.login();
                    return;
                },
                .logout => {
                    AuthHandler.logout();
                    return;
                },
                .unrecognized => {
                    AuthHandler.unrecognized();
                    return;
                },
            }
        },
        .issue => {
            const subcommand = std.meta.stringToEnum(IssueSubCommands, subcommand_arg) orelse {
                return error.InvalidChoice;
            };
            switch (subcommand) {
                .list => {
                    IssueHandler.list();
                    return;
                },
                .view => {
                    IssueHandler.view();
                    return;
                },
                .unrecognized => {
                    IssueHandler.unrecognized();
                    return;
                },
            }
        },
    }
}
