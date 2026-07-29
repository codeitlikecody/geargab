use rusqlite::{Connection, Result};

pub fn initialize_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS messages (
            msg_uuid TEXT PRIMARY KEY,
            client_uuid TEXT NOT NULL,
            username TEXT NOT NULL,
            client_type TEXT NOT NULL,
            room TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            text TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_messages_room_ts 
            ON messages(room, timestamp DESC);

        CREATE TABLE IF NOT EXISTS peers (
            client_uuid TEXT PRIMARY KEY,
            display_name TEXT NOT NULL,
            client_type TEXT NOT NULL,
            last_seen INTEGER NOT NULL,
            last_seq_num INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_peers_last_seen 
            ON peers(last_seen DESC);
        ",
    )
}