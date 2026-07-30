use geargab_core::codec::DecodedPacket;
use geargab_core::models::ClientType;
use geargab_net::{HeartbeatEmitter, UdpListener, UdpSender};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_heartbeat_emitter_sequence_increment() -> Result<(), Box<dyn std::error::Error>> {
    let listener = UdpListener::bind("127.0.0.1:0").await?;
    let listener_addr = listener.local_addr()?;

    let sender = Arc::new(UdpSender::bind_any().await?);

    let emitter = HeartbeatEmitter::new(
        "hb-test-client".to_string(),
        "MA3 Desk".to_string(),
        ClientType::LightingConsole,
    );

    // Fast 50ms loop for test speed
    let (task_handle, shutdown_tx) = emitter.spawn_loop(
        sender,
        vec![listener_addr],
        Duration::from_millis(50),
    );

    // Receive first heartbeat
    let (packet1, _) = listener.recv_packet().await?;
    if let DecodedPacket::Heartbeat(hb1) = packet1 {
        assert_eq!(hb1.client_uuid, "hb-test-client");
        assert_eq!(hb1.seq_num, 1);
    } else {
        panic!("Expected Heartbeat packet");
    }

    // Receive second heartbeat
    let (packet2, _) = listener.recv_packet().await?;
    if let DecodedPacket::Heartbeat(hb2) = packet2 {
        assert_eq!(hb2.client_uuid, "hb-test-client");
        assert_eq!(hb2.seq_num, 2);
    } else {
        panic!("Expected Heartbeat packet");
    }

    // Shut down loop
    shutdown_tx.send(())?;
    let task_result = task_handle.await?;
    assert!(task_result.is_ok());

    Ok(())
}

#[tokio::test]
async fn test_heartbeat_emitter_graceful_shutdown() -> Result<(), Box<dyn std::error::Error>> {
    let listener = UdpListener::bind("127.0.0.1:0").await?;
    let listener_addr = listener.local_addr()?;
    let sender = Arc::new(UdpSender::bind_any().await?);

    let emitter = HeartbeatEmitter::new(
        "hb-shutdown-client".to_string(),
        "Digico Desk".to_string(),
        ClientType::AudioConsole,
    );

    let (task_handle, shutdown_tx) = emitter.spawn_loop(
        sender,
        vec![listener_addr],
        Duration::from_millis(100),
    );

    // Shut down immediately
    shutdown_tx.send(())?;

    // Task should join quickly without timing out
    let join_result = tokio::time::timeout(Duration::from_millis(500), task_handle).await?;
    assert!(join_result.is_ok());

    Ok(())
}