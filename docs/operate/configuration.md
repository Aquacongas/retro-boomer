# Configuration

Core reads TOML configuration from:

```text
~/.retro-boomer/retro-boomerd.toml
```

unless `--config` selects another file. A missing file is created atomically
with safe defaults. Malformed TOML fails startup; it is never silently
replaced.

## Complete file

```toml
[network]
listen = "0.0.0.0:19600"
seeds = []

[storage]
backend = "mdbx"
path = "~/.retro-boomer"

[rpc]
listen = "127.0.0.1:19601"

[mining]
enabled = false
miner_address = ""
```

Command-line values override their corresponding file values.

## Network

`network.listen` accepts `HOST:PORT` or a libp2p multiaddress. Bind
`0.0.0.0:19600` to accept public IPv4 connections.

`network.seeds` adds bootstrap peers. Accepted forms include:

```text
198.51.100.10:19600
seed.example.org:19600
/ip4/198.51.100.10/tcp/19600
dnsaddr:seed.example.org
```

The two built-in default seeds are `seed1.retroboomer.org` and
`seed2.retroboomer.org`. Bare hostnames resolve through the existing system
DNS and libp2p DNS transport on TCP 19600; no seed IP is compiled in. The existing
five-second system lookup limit, DNS fallback and connection retry policy remain.
A failed lookup or unavailable seed does not terminate the node; without reachable
peers it cannot synchronize until a peer becomes available.

The `seeds = []` example above means no additional manually configured seeds;
it does not disable the built-in defaults. `--disable-dns-seeds` disables only
the built-in set and preserves configured peers, for isolated operation.
Both official seeds already appear in each node's default set, so the seed
servers need no additional reciprocal `--seed` entries. Their DNS records must
resolve to reachable nodes listening on TCP 19600, with the same genesis/network
and approved bank. This configuration change does not deploy or verify them.

The `--seed HOST:PORT` option is repeatable and appends to configured seeds.

## Storage

`storage.backend` is `mdbx` in production. The RAM backend is for tests and
does not provide durable node State.

`storage.path` contains the chain database, wallet artifacts, peer identity,
proof cache and snapshot staging. Do not point two running node processes at
the same directory.

The command-line `--data-dir` overrides this path.

## RPC

Keep RPC on:

```text
127.0.0.1:19601
```

The interface includes wallet submission and process control. It is not a
public explorer API with general authentication.
An unauthenticated non-loopback listener is rejected at startup.

External-mining deployments require `--mining-key` or
`--mining-key-file`. The credential authorizes only `getBlockTemplate` and
`submitBlock`; it cannot authorize wallet or process-control methods. The
bearer token does not encrypt transport. Use loopback, a private network, an
SSH tunnel or an authenticated TLS proxy when the worker is remote.

A pool or exchange may add a separate `--operator-key-file` for a remote accounting and payout host. Its fixed scope contains bounded wallet status, balance, mined-block and receipt queries, send planning and submission, confirmed and mempool transaction lookup, receipt verification, address validation, chain status, fee estimates, exact wallet consolidation, and submission of an externally authorized raw transaction intent. Mining, process control, wallet scanning and discovery, address management, unbounded wallet listings, and all unlisted methods remain denied. The operator token must differ from the mining token and carries spending authority, so keep it in an owner-only file and expose it only through a firewalled private or encrypted transport. The exact method list is documented in [JSON-RPC API](../reference/rpc.md#authentication).

RPC supports HTTP only. WebSocket upgrades are rejected and request bodies are limited to 2,237,632 bytes. JSON-RPC batches remain supported within that body limit.

## Mining

Process mode is authoritative:

```sh
retro-boomerd --mode node
retro-boomerd --mode miner
retro-boomerd --mode extminer --mining-key-file ~/.retro-boomer/mining.key
```

The legacy `mining.enabled` field does not override `--mode`. An empty
`miner_address` uses the wallet's active address. `--miner-address` overrides
the configured payout for that process.

Use `--cpu-threads N` to limit the shared internal-mining CPU pool. It does not
apply to ordinary node mode or to the separate external worker.

## Logs

`--log` accepts filters such as `error`, `warn`, `info` and `debug`. Start with
`info`. Debug logging is useful during a bounded investigation but may be
substantially noisier.

Under systemd, output goes to the journal. The GUI redirects its private node
to `retro-boomerd.log` in the selected data directory.

## Preflight

Validate CPU support without creating configuration, wallet or database files:

```sh
retro-boomerd --check-hardware
```

The successful final line is `NODE READY`.

Large block production is a command-line choice: `--v2-large-blocks`. It
applies to internal mining and external templates, never to verification
permission. During native execution the node automatically considers Native for
token, launchpad, DEX and NFT operations. The target is 30 seconds. All nodes
verify Small, Large and Native.
Contract RPC methods require local owner scope.
