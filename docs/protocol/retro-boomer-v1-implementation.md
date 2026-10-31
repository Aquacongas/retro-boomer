# TAPE V1 — implementation and qualification status

Historical qualification note: test counts recorded below describe the earlier development session. They do not certify the renamed Retro Boomer binaries. See [current rebranding checks](../retro-boomer-rebranding.md).

Status: the native bank in artifacts/retro-boomer-v1-20261031 is approved in source; all eight file hashes and archive transport integrity have been checked locally. Packaging did not regenerate or mathematically reverify proofs. Historical test results below do not qualify the current five-platform releases or installers.

See [the current generation procedure](../../RETRO_BOOMER_BUILD_INSTRUCTIONS.md) and [artifact status](../../artifacts/RETRO_BOOMER_V1_STATUS.md).

## Historical process and artifact status before the emission change

The earlier real generator completed before the request to stop generation. Its files are preserved. The ongoing release build was interrupted with exit 130. No new HistoryStep, PAK, full R1CS-bank generator, full proof generation, benchmark, miner or live blockchain has been started during this stage.

The previous bank `0f38c6a8f233a845c81458f49f83ee74474e9d531dab639c411d51cf10521d23` predates the no-burn rule. It is explicitly obsolete. The node continues to reject that bank. A replacement ten-billion bank was approved in the historical session; that approval does not apply to the native RETRO BOOMER proof-boundary domain. Development builds without a pack have no proof-acceptance authority. See `artifacts/RETRO_BOOMER_V1_STATUS.md`.

## Fee and supply rules implemented

1. The original base, input, output, state-growth and dynamic occupancy fee minimums are retained. Pressure multipliers remain 1/2/4/8 at the original exact integer thresholds.
2. The state-growth fee is paid to the miner. There is no burn deduction, burned-fee field in core/RPC fee breakdowns, burn allocation, substitute transaction or special recipient.
3. Every complete block has one mandatory primary coinbase, with exactly `subsidy(height) + sum(paid logical user fees)`. Both underpayment and overpayment fail validation. An amount that does not fit the canonical lane causes template construction to fail, rather than silently discarding fees.
4. User transaction conservation still requires inputs to fund outputs plus fees. State supply is derived from actual slot updates; the full coinbase restores those existing fee units. Only scheduled subsidy adds to complete-block supply. The state interpreter must not also credit fees separately, which would mint them twice.
5. The same exact rule is constrained in R1CS. The fee minimum and occupancy checks remain; the former burn subtraction/carry relation is removed and all constrained fees feed the checked block sum. Coinbase equals the complete sum.
6. Token redemption, NFT and LP operations remain. Burning a custom asset is distinct from destroying the native TAPE currency; their original TAPE conservation rules remain in force.

Current tokenomics: ten stages of exactly one billion TAPE each, initial reward 100,000 TAPE, first duration 10,000 blocks; rewards halve while durations double. Six decimals, 60-second blocks, no premine, fund allocation, burn or tail. Exact total/max supply: 10,000,000,000 TAPE; zero subsidy from height 10,230,001. The older results below are historical; see [current economics and artifact requirements](economics.md).

## Main changed files

- `noid_chain/src/consensus/fees.rs`, `emission.rs`, `params.rs`, `mod.rs`: full fee transfer and removal of the burn accounting API; original minimums retained.
- `noid_chain/src/consensus/validation.rs`: exact coinbase amount shared by cheap native validation and sequential consensus validation.
- `noid_chain/src/consensus/template.rs`: all fees paid to the sole coinbase, checked amount conversion, whole-block supply regression test.
- `noid_recursive/src/acceptance/trace/fee_arithmetic.rs`: actual constrained no-burn relation; negative and matrix-invariance component tests.
- `noid_mempool/src/pool.rs`, `mining.rs`, `noid_miner/src/template/trades.rs`: fee revenue for ordinary and native/DEX candidate selection includes every paid unit.
- `noid_rpc/src/server.rs`, `types.rs`, `noid_node/src/wallet/mod.rs`, `noid_node/src/bin/noid_cli.rs`: consistent full-fee reporting and wallet planning; obsolete burn labels/fields removed.
- `noid_node/src/config.rs`, `main.rs`, `wallet/keystore.rs`, `noid_gui/src/backend.rs`: coherent default directory and configuration/path tests.
- `noid_node/build.rs`, `src/embedded_history_step_pack.rs`, `noid_miner/src/history_protocol.rs`, `v21_artifacts.rs`: explicit old-bank revocation and approved-bank production embedding guard.
- `bench_prover/src/bin/v2_capacity_support/state.rs`: source accounting updated for future qualification; the tool was not run.
- Runtime examples, launcher references and current documentation use the new directory. Internal cryptographic domains and key derivation contexts were retained.

## Directory and identity

Default data directory is exactly `~/.retro-boomer`, shared by node, wallet and ordinary GUI. Configuration is `~/.retro-boomer/retro-boomerd.toml`; GUI preferences are `~/.retro-boomer/gui-settings.json`. Explicit directory/configuration overrides remain supported. The optional GUI development feature retains the explicit isolated `gui-dev` subdirectory.

No old data directory was opened for migration, copied, moved or deleted. CLI and external miner remain RPC clients rather than separate wallet stores.

Identity remains Retro Boomer, ticker TAPE, protocol V1, Bech32m `tape1` addresses, Poseidon2b and `retro_boomer_` RPC. All feature sets remain active from block 1. Media infrastructure was not changed in this stage.

## Historical verification before the ten-billion emission change

The selected tests were inspected before execution. They do not launch a generator or miner. Fee R1CS tests construct small in-memory component relations and check their constraints; they write no artifacts and generate no cryptographic proofs.

| Check | Result |
|---|---|
| Offline development check: node, GUI, external miner | Passed; existing compiler warnings remain |
| Fee formula and occupancy thresholds | 5 passed |
| Emission accounting | 2 passed |
| Integer halving schedule and rounded total | 3 passed |
| Coinbase/shape validation | 5 passed |
| Whole-block growth fee + tip transfer and supply | 1 passed |
| Fee R1CS constraints, invalid amounts, zero subsidy, matrix invariance | 10 passed |
| Address primitives | 12 passed |
| Node defaults and explicit directory round-trip | 2 passed |
| Wallet fee planning | 7 passed |
| GUI default-directory test | 1 passed |

**48 selected unit tests passed, with no failures.** The final offline `cargo check` of node, GUI and external miner also passed. Logs are preserved locally in a historical local build-output directory; the fee R1CS log supersedes the earlier 9-test component run in the combined core log.

The whole-block accounting test covers the first epoch, a halving and zero-subsidy heights. The R1CS tests reject both one-unit underpayment and overpayment, including after emission exhaustion.

Earlier full-chain/P2P/RPC results belong to the pre-no-burn state and are not current full-suite qualification. The historical test totals above were not rerun during Stage 1. The current bank was generated subsequently; production mining/synchronization, native end-to-end workflows and five-platform packaging remain to be qualified separately.
