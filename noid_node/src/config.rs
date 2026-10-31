// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Paranoid Zero.

//! Node configuration (parsed from TOML file or CLI flags).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub network: NetworkConfig,
    pub storage: StorageConfig,
    pub rpc: RpcConfig,
    pub mining: MiningConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkConfig {
    /// P2P listen address.
    /// Config file: HOST:PORT or libp2p multiaddr ("/ip4/...").
    /// CLI flag: --p2p-listen HOST:PORT  (e.g. 0.0.0.0:19600)
    /// Defaults to the compiled network's P2P port (19600 on mainnet).
    pub listen: Option<String>,
    /// Bootstrap seed peers.
    /// Config file: list of HOST:PORT strings (e.g. ["1.2.3.4:19600"]).
    /// CLI flag: --seed HOST:PORT  (repeat for multiple seeds).
    pub seeds: Vec<String>,
    /// Public TCP addresses at which this node is reachable.
    ///
    /// Most wallets leave this empty. Public nodes behind an unspecified
    /// listen socket (for example `0.0.0.0:19600`) set the externally reachable
    /// IP here so Identify and Circuit Relay v2 can advertise a usable path.
    /// Config file: list of IP:PORT strings or libp2p multiaddrs.
    #[serde(default)]
    pub public_addresses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Storage backend: "mdbx" or "ram".
    pub backend: String,
    /// Data directory override. Default: ~/.retro-boomer.
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RpcConfig {
    /// JSON-RPC listen address.
    /// Defaults to the compiled network's local RPC address
    /// (127.0.0.1:19601 on mainnet).
    pub listen: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MiningConfig {
    /// Enable built-in miner.
    pub enabled: bool,
    /// Miner coinbase address (bech32m). Empty = current active wallet address.
    pub miner_address: String,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig {
                listen: None, // determined by --network at runtime
                seeds: vec![],
                public_addresses: vec![],
            },
            storage: StorageConfig {
                backend: "mdbx".into(),
                path: noid_core::application::default_data_dir(),
            },
            rpc: RpcConfig {
                listen: None, // determined by --network at runtime
            },
            mining: MiningConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_storage_uses_only_the_new_directory() {
        let config = NodeConfig::default();
        assert_eq!(config.storage.path, noid_core::application::default_data_dir());
        assert!(config.network.seeds.is_empty());
    }

    #[test]
    fn explicit_storage_directory_survives_configuration_roundtrip() {
        let mut config = NodeConfig::default();
        config.storage.path = PathBuf::from("/tmp/retro-boomerd-explicit-data");
        let encoded = toml::to_string(&config).unwrap();
        let decoded: NodeConfig = toml::from_str(&encoded).unwrap();
        assert_eq!(decoded.storage.path, config.storage.path);
    }
}
