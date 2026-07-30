use geargab_core::codec::osc::{decode_packet, encode_heartbeat, encode_message, DecodedPacket};
use geargab_core::models::{CanonicalMessage, ClientType, Heartbeat};
use geargab_core::error::GearGabError;
use geargab_core::codec::json::{heartbeat_from_json, message_from_json};

use rosc::{OscMessage, OscBundle, OscPacket, OscType};

#[test]
fn test_message_roundtrip() {
    let msg = CanonicalMessage {
        msg_uuid: "123e4567-e89b-12d3-a456-426614174000".to_string(),
        client_uuid: "c83e4567-e89b-12d3-a456-426614174001".to_string(),
        username: "StageMgr".to_string(),
        client_type: ClientType::Desktop,
        room: "stage-ops".to_string(),
        timestamp: 1711000000,
        text: "Standby Cue 12".to_string(),
    };

    let packet = encode_message(&msg);
    let decoded = decode_packet(packet).unwrap();

    assert_eq!(decoded, DecodedPacket::Message(msg));
}

#[test]
fn test_heartbeat_roundtrip() {
    let hb = Heartbeat {
        client_uuid: "c83e4567-e89b-12d3-a456-426614174001".to_string(),
        display_name: "Lighting Desk".to_string(),
        client_type: ClientType::LightingConsole,
        timestamp: 1711000000,
        seq_num: 42,
    };

    let packet = encode_heartbeat(&hb);
    let decoded = decode_packet(packet).unwrap();

    assert_eq!(decoded, DecodedPacket::Heartbeat(hb));
}

#[test]
fn test_unmatched_hardware_osc() {
    let raw_packet = OscPacket::Message(OscMessage {
        addr: "/eos/out/event/cue/1/fire".to_string(),
        args: vec![OscType::Float(1.0)],
    });

    let decoded = decode_packet(raw_packet).unwrap();

    match decoded {
        DecodedPacket::UnmatchedOsc { address, args_summary } => {
            assert_eq!(address, "/eos/out/event/cue/1/fire");
            assert!(args_summary.contains("Float(1.0)"));
        }
        _ => panic!("Expected UnmatchedOsc variant"),
    }
}

#[test]
fn test_numeric_type_resilience_in_osc() {
    // Test parsing when timestamp/seq_num are encoded as Int or Float
    let packet = OscPacket::Message(OscMessage {
        addr: "/messenger/v1/heartbeat".to_string(),
        args: vec![
            OscType::String("client-123".to_string()),
            OscType::String("Eos Console".to_string()),
            OscType::String("eos".to_string()),
            OscType::Double(1711000000.0), // Sent as double
            OscType::Int(10),             // Sent as int
        ],
    });

    let decoded = decode_packet(packet).unwrap();
    if let DecodedPacket::Heartbeat(hb) = decoded {
        assert_eq!(hb.timestamp, 1711000000);
        assert_eq!(hb.seq_num, 10);
    } else {
        panic!("Failed to decode packet with float/int fallback");
    }
}

#[test]
fn test_malformed_osc_argument_count() {
    // Insufficient arguments for say message
    let packet = OscPacket::Message(OscMessage {
        addr: "/messenger/v1/room/main/say".to_string(),
        args: vec![
            OscType::String("msg-uuid".to_string()),
            OscType::String("client-uuid".to_string()),
        ],
    });

    let res = decode_packet(packet);
    assert!(matches!(res, Err(GearGabError::OscDecodeError(_))));
}

#[test]
fn test_malformed_osc_argument_types() {
    // Wrong type for timestamp (string instead of numeric)
    let packet = OscPacket::Message(OscMessage {
        addr: "/messenger/v1/heartbeat".to_string(),
        args: vec![
            OscType::String("client-123".to_string()),
            OscType::String("Eos Console".to_string()),
            OscType::String("eos".to_string()),
            OscType::String("not-a-number".to_string()),
            OscType::Int(10),
        ],
    });

    let res = decode_packet(packet);
    assert!(matches!(res, Err(GearGabError::OscDecodeError(_))));
}

