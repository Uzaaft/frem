/// Nodes is a generic wrapper for arrays of items returned by the Linear API
pub fn Nodes(comptime T: type) type {
    return struct {
        nodes: []T,
    };
}

/// Data is a generic wrapper for the data field in a Linear API response
pub fn Data(comptime T: type) type {
    return struct {
        data: T,
    };
}

/// Response is a generic wrapper for Linear API responses
/// It takes a type T and returns a struct that can parse responses like:
/// {"data":{"issues":{"nodes":[{"id":"...","title":"..."},...]}}}
pub fn IssueResponse(comptime T: type) type {
    return struct {
        data: struct {
            issues: Nodes(T),
        },
    };
}

pub fn TeamsResponse(comptime T: type) type {
    return struct {
        data: struct {
            teams: Nodes(T),
        },
    };
}
