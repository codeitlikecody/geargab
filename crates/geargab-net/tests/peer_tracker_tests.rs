use geargab_core::models::{ClientType, Heartbeat};
use geargab_net::PeerTracker;
use std::net::SocketAddr;

#[test]
fn test_peer_tracker_add_and_get() -> Result<(), Box<dyn std::error::Error>> {
    let tracker = PeerTracker::new();
    let addr: SocketAddr = "192.168.1.50:9000".parse()?;

    let hb = Heartbeat {
        client_uuid: "peer-001".to_string(),
        display_name: "Sound Desk".to_string(),
        client_type: ClientType::AudioConsole,
        timestamp: 1000,
        seq_num: 1,
    };

    tracker.update_peer(hb.clone(), addr, 1000);

    let active = tracker.get_active_peers(1000, 10);
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].heartbeat.client_uuid, "peer-001");
    assert_eq!(active[0].addr, addr);

    Ok(())
}

#[test]
fn test_peer_tracker_eviction_timeout() -> Result<(), Box<dyn std::error::Error>> {
    let tracker = PeerTracker::new();
    let addr: SocketAddr = "192.168.1.50:9000".parse()?;

    let hb_fresh = Heartbeat {
        client_uuid: "peer-fresh".to_string(),
        display_name: "Lighting Desk".to_string(),
        client_type: ClientType::LightingConsole,
        timestamp: 1010,
        seq_num: 1,
    };

    let hb_stale = Heartbeat {
        client_uuid: "peer-stale".to_string(),
        display_name: "Video Desk".to_string(),
        client_type: ClientType::Desktop,
        timestamp: 990,
        seq_num: 1,
    };

    tracker.update_peer(hb_fresh, addr, 1010);
    tracker.update_peer(hb_stale, addr, 990);

    assert_eq!(tracker.peer_count(), 2);

    // With a 15 second timeout at timestamp 1015:
    // fresh (1010) is within 1015 - 15 = 1000 cutoff
    // stale (990) is older than 1000 cutoff
    let pruned_count = tracker.prune_stale_peers(1015, 15);
    assert_eq!(pruned_count, 1);
    assert_eq!(tracker.peer_count(), 1);

    let active = tracker.get_active_peers(1015, 15);
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].heartbeat.client_uuid, "peer-fresh");

    Ok(())
}

#[test]
fn test_peer_tracker_exact_cutoff_boundary() -> Result<(), Box<dyn std::error::Error>> {
    let tracker = PeerTracker::new();
    let addr: SocketAddr = "127.0.0.1:9000".parse()?;

    let hb_exact = Heartbeat {
        client_uuid: "peer-exact".to_string(),
        display_name: "Desk Exact".to_string(),
        client_type: ClientType::LightingConsole,
        timestamp: 100,
        seq_num: 1,
    };

    let hb_expired = Heartbeat {
        client_uuid: "peer-expired".to_string(),
        display_name: "Desk Expired".to_string(),
        client_type: ClientType::LightingConsole,
        timestamp: 99,
        seq_num: 1,
    };

    // Recorded at t=100 and t=99
    tracker.update_peer(hb_exact, addr, 100);
    tracker.update_peer(hb_expired, addr, 99);

    // At now=110 with timeout=10, cutoff is 110 - 10 = 100
    // t=100 is EXACTLY on cutoff (>= 100) -> Active
    // t=99 is below cutoff (< 100) -> Stale
    let active = tracker.get_active_peers(110, 10);
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].heartbeat.client_uuid, "peer-exact");

    let pruned = tracker.prune_stale_peers(110, 10);
    assert_eq!(pruned, 1);
    assert_eq!(tracker.peer_count(), 1);

    Ok(())
}

#[test]
fn test_peer_tracker_out_of_order_heartbeats() -> Result<(), Box<dyn std::error::Error>> {
    let tracker = PeerTracker::new();
    let addr: SocketAddr = "127.0.0.1:9000".parse()?;

    let hb_newer = Heartbeat {
        client_uuid: "peer-ooo".to_string(),
        display_name: "Desk OOO".to_string(),
        client_type: ClientType::AudioConsole,
        timestamp: 1005,
        seq_num: 10,
    };

    let hb_older = Heartbeat {
        client_uuid: "peer-ooo".to_string(),
        display_name: "Desk OOO".to_string(),
        client_type: ClientType::AudioConsole,
        timestamp: 1000,
        seq_num: 9,
    };

    // Newer heartbeat arrives first
    tracker.update_peer(hb_newer, addr, 1005);
    // Out-of-order delayed heartbeat arrives second
    tracker.update_peer(hb_older, addr, 1000);

    let active = tracker.get_active_peers(1005, 10);
    assert_eq!(active.len(), 1);
    // last_seen_local must remain 1005
    assert_eq!(active[0].last_seen_local, 1005);

    Ok(())
}

#[test]
fn test_peer_tracker_ip_rebind() -> Result<(), Box<dyn std::error::Error>> {
    let tracker = PeerTracker::new();
    let addr1: SocketAddr = "10.0.0.10:9000".parse()?;
    let addr2: SocketAddr = "10.0.0.20:9000".parse()?;

    let hb = Heartbeat {
        client_uuid: "peer-rebind".to_string(),
        display_name: "Mobile iPad".to_string(),
        client_type: ClientType::Desktop,
        timestamp: 1000,
        seq_num: 1,
    };

    tracker.update_peer(hb.clone(), addr1, 1000);
    assert_eq!(tracker.peer_count(), 1);

    // Peer re-appears on a new IP (DHCP lease change)
    tracker.update_peer(hb, addr2, 1002);

    assert_eq!(tracker.peer_count(), 1); // Should overwrite, not duplicate
    let active = tracker.get_active_peers(1002, 10);
    assert_eq!(active[0].addr, addr2);

    Ok(())
}