// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Paranoid Zero.

//! Retro Boomer V1 issuance in atomic units (one millionth of TAPE).
//! Ten stages each issue exactly one billion TAPE. Stage lengths double
//! while rewards halve exactly. Genesis and the suffix after stage ten issue zero.

pub const UNITS_PER_TAPE: u64 = 1_000_000;
pub const INITIAL_SUBSIDY: u64 = 100_000 * UNITS_PER_TAPE;
pub const FIRST_STAGE_BLOCKS: u64 = 10_000;
pub const SUBSIDY_STAGE_COUNT: u32 = 10;
pub const LAST_SUBSIDY_EPOCH: u32 = SUBSIDY_STAGE_COUNT - 1;
pub const STAGE_ISSUANCE: u128 = 1_000_000_000 * UNITS_PER_TAPE as u128;
pub const MAX_SUPPLY: u128 = 10_000_000_000 * UNITS_PER_TAPE as u128;
pub const FIRST_ZERO_SUBSIDY_HEIGHT: u64 = subsidy_stage_start(SUBSIDY_STAGE_COUNT);

/// Also accepts stage ten, which marks the start of the zero-subsidy suffix.
pub const fn subsidy_stage_start(stage: u32) -> u64 {
    assert!(stage <= SUBSIDY_STAGE_COUNT);
    FIRST_STAGE_BLOCKS * ((1u64 << stage) - 1) + 1
}

pub const fn subsidy_stage_blocks(stage: u32) -> u64 {
    assert!(stage < SUBSIDY_STAGE_COUNT);
    FIRST_STAGE_BLOCKS << stage
}

/// None at genesis and after the final paying height. At most ten comparisons,
/// including at u64::MAX; no height-dependent shift or multiplication can overflow.
pub const fn subsidy_stage(height: u64) -> Option<u32> {
    if height == 0 {
        return None;
    }
    let mut stage = 0;
    while stage < SUBSIDY_STAGE_COUNT {
        if height < subsidy_stage_start(stage + 1) {
            return Some(stage);
        }
        stage += 1;
    }
    None
}

/// The complete subsidy belongs to the miner. Fees are separate transfers.
pub const fn block_subsidy(height: u64) -> u64 {
    match subsidy_stage(height) {
        Some(stage) => INITIAL_SUBSIDY >> stage,
        None => 0,
    }
}

/// Cumulative newly issued TAPE through this height. Fees never enter this sum.
pub const fn issued_subsidy_through(height: u64) -> u128 {
    let mut remaining = height;
    let mut stage = 0;
    let mut total = 0u128;
    while remaining != 0 && stage < SUBSIDY_STAGE_COUNT {
        let duration = subsidy_stage_blocks(stage);
        let blocks = if remaining < duration {
            remaining
        } else {
            duration
        };
        total += blocks as u128 * (INITIAL_SUBSIDY >> stage) as u128;
        remaining -= blocks;
        stage += 1;
    }
    total
}

pub const TOTAL_SCHEDULED_ISSUANCE: u128 = issued_subsidy_through(u64::MAX);
const _: () = assert!(INITIAL_SUBSIDY % (1 << LAST_SUBSIDY_EPOCH) == 0);
const _: () = assert!(TOTAL_SCHEDULED_ISSUANCE == MAX_SUPPLY);
const _: () = assert!(STAGE_ISSUANCE * SUBSIDY_STAGE_COUNT as u128 == MAX_SUPPLY);

#[cfg(test)]
mod tests {
    use super::*;

    // Independent literal specification: inclusive stage end and atomic reward.
    const STAGES: [(u64, u64); 10] = [
        (10_000, 100_000_000_000),
        (30_000, 50_000_000_000),
        (70_000, 25_000_000_000),
        (150_000, 12_500_000_000),
        (310_000, 6_250_000_000),
        (630_000, 3_125_000_000),
        (1_270_000, 1_562_500_000),
        (2_550_000, 781_250_000),
        (5_110_000, 390_625_000),
        (10_230_000, 195_312_500),
    ];

    #[test]
    fn every_stage_boundary_matches_the_literal_schedule() {
        assert_eq!(block_subsidy(0), 0);
        let mut previous_end = 0;
        let mut previous_reward = 0;
        for (index, (end, reward)) in STAGES.into_iter().enumerate() {
            let first = previous_end + 1;
            assert_eq!(subsidy_stage_start(index as u32), first);
            assert_eq!(subsidy_stage_blocks(index as u32), end - previous_end);
            assert_eq!(block_subsidy(first - 1), previous_reward);
            assert_eq!(block_subsidy(first), reward);
            assert_eq!(block_subsidy(end), reward);
            let next = STAGES.get(index + 1).map(|(_, r)| *r).unwrap_or(0);
            assert_eq!(block_subsidy(end + 1), next);
            previous_end = end;
            previous_reward = reward;
        }
    }

    #[test]
    fn each_stage_issues_exactly_one_billion_and_total_is_ten_billion() {
        let mut previous_end = 0;
        let mut independently_issued = 0u128;
        for (end, reward) in STAGES {
            let stage_units = u128::from(end - previous_end) * u128::from(reward);
            assert_eq!(stage_units, 1_000_000_000_000_000);
            assert_eq!(
                issued_subsidy_through(end) - issued_subsidy_through(previous_end),
                stage_units
            );
            independently_issued += stage_units;
            assert_eq!(issued_subsidy_through(end), independently_issued);
            assert_eq!(
                issued_subsidy_through(previous_end + 1),
                independently_issued - stage_units + u128::from(reward)
            );
            previous_end = end;
        }
        assert_eq!(independently_issued, 10_000_000_000_000_000);
        assert_eq!(TOTAL_SCHEDULED_ISSUANCE, independently_issued);
        assert_eq!(TOTAL_SCHEDULED_ISSUANCE, MAX_SUPPLY);
    }

    #[test]
    fn fractional_rewards_are_exact_atomic_values() {
        assert_eq!(block_subsidy(630_001), 1_562_500_000);
        assert_eq!(block_subsidy(1_270_001), 781_250_000);
        assert_eq!(block_subsidy(2_550_001), 390_625_000);
        assert_eq!(block_subsidy(5_110_001), 195_312_500);
    }

    #[test]
    fn issuance_stops_permanently_without_tail_or_balancing_mint() {
        assert_eq!(FIRST_ZERO_SUBSIDY_HEIGHT, 10_230_001);
        assert_eq!(block_subsidy(FIRST_ZERO_SUBSIDY_HEIGHT - 1), 195_312_500);
        assert_eq!(issued_subsidy_through(0), 0);
        for height in [
            FIRST_ZERO_SUBSIDY_HEIGHT,
            FIRST_ZERO_SUBSIDY_HEIGHT + 1,
            u64::MAX,
        ] {
            assert_eq!(subsidy_stage(height), None);
            assert_eq!(block_subsidy(height), 0);
            assert_eq!(issued_subsidy_through(height), MAX_SUPPLY);
        }
    }
}
