> Native materials in artifacts/retro-boomer-v1-20261031 are approved in source. The eight file hashes and packaged archive transport integrity have been checked locally; five-platform build and installer qualification remain pending. See [build and material status](../../RETRO_BOOMER_BUILD_INSTRUCTIONS.md).

# Retro Boomer (TAPE) V1 economics

One TAPE is 1,000,000 atomic units. Genesis creates no coins. Block time remains
60 seconds. The ten paying stages below each issue exactly 1,000,000,000 TAPE.

| Inclusive heights | Blocks | TAPE per block | Atomic units per block |
|---|---:|---:|---:|
| 1–10,000 | 10,000 | 100,000 | 100,000,000,000 |
| 10,001–30,000 | 20,000 | 50,000 | 50,000,000,000 |
| 30,001–70,000 | 40,000 | 25,000 | 25,000,000,000 |
| 70,001–150,000 | 80,000 | 12,500 | 12,500,000,000 |
| 150,001–310,000 | 160,000 | 6,250 | 6,250,000,000 |
| 310,001–630,000 | 320,000 | 3,125 | 3,125,000,000 |
| 630,001–1,270,000 | 640,000 | 1,562.5 | 1,562,500,000 |
| 1,270,001–2,550,000 | 1,280,000 | 781.25 | 781,250,000 |
| 2,550,001–5,110,000 | 2,560,000 | 390.625 | 390,625,000 |
| 5,110,001–10,230,000 | 5,120,000 | 195.3125 | 195,312,500 |
| From 10,230,001 | Unlimited | 0 | 0 |

For zero-based stage s in 0..9, the first height is
`10_000 * (2^s - 1) + 1`, duration is `10_000 * 2^s`, and reward is
`100_000_000_000 >> s` atomic units. The initial atomic reward is divisible by
512, so every reward is exact: no floating point, rounding, final adjustment,
minimum subsidy or tail. Total/max supply is exactly 10,000,000,000 TAPE,
encoded as 10,000,000,000,000,000 atomic units. Height zero and every height
at or after 10,230,001 have zero subsidy, including u64::MAX.

The sole mandatory coinbase pays exactly subsidy plus all paid transaction
fees, including state-growth charges and tips. Underpayment and overpayment
remain invalid. After emission it pays fees alone; an empty fee-free block has
zero coinbase value under the existing canonical coinbase rules. This is not a
replacement fund transaction. Fees transfer existing TAPE; only subsidy creates
coins. No premine, dev/lab allocation, fee burn or native-coin burning is added.
State-growth pricing, anti-spam checks, native tokens, contracts and DEX are unchanged.

## RPC and display

`retro_boomer_getChainInfo` reports the exact maximum and scheduled issuance as decimal
atomic-unit strings. `subsidy_stages` contains inclusive boundaries, durations,
rewards and per-stage issuance. `subsidy_stage_count` is 10 and
`first_zero_subsidy_height` is 10,230,001. The legacy field `halving_interval`
now denotes the duration of the stage containing the **next** block, or zero
when emission has ended; it must not be interpreted as a constant interval.
`current_subsidy_units` and `retro_boomer_getMiningInfo.block_reward_micronoid` also
refer to the next block. GUI/wallet values continue to use integer atomic units.

## Genesis, proofs and existing data

Genesis bytes, nonce, timestamp, PoW target, empty state root, genesis identifier,
addresses and protocol V1 are unchanged. Genesis has no subsidy and contains no
commitment to the reward schedule. However, from block 1 onward the consensus
and constrained HistoryStep relation change; existing non-genesis chain data is
not made compatible by retaining the genesis. The existing P2P NetworkProfile
already includes the HistoryStep bank digest: a new bank changes the profile
identity without changing wire version or genesis. The storage epoch marker
itself identifies schema/genesis, not this emission schedule; unchanged genesis
is therefore not a certificate that an old database follows the new rules.
Do not continue the old chain
with a new binary. No migration, deletion or data reset is performed here.

The former 210-million no-burn bank, pin
`7143e71c32123654fa41ef33015cc37f6a53ae06f88daf6e0d482136243561ee`,
is rejected for this schedule. Its historical local path was
an earlier local artifact directory; that path is not a claim that the
old files are still present.

