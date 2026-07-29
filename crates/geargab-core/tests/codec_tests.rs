use geargab_core::codec::osc::{decode_packet, encode_heartbeat, encode_message, DecodedPacket};
use geargab_core::models::{CanonicalMessage, ClientType, Heartbeat};
use rosc::{OscMessage, OscPacket, OscType};

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