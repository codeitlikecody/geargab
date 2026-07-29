# GearGab

GearGab is a decentralized, serverless chat and log aggregation tool for live production teams working in theater, live music, broadcast, and corporate AV.

It gives production crews a lightweight, zero-configuration local chat client for human-to-human communication alongside automated log capturing from hardware consoles—all running over local network switches without needing internet or a central server.

## Overview

Live events often rely on crew members using separate radios, intercoms, or personal messaging apps that fail without internet. GearGab provides a single, local network chat workspace that runs across laptop screens, tablets, and web browsers on the show network.

* **Human-to-Human Production Chat:** Real-time channel-based messaging (e.g., `#stage-ops`, `#audio`, `#lighting`) so stage managers, operators, and technicians can send clear text updates, standby alerts, and cue notes.
* **Console & Hardware Log Interleaving:** Incoming OSC traffic from equipment (ETC Eos, grandMA3, QLab, Bitfocus Companion, audio desks) is converted into readable log entries and routed directly into chat channels alongside human messages.
* **Zero-Config Mesh:** Nodes discover each other automatically via local UDP multicast. No server configuration or IP management required—just connect to the show network and start talking.
* **Offline Local Storage:** Messages and logs are saved to an embedded local SQLite database, allowing late-joining nodes to sync history instantly.

## Protocol & Specification

The core messaging schema is built on top of Open Sound Control (OSC) over UDP multicast (`239.254.0.1:3090`).

The address layout and envelope design are inspired by and compatible with the specification in [ETCLabs/OSCMessenger](https://github.com/ETCLabs/OSCMessenger).

* **Chat Protocol:** Standardized `/messenger/v1/room/{room}/say` path structure carrying human messages and metadata.
* **Presence:** `/messenger/v1/heartbeat` for local mesh discovery and active node tracking.
* **Hardware Capture:** Fallback parser that captures raw OSC triggers from show control software and lighting consoles, formatting them as readable system events in chat channels.

## System Architecture

The project is structured as a Rust Cargo workspace:

* `crates/geargab-core`: Domain models for human chat & hardware logs, OSC/JSON codecs, and SQLite storage.
* `crates/geargab-net`: Multi-interface UDP multicast engine and peer presence tracking.
* `crates/geargab-gateway`: Axum-based WebSocket bridge and local web UI server.

## Getting Started

### Prerequisites

* Rust 1.75+ (2021 edition)

### Build and Test

```bash
# Clone the repository
git clone [https://github.com/codeitlikecody/geargab.git](https://github.com/codeitlikecody/geargab.git)
cd geargab

# Run project check
cargo check

# Run tests
cargo test
