# RETRO BOOMER (TAPE)

**OK BOOMER.**

**THE INTERNET GREW UP. WE REFUSED.**

RETRO BOOMER is an independent, native Layer 1 Proof-of-Work blockchain built on the original Parano1d architecture.

Old-school attitude. Real mining. Real blocks. Cryptographic HistoryStep verification. No premine, no shortcuts, and a fixed supply of **10 billion TAPE**.

**Website:** https://retroboomer.org  
**Source code:** https://github.com/Aquacongas/retro-boomer  
**Community:** https://discord.gg/ScDrUsxSWN

---

## What Is RETRO BOOMER?

RETRO BOOMER is not a token deployed on another blockchain. It is an independent Proof-of-Work network with its own genesis block, consensus rules, native currency, wallets, mining infrastructure, and peer-to-peer protocol.

The implementation retains the original Parano1d architecture, including:

- Proof-of-Work consensus and native mining.
- Recursive HistoryStep verification.
- Native wallet and transaction functionality.
- Native asset and token support.
- Contract execution and object mechanisms.
- DEX and liquidity functionality.
- NFT-related functionality.
- Peer-to-peer synchronization and block validation.

**RETRO BOOMER is an independent fork, not an official Parano1d release.**

## Network Parameters

| Parameter | Value |
|---|---|
| Blockchain | RETRO BOOMER |
| Native currency | TAPE |
| Consensus | Proof of Work |
| Target block interval | 60 seconds |
| Maximum supply | 10,000,000,000 TAPE |
| Decimal places | 6 |
| Initial block subsidy | 100,000 TAPE |
| Emission stages | 10 |
| Premine | None |
| Developer allocation | None |
| Tail emission | None |
| Native TAPE fee burning | None |
| Address format | Bech32m |
| Address prefix | `tape1` |
| Default P2P port | `19600/TCP` |
| Default RPC port | `19601` |
| RPC namespace | `retro_boomer_` |
| Network marker | `RTBM` |

### Genesis

RETRO BOOMER uses its own empty genesis block, without a premine or initial coin allocation.

**Genesis hash:**

```text
cede1246070229c7d312f341a27d801dbf5b7bf32f8c76552d21f9203b325a13
```

## TAPE Emission Schedule

RETRO BOOMER has a fixed maximum supply of **10,000,000,000 TAPE**.

Each of the ten emission stages distributes exactly **1,000,000,000 TAPE**. Stage duration doubles while the subsidy halves.

| Stage | Block heights | Reward per block |
|---|---|---|
| 1 | 1–10,000 | 100,000 TAPE |
| 2 | 10,001–30,000 | 50,000 TAPE |
| 3 | 30,001–70,000 | 25,000 TAPE |
| 4 | 70,001–150,000 | 12,500 TAPE |
| 5 | 150,001–310,000 | 6,250 TAPE |
| 6 | 310,001–630,000 | 3,125 TAPE |
| 7 | 630,001–1,270,000 | 1,562.5 TAPE |
| 8 | 1,270,001–2,550,000 | 781.25 TAPE |
| 9 | 2,550,001–5,110,000 | 390.625 TAPE |
| 10 | 5,110,001–10,230,000 | 195.3125 TAPE |
| After stage 10 | 10,230,001 onward | 0 TAPE |

After block 10,230,000, no additional native TAPE is created through block subsidies. Miners receive eligible transaction fees only.

There is no premine, developer tax, mandatory allocation, or perpetual tail emission.

For the full consensus rules, see [Economics](docs/protocol/economics.md).

## Wallet Addresses

RETRO BOOMER wallet addresses use Bech32m encoding and begin with:

```text
tape1
```

Native TAPE amounts use six decimal places.

The default data directory on Linux and macOS is:

```text
~/.retro-boomer
```

The default node configuration file is:

```text
~/.retro-boomer/retro-boomerd.toml
```

Windows stores application data under:

```text
%APPDATA%\RetroBoomer
```

Wallets and private keys should always be backed up securely. Never share wallet seed material or private keys.

## Official Network Seeds

The RETRO BOOMER mainnet has two official DNS bootstrap seeds:

```text
seed1.retroboomer.org
seed2.retroboomer.org
```

