use geargab_core::codec::encode_heartbeat;
use geargab_core::error::GearGabError;
use geargab_core::models::{ClientType, Heartbeat};
use crate::sender::UdpSender;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tokio::time::interval;

pub struct HeartbeatEmitter {
    client_uuid: String,
    display_name: String,
    client_type: ClientType,
    seq_num: Arc<AtomicU64>,
}

impl HeartbeatEmitter {
    pub fn new(client_uuid: String, display_name: String, client_type: ClientType) -> Self {
        Self {
            client_uuid,
            display_name,
            client_type,
            seq_num: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Spawns an async loop task that periodically transmits a Heartbeat packet to target addresses.
    ///
    /// Returns a `JoinHandle` for the task and a shutdown sender channel. Sending a signal
    /// on `shutdown_tx` will terminate the heartbeat loop gracefully.
    pub fn spawn_loop(
        &self,
        sender: Arc<UdpSender>,
        targets: Vec<SocketAddr>,
        period: Duration,
    ) -> (JoinHandle<Result<(), GearGabError>>, broadcast::Sender<()>) {
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel::<()>(1);

        let client_uuid = self.client_uuid.clone();
        let display_name = self.display_name.clone();
        let client_type = self.client_type.clone();
        let seq_num = Arc::clone(&self.seq_num);

        let handle = tokio::spawn(async move {
            let mut timer = interval(period);
            // Skip immediate first tick to align initial delay if needed, or fire immediately
            timer.tick().await;

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                    _ = timer.tick() => {
                        let current_seq = seq_num.fetch_add(1, Ordering::SeqCst);
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64;

                        let hb = Heartbeat {
                            client_uuid: client_uuid.clone(),
                            display_name: display_name.clone(),
                            client_type: client_type.clone(),
                            timestamp: now,
                            seq_num: current_seq,
                        };

                        let packet = encode_heartbeat(&hb);
                        for target in &targets {
                            let _ = sender.send_to(&packet, *target).await;
                        }
                    }
                }
            }

            Ok(())
        });

        (handle, shutdown_tx)
    }

    /// Returns the current sequence number value.
    pub fn current_seq(&self) -> u64 {
        self.seq_num.load(Ordering::SeqCst)
    }
}