The native bank from artifacts/retro-boomer-v1-20261031 is approved in noid_node/build.rs. Its eight hashes match the committed release policy and its distribution archive has passed transport verification. Earlier historical terminals must not be substituted. Production matrix authentication remains mandatory. Pack-free debug builds have no proof-acceptance authority.

The existing generator `bench_prover/src/bin/retro_boomer_v1_pack.rs` produces the
Small/Large/Native matrices, runtime metadata, `pins.env`, and real block-one
terminals. Its code checks final-pin correspondence and verifies terminal
round-trips. No separate PAK file or standalone verification command is
invented here. The internal `v21-*` names do not mean three protocol packs.

Generation is a separate owner-run step and was not executed during the source-only rebrand. Future artifact
approval is distinct from five-platform build, installer, mining and sync
qualification, which remains required before release. See
[artifact status](../../artifacts/RETRO_BOOMER_V1_STATUS.md).

Implementation: `noid_chain/src/consensus/subsidy.rs`, `emission.rs`, `template.rs`,
`validation.rs`; `noid_recursive/src/acceptance/trace/subsidy.rs` and
`fee_arithmetic.rs`; RPC metadata and the production build approval gate.

## Historical source-change verification (before current-bank approval)

Changed source/test files:

- `noid_chain/src/consensus/subsidy.rs`: exact ten-stage schedule, supply and boundary tests.
- `noid_chain/src/consensus/params.rs`: initial-subsidy documentation; value remains derived from subsidy.rs.
- `noid_chain/src/consensus/emission.rs`: test heights aligned with the new boundaries; the full-fee rule is retained.
- `noid_chain/src/consensus/template.rs`, `validation.rs`: tests at every stage transition and after exhaustion; existing production coinbase validation is retained.
- `noid_recursive/src/acceptance/trace/subsidy.rs`: actual constrained stage thresholds and exact reward constants.
- `noid_rpc/src/types.rs`, `server.rs`: accurate variable stage durations, exact schedule metadata, next-block subsidy and JSON regression test.
- `noid_node/build.rs`: the original emission change rejected the historical bank; the later source update approved the current ten-billion bank.
- `noid_node/src/bin/noid_cli.rs`: exact decimal amount regression test.
- `noid_gui/src/model.rs`: no invented reward while offline (live RPC supplies it), exact amount/supply regression test. Address generation, layout and mining controls are unchanged.

Updated documents: root `README.md`, this file, `docs/protocol/retro-boomer-v1-implementation.md`,
`RETRO_BOOMER_BUILD_INSTRUCTIONS.md`, and the historical warning in `RETRO_BOOMER_MINE_ONE_BLOCK.md`.

| Check | Result |
|---|---|
| Offline locked cargo check: chain, recursive, RPC, node, GUI, external miner | Passed; pre-existing warnings remain |
| Native stage boundaries, per-stage billion, total ten billion, fractions and permanent zero | 4 passed |
| Full-fee emission accounting | 2 passed |
| Native coinbase validation at every boundary | 5 passed |
| Template/state integration at every boundary | 1 passed |
| Growth fees and tips transfer with no additional issuance, including zero subsidy | 1 passed |
| Unchanged canonical genesis, PoW and empty root | 6 passed; 2 generation tests ignored |
| Subsidy R1CS: native agreement, forged reward/height rejection, structure invariance | 1 passed |
| Fee/coinbase R1CS including zero-subsidy fee-only accounting | 10 passed |
| RPC schedule JSON | 1 passed |
| GUI exact reward/supply presentation | 1 passed |
| CLI decimal parsing/formatting and existing amount checks | 3 passed |
| Debug check supplied with the old 7143... bank | Rejected as expected before embedding |

These selected suites total 35 passed tests. The broader existing GUI model suite
also ran: 12 passed and 1 failed in `preview_addresses_match_the_canonical_display_width`.
That unchanged test expects 60-character preview addresses while the existing
preview data mixes `o1` and 63-character `tape1` addresses. This is separate from
emission; neither address fixtures nor address format were changed to hide it.
The new emission presentation test passes independently.

The R1CS component tests construct only small in-memory constraints; they do not
write matrix banks or generate cryptographic proofs. No full pack generation,
release build, node, miner, migration, or wallet/chain deletion was performed.
The table records earlier component checks, not tests rerun during repository
preparation. Current-bank production proof acceptance and multi-node operation
still require recorded end-to-end release qualification; artifact generation
itself has already completed.
