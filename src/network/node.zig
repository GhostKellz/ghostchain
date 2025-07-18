const std = @import("std");
const zquic = @import("zquic");

pub const Node = struct {
    allocator: std.mem.Allocator,
    address: []const u8,
    port: u16,
    peers: std.ArrayList(Peer),
    
    pub fn init(allocator: std.mem.Allocator, address: []const u8, port: u16) Node {
        return Node{
            .allocator = allocator,
            .address = address,
            .port = port,
            .peers = std.ArrayList(Peer).init(allocator),
        };
    }
    
    pub fn deinit(self: *Node) void {
        self.peers.deinit();
    }
    
    pub fn start(self: *Node) !void {
        std.log.info("Starting GhostChain node on {s}:{}", .{ self.address, self.port });
        // TODO: Implement QUIC server using zquic
    }
    
    pub fn addPeer(self: *Node, peer: Peer) !void {
        try self.peers.append(peer);
    }
    
    pub fn broadcast(self: *Node, message: []const u8) !void {
        for (self.peers.items) |peer| {
            try peer.send(message);
        }
    }
};

pub const Peer = struct {
    address: []const u8,
    port: u16,
    connected: bool,
    
    pub fn init(address: []const u8, port: u16) Peer {
        return Peer{
            .address = address,
            .port = port,
            .connected = false,
        };
    }
    
    pub fn connect(self: *Peer) !void {
        // TODO: Implement QUIC connection to peer
        self.connected = true;
    }
    
    pub fn send(self: *Peer, message: []const u8) !void {
        if (!self.connected) return error.NotConnected;
        // TODO: Send message via QUIC
        _ = message;
    }
};