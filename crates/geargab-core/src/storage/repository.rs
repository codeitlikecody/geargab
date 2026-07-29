use crate::error::GearGabError;
use crate::models::{CanonicalMessage, ClientType, Heartbeat};
use rusqlite::{params, Connection};

pub struct Repository<'a> {
    conn: &'a Connection,
}

impl<'a> Repository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Idempotently inserts a message. Returns `true` if inserted, `false` if ignored as a duplicate.
    pub fn insert_message(&self, msg: &CanonicalMessage) -> Result<bool, GearGabError> {
        let inserted = self.conn.execute(
            "INSERT OR IGNORE INTO messages (msg_uuid, client_uuid, username, client_type, room, timestamp, text)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                msg.msg_uuid,
                msg.client_uuid,
                msg.username,
                msg.client_type.to_string(),
                msg.room,
                msg.timestamp,
                msg.text
            ],
        )?;

        Ok(inserted > 0)
    }

    /// Fetches recent chat history for a specific room ordered by timestamp ascending.
    pub fn fetch_recent_history(
        &self,
        room: &str,
        limit: usize,
    ) -> Result<Vec<CanonicalMessage>, GearGabError> {
        let mut stmt = self.conn.prepare(
            "SELECT msg_uuid, client_uuid, username, client_type, room, timestamp, text
             FROM messages
             WHERE room = ?1
             ORDER BY timestamp DESC, rowid DESC
             LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![room, limit as i64], |row| {
            let client_type_str: String = row.get(3)?;
            Ok(CanonicalMessage {
                msg_uuid: row.get(0)?,
                client_uuid: row.get(1)?,
                username: row.get(2)?,
                client_type: ClientType::from(client_type_str.as_str()),
                room: row.get(4)?,
                timestamp: row.get(5)?,
                text: row.get(6)?,
            })
        })?;

        let mut messages = Vec::new();
        for msg_result in rows {
            messages.push(msg_result?);
        }

        // Reverse so the caller receives chronological order (oldest to newest)
        messages.reverse();
        Ok(messages)
    }

    /// Updates peer presence from a received heartbeat.
    pub fn upsert_peer(&self, hb: &Heartbeat) -> Result<(), GearGabError> {
        self.conn.execute(
            "INSERT INTO peers (client_uuid, display_name, client_type, last_seen, last_seq_num)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(client_uuid) DO UPDATE SET
                 display_name = excluded.display_name,
                 client_type = excluded.client_type,
                 last_seen = excluded.last_seen,
                 last_seq_num = excluded.last_seq_num",
            params![
                hb.client_uuid,
                hb.display_name,
                hb.client_type.to_string(),
                hb.timestamp,
                hb.seq_num as i64
            ],
        )?;

        Ok(())
    }

    /// Retrieves active peers seen within `timeout_seconds` from `now`.
    pub fn fetch_active_peers(
        &self,
        now: i64,
        timeout_seconds: i64,
    ) -> Result<Vec<Heartbeat>, GearGabError> {
        let cutoff = now - timeout_seconds;
        let mut stmt = self.conn.prepare(
            "SELECT client_uuid, display_name, client_type, last_seen, last_seq_num
             FROM peers
             WHERE last_seen >= ?1
             ORDER BY display_name ASC",
        )?;

        let rows = stmt.query_map(params![cutoff], |row| {
            let client_type_str: String = row.get(2)?;
            let seq_num_i64: i64 = row.get(4)?;
            Ok(Heartbeat {
                client_uuid: row.get(0)?,
                display_name: row.get(1)?,
                client_type: ClientType::from(client_type_str.as_str()),
                timestamp: row.get(3)?,
                seq_num: seq_num_i64 as u64,
            })
        })?;

        let mut peers = Vec::new();
        for peer_result in rows {
            peers.push(peer_result?);
        }

        Ok(peers)
    }
}