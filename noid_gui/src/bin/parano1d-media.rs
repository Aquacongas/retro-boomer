// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Paranoid Zero.

#[allow(dead_code)]
#[path = "../assets/media.rs"]
mod media;

#[allow(dead_code)]
#[path = "../market_data.rs"]
mod market_data;

use std::{path::PathBuf, sync::Arc};

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() == 1 && args[0] == "--help" {
        println!("retro-boomer-media\n  --verify-store <directory>\n\nAsset images, metadata and indexed price history.\nEnvironment: RETRO_BOOMER_MEDIA_STORE, RETRO_BOOMER_MEDIA_LISTEN (loopback), RETRO_BOOMER_MEDIA_RPC (loopback), RETRO_BOOMER_MEDIA_QUOTA_MIB, RETRO_BOOMER_HISTORY_QUOTA_MIB. Place a TLS proxy in front of the listener.");
        return Ok(());
    }
    if args.len() == 2 && args[0] == "--verify-store" {
        let directory = PathBuf::from(&args[1]);
        println!(
            "{} verified media bundles",
            media::service::check_store(&directory)?
        );
        return Ok(());
    }
    if !args.is_empty() {
        return Err("Use --help for media service options.".into());
    }
    let directory = std::env::var_os("RETRO_BOOMER_MEDIA_STORE")
        .map(PathBuf::from)
        .ok_or("Set RETRO_BOOMER_MEDIA_STORE to a dedicated media directory.")?;
    let address = std::env::var("RETRO_BOOMER_MEDIA_LISTEN")
        .unwrap_or_else(|_| "127.0.0.1:19610".into())
        .parse()
        .map_err(|_| "Invalid media listen address.")?;
    let rpc = std::env::var("RETRO_BOOMER_MEDIA_RPC").unwrap_or_else(|_| "http://127.0.0.1:19601".into());
    let quota = std::env::var("RETRO_BOOMER_MEDIA_QUOTA_MIB")
        .unwrap_or_else(|_| "4096".into())
        .parse::<u64>()
        .ok()
        .filter(|n| *n > 0)
        .and_then(|n| n.checked_mul(1024 * 1024))
        .ok_or("Invalid media storage quota.")?;
    let store = Arc::new(media::service::Store::open(directory, quota, rpc)?);
    media::service::serve(store, address).await
}
