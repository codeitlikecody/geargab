use geargab_core::models::Heartbeat;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, PartialEq)]
pub struct PeerInfo {
    pub heartbeat: Heartbeat,
    pub addr: SocketAddr,
    pub last_seen_local: i64,
}

#[derive(Debug, Clone, Default)]
pub struct PeerTracker {
    peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
}

impl PeerTracker {
    pub fn new() -> Self {
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Records or updates an active peer from a received heartbeat and socket address.
    pub fn update_peer(&self, hb: Heartbeat, addr: SocketAddr, now_timestamp: i64) {
    if let Ok(mut peers) = self.peers.write() {
        peers
            .entry(hb.client_uuid.clone())
            .and_modify(|entry| {
                entry.heartbeat = hb.clone();
                entry.addr = addr;
                // Only advance timestamp forward to protect against out-of-order network arrival
                if now_timestamp > entry.last_seen_local {
                    entry.last_seen_local = now_timestamp;
                }
            })
            .or_insert(PeerInfo {
                heartbeat: hb,
                addr,
                last_seen_local: now_timestamp,
            });
        }
    }

    /// Returns a list of all active peers seen within `timeout_seconds` of `now_timestamp`.
    pub fn get_active_peers(&self, now_timestamp: i64, timeout_seconds: i64) -> Vec<PeerInfo> {
        let cutoff = now_timestamp - timeout_seconds;
        if let Ok(peers) = self.peers.read() {
            peers
                .values()
                .filter(|peer| peer.last_seen_local >= cutoff)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Prunes stale peers that haven't sent a heartbeat within `timeout_seconds`.
    pub fn prune_stale_peers(&self, now_timestamp: i64, timeout_seconds: i64) -> usize {
        let cutoff = now_timestamp - timeout_seconds;
        if let Ok(mut peers) = self.peers.write() {
            let initial_count = peers.len();
            peers.retain(|_, peer| peer.last_seen_local >= cutoff);
            initial_count - peers.len()
        } else {
            0
        }
    }

    /// Returns the current total count of tracked peers.
    pub fn peer_count(&self) -> usize {
        if let Ok(peers) = self.peers.read() {
            peers.len()
        } else {
            0
        }
    }
}