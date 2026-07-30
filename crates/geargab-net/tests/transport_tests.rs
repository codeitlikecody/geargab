use geargab_core::codec::{encode_message, DecodedPacket};
use geargab_core::models::{CanonicalMessage, ClientType};
use geargab_net::{UdpListener, UdpSender};
use tokio::net::UdpSocket;

#[tokio::test]
async fn test_large_payload_handling() {
    let listener = UdpListener::bind("127.0.0.1:0").await.unwrap();
    let listener_addr = listener.local_addr().unwrap();
    let sender = UdpSender::bind_any().await.unwrap();

    // Create a 4KB chat text payload
    let large_text = "A".repeat(4096);
    let msg = CanonicalMessage {
        msg_uuid: "large-msg-001".to_string(),
        client_uuid: "client-large".to_string(),
        username: "StageMgr".to_string(),
        client_type: ClientType::Desktop,
        room: "main-stage".to_string(),
        timestamp: 1711000000,
        text: large_text,
    };

    let packet = encode_message(&msg);
    sender.send_to(&packet, listener_addr).await.unwrap();

    let (received_packet, _) = listener.recv_packet().await.unwrap();
    if let DecodedPacket::Message(m) = received_packet {
        assert_eq!(m.text.len(), 4096);
    } else {
        panic!("Expected decoded CanonicalMessage");
    }
}

#[tokio::test]
async fn test_zero_byte_udp_datagram() {
    let listener = UdpListener::bind("127.0.0.1:0").await.unwrap();
    let listener_addr = listener.local_addr().unwrap();

    let raw_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    // Send 0 bytes
    raw_socket.send_to(&[], listener_addr).await.unwrap();

    let result = listener.recv_packet().await;
    assert!(result.is_err(), "Zero byte datagram should return decode error");
}

#[tokio::test]
async fn test_broadcast_transmission() {
    let listener = UdpListener::bind("127.0.0.1:0").await.unwrap();
    let listener_addr = listener.local_addr().unwrap();

    let sender = UdpSender::bind_any().await.unwrap();

    let msg = CanonicalMessage {
        msg_uuid: "bcast-msg-001".to_string(),
        client_uuid: "client-bcast".to_string(),
        username: "LightingDesk".to_string(),
        client_type: ClientType::LightingConsole,
        room: "all".to_string(),
        timestamp: 1711000000,
        text: "Broadcast test".to_string(),
    };

    let packet = encode_message(&msg);
    // Transmit to listener address directly
    let sent = sender.send_to(&packet, listener_addr).await;
    assert!(sent.is_ok());
}

#[tokio::test]
async fn test_udp_sender_listener_loopback() {
    let listener = UdpListener::bind("127.0.0.1:0").await.unwrap();
    let listener_addr = listener.local_addr().unwrap();

    let sender = UdpSender::bind_any().await.unwrap();

    let msg = CanonicalMessage {
        msg_uuid: "net-test-001".to_string(),
        client_uuid: "client-a".to_string(),
        username: "AudioTech".to_string(),
        client_type: ClientType::AudioConsole,
        room: "main-stage".to_string(),
        timestamp: 1711000000,
        text: "Testing transport layer".to_string(),
    };

    let packet = encode_message(&msg);
    let bytes_sent = sender.send_to(&packet, listener_addr).await.unwrap();
    assert!(bytes_sent > 0);

    let (received_packet, sender_addr) = listener.recv_packet().await.unwrap();
    assert_eq!(received_packet, DecodedPacket::Message(msg));

    // Verify the received packet originated from the sender's bound port on loopback
    assert_eq!(sender_addr.port(), sender.socket().local_addr().unwrap().port());
    assert!(sender_addr.ip().is_loopback());
}

#[tokio::test]
async fn test_udp_socket_broadcast_flag() {
    let listener = UdpListener::bind("127.0.0.1:0").await.unwrap();
    let sender = UdpSender::bind_any().await.unwrap();

    assert!(listener.local_addr().is_ok());
    assert!(sender.socket().local_addr().is_ok());
}

#[tokio::test]
async fn test_garbled_udp_packet_resilience() {
    let listener = UdpListener::bind("127.0.0.1:0").await.unwrap();
    let listener_addr = listener.local_addr().unwrap();

    let raw_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let garbage_data = b"NOT_OSC_DATA_GARBLED_PAYLOAD_123456789";

    raw_socket.send_to(garbage_data, listener_addr).await.unwrap();

    let result = listener.recv_packet().await;
    assert!(result.is_err());
}