// GhostChain networking integration - combines zquic and ghostnet
const zquic = @import("zquic");
const ghostnet = @import("ghostnet");

pub const Node = @import("node.zig").Node;
pub const Peer = @import("node.zig").Peer;
pub const quic = zquic;
pub const P2PManager = @import("p2p.zig").P2PManager;
pub const P2PConfig = @import("p2p.zig").P2PConfig;

// Re-export zquic and ghostnet functionality
pub const QUIC = zquic.QUIC;
pub const QuicClient = zquic.Client;
pub const QuicServer = zquic.Server;
pub const GhostNet = ghostnet.GhostNet;
pub const DHT = ghostnet.DHT;
pub const Gossip = ghostnet.Gossip;