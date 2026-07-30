use geargab_core::models::{CanonicalMessage, ClientType, Heartbeat};
use geargab_core::storage::{initialize_schema, Repository};
use geargab_core::models::HardwareEvent;

use rusqlite::Connection;

fn setup_in_memory_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    initialize_schema(&conn).unwrap();
    conn
}

#[test]
fn test_message_idempotent_insertion() {
    let conn = setup_in_memory_db();
    let repo = Repository::new(&conn);

    let msg = CanonicalMessage {
        msg_uuid: "unique-msg-001".to_string(),
        client_uuid: "node-a".to_string(),
        username: "SoundGuy".to_string(),
        client_type: ClientType::AudioConsole,
        room: "stage-ops".to_string(),
        timestamp: 1000,
        text: "Testing audio mic 1".to_string(),
    };

    // First insert succeeds
    let inserted = repo.insert_message(&msg).unwrap();
    assert!(inserted);

    // Duplicate insert on same msg_uuid is silently ignored
    let duplicate = repo.insert_message(&msg).unwrap();
    assert!(!duplicate);

    let history = repo.fetch_recent_history("stage-ops", 10).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].text, "Testing audio mic 1");
}

#[test]
fn test_peer_presence_upsert() {
    let conn = setup_in_memory_db();
    let repo = Repository::new(&conn);

    let hb = Heartbeat {
        client_uuid: "desk-01".to_string(),
        display_name: "gMA3 Desk".to_string(),
        client_type: ClientType::LightingConsole,
        timestamp: 5000,
        seq_num: 1,
    };

    repo.upsert_peer(&hb).unwrap();

    let active = repo.fetch_active_peers(5000, 10).unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].display_name, "gMA3 Desk");

    // Peer times out if current time is way ahead
    let active_later = repo.fetch_active_peers(5020, 10).unwrap();
    assert_eq!(active_later.len(), 0);
}

#[test]
fn test_hardware_event_persistence() {
    let conn = setup_in_memory_db();
    let repo = Repository::new(&conn);

    let event1 = HardwareEvent {
        source_ip: "192.168.1.50".to_string(),
        raw_address: "/eos/out/event/cue/1/fire".to_string(),
        arguments_summary: "Float(1.0)".to_string(),
        timestamp: 1711000100,
    };

    let event2 = HardwareEvent {
        source_ip: "192.168.1.51".to_string(),
        raw_address: "/qlab/workspace/active/go".to_string(),
        arguments_summary: "".to_string(),
        timestamp: 1711000105,
    };

    let row_id_1 = repo.insert_hardware_event(&event1).unwrap();
    let row_id_2 = repo.insert_hardware_event(&event2).unwrap();

    assert!(row_id_1 > 0);
    assert!(row_id_2 > row_id_1);

    let fetched = repo.fetch_recent_hardware_events(10).unwrap();
    assert_eq!(fetched.len(), 2);
    assert_eq!(fetched[0].source_ip, "192.168.1.50");
    assert_eq!(fetched[1].raw_address, "/qlab/workspace/active/go");
}

#[test]
fn test_large_history_limit_and_empty_room() {
    let conn = setup_in_memory_db();
    let repo = Repository::new(&conn);

    // Fetching from a room with no messages returns empty Vec without error
    let history = repo.fetch_recent_history("nonexistent-room", 50).unwrap();
    assert!(history.is_empty());

    // Requesting limit 0 returns empty Vec
    let history_zero = repo.fetch_recent_history("stage-ops", 0).unwrap();
    assert!(history_zero.is_empty());
}

#[test]
fn test_hardware_event_empty_summary() {
    let conn = setup_in_memory_db();
    let repo = Repository::new(&conn);

    let event = HardwareEvent {
        source_ip: "10.0.0.1".to_string(),
        raw_address: "/eos/out/ping".to_string(),
        arguments_summary: "".to_string(), // Hardware sent no args
        timestamp: 1711000500,
    };

    assert!(repo.insert_hardware_event(&event).is_ok());
    let fetched = repo.fetch_recent_hardware_events(1).unwrap();
    assert_eq!(fetched.len(), 1);
    assert_eq!(fetched[0].arguments_summary, "");
}

#[test]
fn test_sql_injection_and_quotes_persistence() {
    let conn = setup_in_memory_db();
    let repo = Repository::new(&conn);

    let msg = CanonicalMessage {
        msg_uuid: "uuid-injection-001".to_string(),
        client_uuid: "client-001".to_string(),
        username: "O'Connor'; DROP TABLE messages; --".to_string(),
        client_type: ClientType::Desktop,
        room: "room'OR'1'='1".to_string(),
        timestamp: 1711000000,
        text: "Testing 'single quotes' and \"double quotes\" and ; semicolons".to_string(),
    };

    // Insert should safely execute parameterized query
    assert!(repo.insert_message(&msg).unwrap());

    // Retrieve by literal room name
    let fetched = repo.fetch_recent_history("room'OR'1'='1", 10).unwrap();
    assert_eq!(fetched.len(), 1);
    assert_eq!(fetched[0].username, "O'Connor'; DROP TABLE messages; --");
    assert_eq!(
        fetched[0].text,
        "Testing 'single quotes' and \"double quotes\" and ; semicolons"
    );
}

#[test]
fn test_peer_presence_sequence_wrap_around() {
    let conn = setup_in_memory_db();
    let repo = Repository::new(&conn);

    // Initial heartbeat near u64 boundary
    let hb1 = Heartbeat {
        client_uuid: "desk-boundary".to_string(),
        display_name: "MA3 Fullsize".to_string(),
        client_type: ClientType::LightingConsole,
        timestamp: 1000,
        seq_num: u64::MAX - 1,
    };

    repo.upsert_peer(&hb1).unwrap();

    let active = repo.fetch_active_peers(1000, 10).unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].seq_num, u64::MAX - 1);

    // Update with wrapped sequence number (e.g. after system reboot)
    let hb2 = Heartbeat {
        client_uuid: "desk-boundary".to_string(),
        display_name: "MA3 Fullsize".to_string(),
        client_type: ClientType::LightingConsole,
        timestamp: 1005,
        seq_num: 0,
    };

    repo.upsert_peer(&hb2).unwrap();

    let updated = repo.fetch_active_peers(1005, 10).unwrap();
    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0].seq_num, 0);
}