Both use the default P2P port `19600/TCP`.

Seed discovery is provided through the existing peer-to-peer networking implementation. Manually configured peers and persisted peers are also supported.

## Building RETRO BOOMER

RETRO BOOMER is implemented in Rust.

The primary executable targets are:

| Component | Executable |
|---|---|
| Full node | `retro-boomerd` |
| Command-line wallet and tools | `retro-boomer-cli` |
| External miner | `retro-boomer-miner` |
| Desktop GUI wallet | `retro-boomer-gui` |

### Prerequisites

A working Rust/Cargo toolchain and the project-specific dependencies are required.

Production builds also require the approved RETRO BOOMER HistoryStep materials and matching bank pins.

**The HistoryStep material pack must be authentic and verified. The release build must not bypass or replace the existing proof and matrix validation process.**

### Standard local release build

From the repository root, with the approved local HistoryStep pack available:

```bash
CARGO_BUILD_JOBS=8 ./scripts/build_release.sh \
  --v1-pack artifacts/retro-boomer-v1-20261031 \
  --v1-pins artifacts/retro-boomer-v1-20261031/pins.env \
  --with-gui
```

This builds the node, CLI, external miner, and desktop GUI using the existing project build procedure.

The example assumes the approved material directory is present at the specified local path. Adjust the paths if the verified materials are stored elsewhere.

See [RETRO BOOMER Build Instructions](RETRO_BOOMER_BUILD_INSTRUCTIONS.md) for environment setup and additional requirements.

## Platform Support

The release infrastructure targets five platforms:

| Platform | Core package | GUI package |
|---|---|---|
| Linux x86-64 | `.tar.gz` | `.deb` |
| Linux ARM64 | `.tar.gz` | `.deb` |
| Windows x86-64 | `.zip` | `.exe` installer |
| macOS Intel | `.tar.gz` | `.dmg` |
| macOS Apple Silicon | `.tar.gz` | `.dmg` |

Official platform availability depends on successful release qualification and publication.

Check [GitHub Releases](https://github.com/Aquacongas/retro-boomer/releases) for published downloads.

## HistoryStep Verification

RETRO BOOMER uses the original Parano1d-derived recursive HistoryStep implementation to verify cryptographic history relations.

The RETRO BOOMER V1 material bank is independently prepared, pinned, and checked by the existing build and verification mechanisms.

The approved V1 bank identifier is:

```text
841c496f9b1b24db586fb3a976e2c201c335634b84a91193bbe4e4f5d7537f58
```

Release builds require the matching verified materials. Missing or inconsistent materials must cause the build to fail rather than silently producing an unverified node.

For technical documentation, see [HistoryStep and V1 Implementation](docs/protocol/retro-boomer-v1-implementation.md).

## Development and Releases

The repository uses separate workflows for:

- Platform continuous integration.
- Release candidate builds and packaging.
- Explicitly authorized release publication.

Release candidates are built from the same source revision and verified HistoryStep input.

The release process supports unpublished Draft Releases so that release products can be reviewed before publication.

See [Release Documentation](.github/RELEASING.md) for details.

## Security

Never disclose wallet private keys, recovery information, access tokens, or sensitive credentials.

For security-sensitive reports, follow the [Security Policy](.github/SECURITY.md). Do not disclose exploitable vulnerabilities or private information in public GitHub issues.

For general bugs and feature requests, use [GitHub Issues](https://github.com/Aquacongas/retro-boomer/issues).

## Project and Community

- **Official website:** https://retroboomer.org
- **GitHub:** https://github.com/Aquacongas/retro-boomer
- **Discord:** https://discord.gg/ScDrUsxSWN
- **Contact:** aquacongass@proton.me

## Open Source and Attribution

RETRO BOOMER is an independent project based on the original Parano1d codebase.

Original upstream project: https://git.parano1d.org/ignotusnemo/parano1d

RETRO BOOMER is not affiliated with or endorsed by the original Parano1d maintainers.

The project retains applicable copyright notices, licensing obligations, and third-party attributions. Refer to [LICENSE](LICENSE), [NOTICE](NOTICE), and the relevant third-party license files.

---

**OK BOOMER. REWIND THE HYPE. MINE THE FUTURE.**