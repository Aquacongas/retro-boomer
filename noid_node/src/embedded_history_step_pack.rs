// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Paranoid Zero.

//! The sole RETRO BOOMER V1 bank. Release builds require authenticated matrix bytes;
//! a development build without a pack has no proof acceptance authority.

struct EmbeddedV21Pack {
    metadata: &'static [u8],
    compressed_matrices: [&'static [u8]; 3],
    release_bank: [u8; 32],
    build_seals: [noid_ivc_core::field_r1cs::BuildAuthenticatedFieldR1csSeal; 3],
}

pub fn embedded_finance_runtime() -> Result<
    Option<std::sync::Arc<noid_recursive::acceptance::history_step::v2::capacity_banked::Runtime>>,
    String,
> {
    let Some(pack) = GENERATED_V21_PACK.as_ref() else {
        return Ok(None);
    };
    let metadata = noid_miner::v21_artifacts::decode_v21_runtime_metadata_pinned(
        pack.metadata,
        pack.release_bank,
    )?;
    // SAFETY: embed_v21_pack fully authenticates each class before staging
    // these exact immutable bytes and emitting the corresponding private seal.
    unsafe { metadata.into_embedded_runtime(pack.compressed_matrices, pack.build_seals) }
        .map(|runtime| Some(std::sync::Arc::new(runtime)))
}

include!(concat!(env!("OUT_DIR"), "/v21_pack.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn embedded_bank_matches_the_source_consensus() {
        if let Some(runtime) = embedded_finance_runtime().unwrap() {
            use noid_recursive::acceptance::history_step::v2::capacity_banked as bank;
            assert_eq!(runtime.bank().config(), bank::Config::retro_boomer_v1().unwrap());
            assert_ne!(
                runtime.bank().digest(),
                noid_miner::v21_artifacts::OBSOLETE_BURN_BANK
            );
            let origin = bank::VerifiedOrigin::from_genesis(runtime.bank()).unwrap();
            assert_eq!(
                origin.origin().boundary(),
                &noid_recursive::genesis_accumulator()
            );
            for class in bank::Class::ALL {
                runtime.prepare_matrix_cache(class).unwrap();
            }
        } else {
            assert!(!cfg!(not(debug_assertions)), "release requires a real bank");
        }
    }
}
