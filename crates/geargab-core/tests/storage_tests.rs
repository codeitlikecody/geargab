use geargab_core::models::{CanonicalMessage, ClientType, Heartbeat};
use geargab_core::storage::{initialize_schema, Repository};
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