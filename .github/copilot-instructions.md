# AION-Transporter — Copilot instructions

Purpose: Give AI coding agents the essential, actionable knowledge to be productive in this repo.

Big picture
- High-performance Rust toolkit for distributed network transport: multicast messaging, QUIC client/server, and certificate generation.
- Provides both library and CLI interface for inter-node communication in the AION ecosystem.
- Designed for experimentation, research, and distributed systems prototyping.

Key files & where to look
- src/main.rs: CLI entry point with subcommands (serve, multicast)
- src/lib.rs: Module declarations (multicast, quic, certificate)
- src/multicast/mod.rs: UDP broadcast implementation
- src/quic/: QUIC client/server with rustls encryption
- src/certificate/pem.rs: Self-signed certificate generation

Build & run (reproducible commands)
- Build: `cargo build --release`
- QUIC server: `CERT_PATH=./cert.pem KEY_PATH=./key.pem aion-transporter serve --port 4433 true`
- Multicast listen: `aion-transporter multicast`

Project-specific conventions (do not change silently)
- Multicast group: Hardcoded to 239.255.255.250:9999 (IPv4 all-systems multicast)
- Certificate: Self-signed localhost certs for development
- Async callbacks: Generic closures for message handling (decoupled design)
- Environment vars: CERT_PATH, KEY_PATH for QUIC server

Integration points & expectations
- Used by AION-Agent for agent discovery and state broadcasts
- QUIC init: Call aion_transporter::quic::init() before any QUIC operations
- Certificate verification: Disabled in client for local testing (unsafe for production)

Safe edits checklist for feature changes
- When adding new transport protocols: Follow module pattern (new mod in lib.rs, CLI subcommand in main.rs)
- When modifying QUIC: Ensure rustls provider init is called once per process
- When changing multicast: Test with multiple nodes for broadcast reliability

Quick debugging tips
- QUIC connections: Check CERT_PATH/KEY_PATH environment variables
- Multicast: Verify UDP port 9999 is not blocked by firewall
- Certificates: Use `openssl x509 -in cert.pem -text` to inspect generated certs</content>
<parameter name="filePath">/media/elpixeler/Develop/Projects/AION/AION-Transporter/.github/copilot-instructions.md