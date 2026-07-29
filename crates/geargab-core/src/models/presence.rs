use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientType {
    Desktop,
    Gateway,
    LightingConsole,
    AudioConsole,
    ShowControl,
    MidiDevice,
    UnknownHardware,
}

impl Default for ClientType {
    fn default() -> Self {
        ClientType::UnknownHardware
    }
}

impl From<&str> for ClientType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "desktop" => ClientType::Desktop,
            "gateway" => ClientType::Gateway,
            "lighting_console" | "gma3" | "eos" => ClientType::LightingConsole,
            "audio_console" | "digico" | "yamaha" => ClientType::AudioConsole,
            "show_control" | "qlab" | "companion" => ClientType::ShowControl,
            "midi_device" | "midi" | "rtp_midi" => ClientType::MidiDevice,
            _ => ClientType::UnknownHardware,
        }
    }
}

impl fmt::Display for ClientType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ClientType::Desktop => "desktop",
            ClientType::Gateway => "gateway",
            ClientType::LightingConsole => "lighting_console",
            ClientType::AudioConsole => "audio_console",
            ClientType::ShowControl => "show_control",
            ClientType::MidiDevice => "midi_device",
            ClientType::UnknownHardware => "unknown_hardware",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Heartbeat {
    pub client_uuid: String,
    pub display_name: String,
    pub client_type: ClientType,
    pub timestamp: i64,
    pub seq_num: u64,
}