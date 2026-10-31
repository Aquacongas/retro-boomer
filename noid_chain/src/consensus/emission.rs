// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Paranoid Zero.

//! TAPE issuance and miner-claimable transaction fees. There is no reserved
//! subsidy share, state-dependent reward, minimum reward or tail emission.

use crate::consensus::fees::claimable_fee_for_tx_body;
use crate::consensus::params::MICRONOID_PER_NOID;
use noid_tx::types::TxBody;

pub fn block_reward_at_height(height: u64, _log_slots: u32) -> u64 {
    super::subsidy::block_subsidy(height)
}

/// Sum all gross transaction fees (non-coinbase) in μTAPE.
///
/// A block can contain 255 `u64` fees, so aggregation uses `u128`; consensus
/// predicates never silently saturate a monetary total. This is accounting
/// data. Every paid fee, including the state-growth component and tips,
/// is transferred to the miner.
pub fn total_fees(txs: &[TxBody]) -> u128 {
    txs.iter()
        .filter(|tx| !tx.is_coinbase)
        .map(|tx| u128::from(tx.fee))
        .sum()
}

/// Required canonical coinbase value (μTAPE); admission enforces equality.
///
/// Every paid user fee is included; the state-growth fee is not destroyed.
pub fn max_coinbase_value(
    child_height: u64,
    child_log_slots: u32,
    parent_active_slot_count: u64,
    parent_log_slots: u32,
    non_coinbase_txs: &[TxBody],
) -> u128 {
    let claimable_fee_sum: u128 = non_coinbase_txs
        .iter()
        .filter(|tx| !tx.is_coinbase)
        .map(|tx| {
            u128::from(claimable_fee_for_tx_body(
                tx,
                parent_active_slot_count,
                parent_log_slots,
            ))
        })
        .sum();
    max_coinbase_value_from_claimable_fee_sum(child_height, child_log_slots, claimable_fee_sum)
}

/// Same as [`max_coinbase_value`] but accepts an already checked sum of
/// miner-paid fees (the full paid amount, including state growth).
///
/// Used by `validate_block_consensus` to avoid cloning all non-coinbase
/// bodies just to repeat fee accounting.
#[inline]
pub fn max_coinbase_value_from_claimable_fee_sum(
    child_height: u64,
    _child_log_slots: u32,
    claimable_fee_sum: u128,
) -> u128 {
    u128::from(super::subsidy::block_subsidy(child_height)) + claimable_fee_sum
}

/// Format a μTAPE amount as a human-readable string (not consensus-critical).
pub fn format_noid(micronoid: u64) -> String {
    let whole = micronoid / MICRONOID_PER_NOID;
    let frac = micronoid % MICRONOID_PER_NOID;
    format!("{}.{:06} TAPE", whole, frac)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::subsidy::{block_subsidy, FIRST_ZERO_SUBSIDY_HEIGHT};
    use noid_poseidon2b::primitives::Address;
    use noid_tx::{output_bitmap_bit, TxInput, TxOutput, TX_INPUTS};

    #[test]
    fn miner_receives_full_subsidy_and_all_state_growth_fees() {
        let tx = TxBody {
            epoch_anchor: [1; 32],
            fee: 9_000,
            input_owner: Address([3; 32]),
            inputs: std::array::from_fn(|i| {
                if i == 0 {
                    TxInput {
                        slot_index: 1,
                        creation_id: 1,
                        amount: 100_000,
                    }
                } else {
                    TxInput::dummy()
                }
            }),
            outputs: [
                TxOutput {
                    slot_index: 2,
                    amount: 50_000,
                    owner: Address([4; 32]),
                },
                TxOutput {
                    slot_index: 3,
                    amount: 41_000,
                    owner: Address([3; 32]),
                },
            ],
            validity_bitmap: 1 | output_bitmap_bit(0) | output_bitmap_bit(1),
            is_coinbase: false,
        };
        assert_eq!(TX_INPUTS, 8);
        assert_eq!(total_fees(std::slice::from_ref(&tx)), 9_000);
        for height in [
            1,
            1_440,
            4_320,
            10_000,
            10_001,
            30_000,
            30_001,
            FIRST_ZERO_SUBSIDY_HEIGHT,
            u64::MAX,
        ] {
            for depth in [24, 25, 32] {
                assert_eq!(
                    max_coinbase_value(height, depth, 0, 24, std::slice::from_ref(&tx)),
                    u128::from(block_subsidy(height)) + 9_000
                );
            }
        }
    }

    #[test]
    fn no_state_expansion_or_old_payout_height_changes_the_reward() {
        for height in [
            0,
            1,
            1_440,
            4_320,
            10_000,
            10_001,
            30_000,
            30_001,
            FIRST_ZERO_SUBSIDY_HEIGHT,
            u64::MAX,
        ] {
            for depth in 24..=32 {
                assert_eq!(block_reward_at_height(height, depth), block_subsidy(height));
                assert_eq!(
                    max_coinbase_value_from_claimable_fee_sum(height, depth, 0),
                    u128::from(block_subsidy(height))
                );
            }
        }
        assert_eq!(format_noid(500_000_001), "500.000001 TAPE");
    }
}
