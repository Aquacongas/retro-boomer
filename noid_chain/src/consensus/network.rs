// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Paranoid Zero.

//! Network configuration for Retro Boomer mainnet.
//!
//! Every network participant shares these magic bytes, ports, protocol ID,
//! and gossipsub topics. libp2p protocol IDs prevent cross-network connections.
//!
//! # Ports (no conflicts with Bitcoin 8333/8332, Ethereum 30303/8545, Monero 18080/18081)
//!
//! | Network  | P2P   | RPC   |
//! |----------|-------|-------|
//! | Mainnet  | 19600  | 19601  |
//!
//! # Magic bytes
//!
//! | Network  | Magic (ASCII) |
//! |----------|---------------|
//! | Mainnet  | 0x5254424D "RTBM" |

use std::str::FromStr;

// ---------------------------------------------------------------------------
// NetworkKind
// ---------------------------------------------------------------------------

/// Which network this node participates in.
///
/// Only mainnet is supported by this release. Attempting to parse
/// any other string returns
/// an error at startup so misconfigured nodes fail fast.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NetworkKind {
    /// Retro Boomer mainnet.
    #[default]
    Mainnet,
}

impl NetworkKind {
    pub fn as_str(&self) -> &'static str {
        "mainnet"
    }

    pub fn is_mainnet(&self) -> bool {
        true
    }
}

impl std::fmt::Display for NetworkKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for NetworkKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mainnet" => Ok(Self::Mainnet),
            other => Err(format!(
                "unknown network '{other}'; only 'mainnet' is supported"
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// NetworkConfig
// ---------------------------------------------------------------------------

/// All network-specific runtime constants.
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub kind: NetworkKind,

    /// Four-byte project marker. Active libp2p isolation uses protocol IDs and the network profile.
    pub magic: [u8; 4],

    /// Default P2P listening port.
    pub default_p2p_port: u16,

    /// Default JSON-RPC listening port.
    pub default_rpc_port: u16,

    /// libp2p protocol identifier string.
    /// Nodes refuse connections from peers using a different protocol ID.
    pub p2p_protocol_id: &'static str,

    /// Gossipsub topic for new block announcements.
    pub topic_blocks: &'static str,

    /// Gossipsub topic for new TxIntent announcements.
    pub topic_txs: &'static str,

    /// DNS seeds for peer discovery.
    pub dns_seeds: &'static [&'static str],
}

impl NetworkConfig {
    pub fn mainnet() -> Self {
        Self {
            kind: NetworkKind::Mainnet,
            magic: *b"RTBM", // Four-byte Retro Boomer project marker.
            default_p2p_port: 19600,
            default_rpc_port: 19601,
            // Mainnet is isolated by its genesis-bound namespace. The exact
            // authenticated network profile adds a second fail-closed gate.
            p2p_protocol_id: "/retro-boomer/mainnet/cede1246070229c7/1",
            topic_blocks: "/retro-boomer/mainnet/cede1246070229c7/blocks/1",
            topic_txs: "/retro-boomer/mainnet/cede1246070229c7/txs/1",
            // Official hostnames use the existing system DNS and libp2p
            // /dns/<hostname>/tcp/<default_port> resolution, including A/AAAA.
            // Configured peers remain additive; dnsaddr syntax stays available
            // for manual peers. Unavailable seeds use the existing retry policy.
            dns_seeds: &["seed1.retroboomer.org", "seed2.retroboomer.org"],
        }
    }

    pub fn for_kind(_kind: NetworkKind) -> Self {
        Self::mainnet()
    }

    /// Default P2P listen address as a libp2p multiaddr string.
    pub fn default_p2p_listen(&self) -> String {
        format!("/ip4/0.0.0.0/tcp/{}", self.default_p2p_port)
    }

    /// Default RPC listen address.
    pub fn default_rpc_listen(&self) -> String {
        format!("127.0.0.1:{}", self.default_rpc_port)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mainnet_magic_is_retro_boomer() {
        assert_eq!(NetworkConfig::mainnet().magic, *b"RTBM");
    }

    #[test]
    fn mainnet_ports() {
        let m = NetworkConfig::mainnet();
        assert_eq!(m.default_p2p_port, 19600);
        assert_eq!(m.default_rpc_port, 19601);
    }

    #[test]
    fn mainnet_protocol_id() {
        let mainnet = NetworkConfig::mainnet();
        assert_eq!(mainnet.p2p_protocol_id, "/retro-boomer/mainnet/cede1246070229c7/1");
        assert_eq!(
            mainnet.topic_blocks,
            "/retro-boomer/mainnet/cede1246070229c7/blocks/1"
        );
        assert_eq!(mainnet.topic_txs, "/retro-boomer/mainnet/cede1246070229c7/txs/1");
    }

    #[test]
    fn mainnet_uses_only_the_two_official_retro_boomer_seeds() {
        assert_eq!(
            NetworkConfig::mainnet().dns_seeds,
            &["seed1.retroboomer.org", "seed2.retroboomer.org"]
        );
    }

    #[test]
    fn parse_mainnet() {
        let k: NetworkKind = "mainnet".parse().unwrap();
        assert_eq!(k.to_string(), "mainnet");
        assert!(k.is_mainnet());
    }

    #[test]
    fn parse_unknown_fails() {
        assert!("devnet".parse::<NetworkKind>().is_err());
        assert!("testnet".parse::<NetworkKind>().is_err());
    }
}
