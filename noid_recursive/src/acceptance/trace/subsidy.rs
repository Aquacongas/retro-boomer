// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Paranoid Zero.

//! Height-bound TAPE halving relation. The circuit has identical structure at
//! every height, including genesis and the permanently zero-subsidy suffix.

use noid_chain::consensus::subsidy::{subsidy_stage_start, INITIAL_SUBSIDY, LAST_SUBSIDY_EPOCH};

use super::{flat_const, mul, pin_eq, range_check_bits, FieldR1csBuilder, LinExpr, Wire, F128};

pub struct SubsidyTrace {
    pub height_bits: [Wire; 64],
    pub miner_subsidy: LinExpr,
}

/// Return the Boolean predicate `height >= threshold` from a constrained
/// little-endian bit decomposition. Borrow arithmetic is over bits, never a
/// characteristic-two field comparison of monetary values.
fn at_least(b: &mut FieldR1csBuilder, bits: &[Wire; 64], threshold: u64) -> LinExpr {
    let mut borrow = LinExpr::zero();
    for (index, &wire) in bits.iter().enumerate() {
        let bit = LinExpr::from_wire(wire);
        borrow = if threshold >> index & 1 == 0 {
            mul(b, &borrow, &bit.add_const(F128::ONE))
        } else {
            let keep = mul(b, &borrow, &bit);
            bit.add_const(F128::ONE).add(&keep)
        };
    }
    borrow.add_const(F128::ONE)
}

pub fn bind_subsidy(b: &mut FieldR1csBuilder, height: &LinExpr) -> SubsidyTrace {
    let height_bits: [Wire; 64] = range_check_bits(b, height, 64).try_into().unwrap();
    let starts: Vec<_> = (0..=LAST_SUBSIDY_EPOCH + 1)
        .map(|epoch| at_least(b, &height_bits, subsidy_stage_start(epoch)))
        .collect();
    // Adjacent threshold predicates differ only in their disjoint epoch.
    // Every selected constant is exact in atomic units; no truncation is needed.
    let miner_subsidy = (0..=LAST_SUBSIDY_EPOCH as usize).fold(LinExpr::zero(), |sum, epoch| {
        let selected = starts[epoch].add(&starts[epoch + 1]);
        sum.add(&selected.scale(flat_const(u128::from(INITIAL_SUBSIDY >> epoch))))
    });
    // Keep a dedicated output wire: downstream accounting consumes this exact
    // constrained value, rather than a separately supplied native reward.
    let output = b.alloc_f128(miner_subsidy.eval(b.values()));
    let output_expr = LinExpr::from_wire(output);
    pin_eq(b, &output_expr, &miner_subsidy);
    SubsidyTrace {
        height_bits,
        miner_subsidy: output_expr,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::acceptance::trace::{alloc_block, const_block};
    use noid_chain::consensus::subsidy::{block_subsidy, FIRST_ZERO_SUBSIDY_HEIGHT};
    use noid_core::Block128;

    #[test]
    fn all_epoch_boundaries_match_native_and_reject_forged_rewards() {
        let mut heights = vec![0, 1, FIRST_ZERO_SUBSIDY_HEIGHT, u64::MAX];
        for epoch in 1..=LAST_SUBSIDY_EPOCH + 1 {
            let first = subsidy_stage_start(epoch);
            heights.extend([first - 1, first, first + 1]);
        }
        let mut digest = None;
        for height in heights {
            let mut b = FieldR1csBuilder::new();
            let h = alloc_block(&mut b, Block128::from(height));
            let trace = bind_subsidy(&mut b, &h);
            let expected = const_block(Block128::from(block_subsidy(height)));
            assert_eq!(
                trace.miner_subsidy.eval(b.values()),
                expected.eval(b.values())
            );
            let reward_wire = trace.miner_subsidy.terms[0].0 as usize;
            let height_wire = h.terms[0].0 as usize;
            let (matrix, mut witness) = b.build();
            assert!(matrix.satisfies(&witness), "height={height}");
            let actual = matrix.structural_statement_digest();
            assert!(digest.is_none_or(|expected| expected == actual));
            digest = Some(actual);
            witness[reward_wire] += F128::ONE;
            assert!(!matrix.satisfies(&witness), "forged subsidy at {height}");
            witness[reward_wire] += F128::ONE;
            witness[height_wire] += F128::ONE;
            assert!(!matrix.satisfies(&witness), "unbound height at {height}");
        }
    }
}
