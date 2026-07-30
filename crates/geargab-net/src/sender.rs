use geargab_core::error::GearGabError;
use rosc::{encoder, OscPacket};
use std::net::SocketAddr;
use tokio::net::UdpSocket;

pub struct UdpSender {
    socket: UdpSocket,
}

impl UdpSender {
    /// Creates a new `UdpSender` bound to an arbitrary local address (e.g., `0.0.0.0:0`).
    pub async fn bind_any() -> Result<Self, GearGabError> {
        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| GearGabError::OscEncodeError(format!("Failed to bind UDP sender socket: {e}")))?;
        socket
            .set_broadcast(true)
            .map_err(|e| GearGabError::OscEncodeError(format!("Failed to set broadcast flag: {e}")))?;

        Ok(Self { socket })
    }

    /// Creates a `UdpSender` wrapping an existing `UdpSocket`.
    pub fn new(socket: UdpSocket) -> Self {
        Self { socket }
    }

    /// Encodes and transmits an `OscPacket` to a specified `SocketAddr`.
    pub async fn send_to(&self, packet: &OscPacket, target: SocketAddr) -> Result<usize, GearGabError> {
        let bytes = encoder::encode(packet)
            .map_err(|e| GearGabError::OscEncodeError(format!("OSC encoding failed: {e}")))?;

        self.socket
            .send_to(&bytes, target)
            .await
            .map_err(|e| GearGabError::OscEncodeError(format!("Failed to send UDP datagram: {e}")))
    }

    /// Returns a reference to the underlying socket.
    pub fn socket(&self) -> &UdpSocket {
        &self.socket
    }
}