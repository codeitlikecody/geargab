use crate::error::GearGabError;
use crate::models::{CanonicalMessage, ClientType, Heartbeat};
use rosc::{OscMessage, OscPacket, OscType};

#[derive(Debug, Clone, PartialEq)]
pub enum DecodedPacket {
    Message(CanonicalMessage),
    Heartbeat(Heartbeat),
    UnmatchedOsc {
        address: String,
        args_summary: String,
    },
}

/// Encodes a `CanonicalMessage` into a binary OSC packet.
/// Address format: `/messenger/v1/room/{room}/say`
pub fn encode_message(msg: &CanonicalMessage) -> OscPacket {
    OscPacket::Message(OscMessage {
        addr: format!("/messenger/v1/room/{}/say", msg.room),
        args: vec![
            OscType::String(msg.msg_uuid.clone()),
            OscType::String(msg.client_uuid.clone()),
            OscType::String(msg.username.clone()),
            OscType::String(msg.client_type.to_string()),
            OscType::String(msg.room.clone()),
            OscType::Long(msg.timestamp),
            OscType::String(msg.text.clone()),
        ],
    })
}

/// Encodes a `Heartbeat` into a binary OSC packet.
/// Address format: `/messenger/v1/heartbeat`
pub fn encode_heartbeat(hb: &Heartbeat) -> OscPacket {
    OscPacket::Message(OscMessage {
        addr: "/messenger/v1/heartbeat".to_string(),
        args: vec![
            OscType::String(hb.client_uuid.clone()),
            OscType::String(hb.display_name.clone()),
            OscType::String(hb.client_type.to_string()),
            OscType::Long(hb.timestamp),
            OscType::Long(hb.seq_num as i64),
        ],
    })
}

/// Decodes an incoming `OscPacket` into a `DecodedPacket`.
pub fn decode_packet(packet: OscPacket) -> Result<DecodedPacket, GearGabError> {
    match packet {
        OscPacket::Message(msg) => decode_message_struct(&msg),
        OscPacket::Bundle(bundle) => {
            if let Some(first) = bundle.content.into_iter().next() {
                decode_packet(first)
            } else {
                Err(GearGabError::OscDecodeError("Empty OSC bundle".to_string()))
            }
        }
    }
}

fn decode_message_struct(msg: &OscMessage) -> Result<DecodedPacket, GearGabError> {
    if msg.addr.starts_with("/messenger/v1/room/") && msg.addr.ends_with("/say") {
        if msg.args.len() < 7 {
            return Err(GearGabError::OscDecodeError(format!(
                "Invalid argument count for message: expected 7, got {}",
                msg.args.len()
            )));
        }

        let msg_uuid = extract_string(&msg.args[0])?;
        let client_uuid = extract_string(&msg.args[1])?;
        let username = extract_string(&msg.args[2])?;
        let client_type_raw = extract_string(&msg.args[3])?;
        let room = extract_string(&msg.args[4])?;
        let timestamp = extract_long(&msg.args[5])?;
        let text = extract_string(&msg.args[6])?;

        Ok(DecodedPacket::Message(CanonicalMessage {
            msg_uuid,
            client_uuid,
            username,
            client_type: ClientType::from(client_type_raw.as_str()),
            room,
            timestamp,
            text,
        }))
    } else if msg.addr == "/messenger/v1/heartbeat" {
        if msg.args.len() < 5 {
            return Err(GearGabError::OscDecodeError(format!(
                "Invalid argument count for heartbeat: expected 5, got {}",
                msg.args.len()
            )));
        }

        let client_uuid = extract_string(&msg.args[0])?;
        let display_name = extract_string(&msg.args[1])?;
        let client_type_raw = extract_string(&msg.args[2])?;
        let timestamp = extract_long(&msg.args[3])?;
        let seq_num = extract_long(&msg.args[4])? as u64;

        Ok(DecodedPacket::Heartbeat(Heartbeat {
            client_uuid,
            display_name,
            client_type: ClientType::from(client_type_raw.as_str()),
            timestamp,
            seq_num,
        }))
    } else {
        // Fallback: Arbitrary hardware desk/app OSC (e.g. /eos/out/event/cue, /qlab/cue/1/start)
        let mut args_summary = Vec::new();
        for arg in &msg.args {
            args_summary.push(format!("{:?}", arg));
        }

        Ok(DecodedPacket::UnmatchedOsc {
            address: msg.addr.clone(),
            args_summary: args_summary.join(" "),
        })
    }
}

fn extract_string(arg: &OscType) -> Result<String, GearGabError> {
    match arg {
        OscType::String(s) => Ok(s.clone()),
        _ => Err(GearGabError::OscDecodeError("Expected String argument".to_string())),
    }
}

fn extract_long(arg: &OscType) -> Result<i64, GearGabError> {
    match arg {
        OscType::Long(i) => Ok(*i),
        OscType::Int(i) => Ok(*i as i64),
        OscType::Float(f) => Ok(*f as i64),
        OscType::Double(d) => Ok(*d as i64),
        OscType::String(s) => s.parse::<i64>().map_err(|_| {
            GearGabError::OscDecodeError(format!("Failed to parse string '{s}' as integer"))
        }),
        _ => Err(GearGabError::OscDecodeError(
            "Expected numeric argument (Long/Int/Float/Double/Numeric String)".to_string(),
        )),
    }
}