#[test]
fn test_osc_bundle_wrapping() {
    // Stage software like QLab often wraps messages inside bundles
    let inner_msg = OscMessage {
        addr: "/messenger/v1/heartbeat".to_string(),
        args: vec![
            OscType::String("client-999".to_string()),
            OscType::String("QLab Rig".to_string()),
            OscType::String("qlab".to_string()),
            OscType::Long(1711000000),
            OscType::Long(1),
        ],
    };

    let bundle_packet = OscPacket::Bundle(OscBundle {
        timetag: rosc::OscTime { seconds: 0, fractional: 0 },
        content: vec![OscPacket::Message(inner_msg)],
    });

    let decoded = decode_packet(bundle_packet).unwrap();
    if let DecodedPacket::Heartbeat(hb) = decoded {
        assert_eq!(hb.client_uuid, "client-999");
        assert_eq!(hb.display_name, "QLab Rig");
    } else {
        panic!("Failed to unpack message inside OSC bundle");
    }
}

#[test]
fn test_empty_osc_bundle_error() {
    let empty_bundle = OscPacket::Bundle(OscBundle {
        timetag: rosc::OscTime { seconds: 0, fractional: 0 },
        content: vec![],
    });

    let res = decode_packet(empty_bundle);
    assert!(matches!(res, Err(GearGabError::OscDecodeError(_))));
}

#[test]
fn test_empty_and_whitespace_chat_fields() {
    // Senders transmitting empty chat text or room fallback behavior
    let msg = CanonicalMessage {
        msg_uuid: "uuid-empty-text".to_string(),
        client_uuid: "client-001".to_string(),
        username: "   ".to_string(), // Whitespace username
        client_type: ClientType::UnknownHardware,
        room: "general".to_string(),
        timestamp: 0,
        text: "".to_string(), // Empty chat body
    };

    let packet = encode_message(&msg);
    let decoded = decode_packet(packet).unwrap();
    assert_eq!(decoded, DecodedPacket::Message(msg));
}

#[test]
fn test_malformed_json_decoding() {
    let bad_json = r#"{"msg_uuid": "123", "corrupted": true}"#;
    assert!(message_from_json(bad_json).is_err());
    assert!(heartbeat_from_json(bad_json).is_err());
}

#[test]
fn test_unicode_and_emoji_payloads() {
    let msg = CanonicalMessage {
        msg_uuid: "uuid-emoji-001".to_string(),
        client_uuid: "client-001".to_string(),
        username: "Lighting⚡Tech".to_string(),
        client_type: ClientType::LightingConsole,
        room: "stage-ops".to_string(),
        timestamp: 1711000000,
        text: "Standby Cue 42! 🎭✨🔥".to_string(),
    };

    let packet = encode_message(&msg);
    let decoded = decode_packet(packet).unwrap();
    assert_eq!(decoded, DecodedPacket::Message(msg));
}

#[test]
fn test_numeric_string_fallback_in_osc() {
    // Console sends timestamp/sequence as string primitives ("1711000000", "42")
    let packet = OscPacket::Message(OscMessage {
        addr: "/messenger/v1/heartbeat".to_string(),
        args: vec![
            OscType::String("client-str-num".to_string()),
            OscType::String("DiGiCo Desk".to_string()),
            OscType::String("digico".to_string()),
            OscType::String("1711000000".to_string()), // String timestamp
            OscType::String("42".to_string()),         // String sequence number
        ],
    });

    let decoded = decode_packet(packet).unwrap();
    if let DecodedPacket::Heartbeat(hb) = decoded {
        assert_eq!(hb.timestamp, 1711000000);
        assert_eq!(hb.seq_num, 42);
    } else {
        panic!("Failed to parse string-encoded numbers");
    }
}

#[test]
fn test_negative_timestamps_and_sequence_numbers() {
    let hb = Heartbeat {
        client_uuid: "client-neg".to_string(),
        display_name: "Test Desk".to_string(),
        client_type: ClientType::Desktop,
        timestamp: -500, // Pre-1970 timestamp
        seq_num: 0,
    };

    let packet = encode_heartbeat(&hb);
    let decoded = decode_packet(packet).unwrap();
    assert_eq!(decoded, DecodedPacket::Heartbeat(hb));
}

#[test]
fn test_osc_path_injection_characters() {
    // Test addresses containing potential OSC wildcards or quotes
    let raw_packet = OscPacket::Message(OscMessage {
        addr: "/eos/out/cue/'1'/fire;DROP TABLE--".to_string(),
        args: vec![OscType::String("test_quote'_and_slash".to_string())],
    });

    let decoded = decode_packet(raw_packet).unwrap();
    match decoded {
        DecodedPacket::UnmatchedOsc { address, args_summary } => {
            assert_eq!(address, "/eos/out/cue/'1'/fire;DROP TABLE--");
            assert!(args_summary.contains("test_quote'_and_slash"));
        }
        _ => panic!("Expected UnmatchedOsc variant"),
    }
}