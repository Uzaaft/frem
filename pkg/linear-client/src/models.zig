const std = @import("std");

// pub const Me = struct { id: []const u8, name: []const u8, email: []const u8 };
pub const IssueTitleID = struct {
    /// ID field
    id: []const u8,
    /// Issue title
    title: []const u8,
};
pub const IssueAllMeta = struct {
    /// ID field
    id: []const u8,
    /// Title
    title: []const u8,
    /// Created at date
    createdAt: []const u8,
    /// Updated at date
    updatedAt: []const u8,
};

pub const Teams = struct {
    /// Team ID
    id: []const u8,
    /// Team name
    name: []const u8,
};
