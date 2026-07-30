use geargab_core::codec::{decode_packet, DecodedPacket};
use geargab_core::error::GearGabError;
use rosc::decoder;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

pub struct UdpListener {
    socket: UdpSocket,
    buffer_size: usize,
}

impl UdpListener {
    /// Binds a UDP listener to the specified address string (e.g. `127.0.0.1:0` or `0.0.0.0:9000`).
    pub async fn bind(addr: &str) -> Result<Self, GearGabError> {
        let socket = UdpSocket::bind(addr)
            .await
            .map_err(|e| GearGabError::OscDecodeError(format!("Failed to bind listener socket: {e}")))?;
        socket
            .set_broadcast(true)
            .map_err(|e| GearGabError::OscDecodeError(format!("Failed to set broadcast flag: {e}")))?;

        Ok(Self {
            socket,
            buffer_size: 65535, // Max UDP payload size
        })
    }

    /// Receives a single packet, returning the decoded `DecodedPacket` and the sender's `SocketAddr`.
    /// Invalid OSC or corrupt packets return a `GearGabError::OscDecodeError`.
    pub async fn recv_packet(&self) -> Result<(DecodedPacket, SocketAddr), GearGabError> {
        let mut buf = vec![0u8; self.buffer_size];
        let (amt, src) = self
            .socket
            .recv_from(&mut buf)
            .await
            .map_err(|e| GearGabError::OscDecodeError(format!("UDP recv failed: {e}")))?;

        let (_, osc_packet) = decoder::decode_udp(&buf[..amt])
            .map_err(|e| GearGabError::OscDecodeError(format!("OSC binary decode failed: {e:?}")))?;

        let decoded = decode_packet(osc_packet)?;
        Ok((decoded, src))
    }

    pub fn local_addr(&self) -> Result<SocketAddr, GearGabError> {
        self.socket
            .local_addr()
            .map_err(|e| GearGabError::OscDecodeError(format!("Failed to get local addr: {e}")))
    }
}