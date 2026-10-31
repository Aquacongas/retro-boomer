// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Paranoid Zero.

//! Derive and qualify the RETRO BOOMER bank from an actually mined genesis successor.
//! `--qualify` derives integrated geometry without emitting release artifacts.
//! Full generation checks every frozen matrix, rechecks final-pin identity,
//! and proves/verifies a real block in each class before publishing metadata.

use noid_chain::{Block, BlockHeader, ChainState};
use noid_ivc_core::field_r1cs::{CompactFieldR1cs, FieldR1cs};
use noid_miner::v21_artifacts::{
    encode_v21_runtime_metadata, v21_matrix_file_name, V21_MATRIX_MAX_BYTES, V21_METADATA_FILE,
};
use noid_recursive::acceptance::history_step::v2::{
    self, capacity_banked as bank, V2Config, V2Error,
};
use noid_recursive::{
    HistoryStepBlockInput, HistoryStepMatrixLease, PreparedHistoryStepGhostAuthorization,
};
use std::{
    error::Error,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{atomic::AtomicBool, Arc},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
fn error(e: impl std::fmt::Debug) -> Box<dyn Error> {
    format!("{e:?}").into()
}

/// The same authenticated file source serves generation and qualification.
/// Reading a not-yet-written class fails; it never substitutes a fake matrix.
struct MatrixFiles {
    directory: PathBuf,
    bank: bank::Bank,
}
impl bank::MatrixSource for MatrixFiles {
    fn load(&self, class: bank::Class) -> std::result::Result<HistoryStepMatrixLease, V2Error> {
        let file = fs::File::open(self.directory.join(v21_matrix_file_name(class)))
            .map_err(|_| V2Error::Matrix)?;
        if file.metadata().map_err(|_| V2Error::Matrix)?.len() > V21_MATRIX_MAX_BYTES as u64 {
            return Err(V2Error::Matrix);
        }
        let mut decoder = zstd::stream::read::Decoder::new(file).map_err(|_| V2Error::Matrix)?;
        decoder.window_log_max(27).map_err(|_| V2Error::Matrix)?;
        let mut bytes = Vec::new();
        decoder
            .take(V21_MATRIX_MAX_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| V2Error::Matrix)?;
        if bytes.len() > V21_MATRIX_MAX_BYTES {
            return Err(V2Error::Matrix);
        }
        let matrix = CompactFieldR1cs::open(
            bytes.into_boxed_slice(),
            self.bank.config().class(class).shape(),
            self.bank.matrix_digest(class),
        )
        .map_err(|_| V2Error::Matrix)?;
        Ok(HistoryStepMatrixLease::Compact(Arc::new(matrix)))
    }
}

struct Fixture {
    state: ChainState,
    genesis: BlockHeader,
    block: Block,
    start: noid_recursive::ChainAccumulator,
    ghost: PreparedHistoryStepGhostAuthorization,
}
impl Fixture {
    fn new() -> Result<Self> {
        let state = ChainState::new();
        eprintln!("RETRO BOOMER V1: empty state ready");
        let genesis = noid_chain::consensus::genesis_header();
        noid_chain::consensus::validate_pow(&genesis).map_err(error)?;
        // Qualification uses a real owner preimage. This public fixture key
        // is never a node default and must not be used to receive real funds.
        let fixture_secret = noid_poseidon2b::primitives::SpendSecret::from_bytes([1; 32]);
        let miner = noid_poseidon2b::primitives::derive_address(&fixture_secret);
        let mut block = noid_chain::consensus::build_block_template(
            &genesis,
            &state,
            &[0],
            vec![],
            miner,
            genesis.timestamp + noid_chain::consensus::params::BLOCK_TIME,
            noid_chain::consensus::params::GENESIS_TARGET,
        )
        .map_err(error)?
        .into_block(0);
        eprintln!("RETRO BOOMER V1: mining block 1 with original PoW");
        block.header.nonce = noid_chain::consensus::search_pow(&block.header, 0, 100_000_000)
            .ok_or("RETRO BOOMER qualification PoW search exhausted")?;
        noid_chain::consensus::validate_pow(&block.header).map_err(error)?;
        eprintln!("RETRO BOOMER V1: proving canonical V4 padding authorization");
        let version = noid_tx::OwnerAuthorizationVersion::V4;
        let ghost = noid_recursive::prepare_history_step_ghost_authorization_for_version(
            noid_gkr::ghost_tx::prove_selected_ghost_authorization_for_version(version)
                .map_err(error)?,
            version,
        )
        .map_err(error)?;
        Ok(Self {
            state,
            genesis,
            block,
            start: noid_recursive::genesis_accumulator(),
            ghost,
        })
    }
    fn input<const PAGES: usize>(&self, config: V2Config) -> Result<HistoryStepBlockInput<PAGES>> {
        self.input_for_block(&self.block, config)
    }
    fn pow_input<const PAGES: usize>(
        &self,
        config: V2Config,
    ) -> Result<HistoryStepBlockInput<PAGES>> {
        // The original two-phase API requires an unsealed nonce here; its
        // seal_nonce call below supplies the actually mined nonce and proves
        // the complete header tail. No PoW predicate is removed or relaxed.
        let mut template = self.block.clone();
        template.header.nonce = 0;
        self.input_for_block(&template, config)
    }
    fn input_for_block<const PAGES: usize>(
        &self,
        block: &Block,
        config: V2Config,
    ) -> Result<HistoryStepBlockInput<PAGES>> {
        noid_block::candidate_history::prepare_candidate_input(
            block,
            noid_block::HistoryStepPreparationContext {
                parent_header: &self.genesis,
                tx_epoch_anchor_header: &self.genesis,
                parent_state: &self.state,
                start_accumulator: &self.start,
                previous_timestamps: &[self.genesis.timestamp],
                finalized_active_counts: &[0],
                asert_anchor: &noid_chain::consensus::AnchorInfo {
                    anchor_height: self.genesis.height,
                    anchor_timestamp: self.genesis.timestamp,
                    anchor_target: self.genesis.difficulty_target,
                },
                local_time: self.block.header.timestamp,
            },
            vec![],
            &self.ghost,
            &[],
            config,
        )
        .map_err(error)
    }
}

fn runtime(
    parts: bank::RuntimeParts,
    digests: [[u8; 32]; 3],
    directory: &Path,
) -> Result<bank::Runtime> {
    let pinned = bank::Bank::pin(digests, &parts);
    let source = MatrixFiles {
        directory: directory.into(),
        bank: pinned.clone(),
    };
    bank::Runtime::new(pinned, parts, Box::new(source)).map_err(error)
}

fn write_matrix(path: &Path, matrix: &FieldR1cs) -> Result<()> {
    let temporary = path.with_extension("partial");
    let mut encoder = zstd::stream::write::Encoder::new(fs::File::create(&temporary)?, 3)?;
    matrix.write_artifact(&mut encoder).map_err(error)?;
    encoder.flush()?;
    encoder.finish()?.sync_all()?;
    fs::rename(temporary, path)?;
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let qualify = args.as_slice() == ["--qualify"];
    if !qualify && args.len() != 1 {
        return Err("usage: retro_boomer_v1_pack --qualify | NEW_OUTPUT_DIRECTORY".into());
    }
    if !qualify && cfg!(debug_assertions) {
        return Err(
            "release artifacts require a release build with production cryptographic parameters"
                .into(),
        );
    }
    let directory = if qualify {
        PathBuf::from("target/retro-boomer-v1-qualification")
    } else {
        PathBuf::from(&args[0])
    };
    if !qualify {
        fs::create_dir(&directory)?;
    }
    let config = bank::Config::retro_boomer_v1().map_err(error)?;
    eprintln!("RETRO BOOMER V1: preparing real block 1 and V4 authorization");
    let fixture = Fixture::new()?;
    eprintln!("RETRO BOOMER V1: deriving Small, Large and Native block relations");
    let blocks = [
        v2::derive_direct_block_vk(
            config.class(bank::Class::Small),
            fixture.input::<63>(config.class(bank::Class::Small))?,
        )
        .map_err(error)?,
        v2::derive_direct_block_vk(
            config.class(bank::Class::Large),
            fixture.input::<206>(config.class(bank::Class::Large))?,
        )
        .map_err(error)?,
        v2::derive_direct_block_vk(
            config.class(bank::Class::Native),
            fixture.input::<63>(config.class(bank::Class::Native))?,
        )
        .map_err(error)?,
    ];
    let mut parts = bank::derive_runtime_parts(config, blocks).map_err(error)?;
    let mut converged = false;
    for pass in 0..4 {
        let current = runtime(parts.clone(), [[0; 32]; 3], &directory)?;
        let origin = bank::Origin::from_genesis(current.bank()).map_err(error)?;
        let mut blocks = bank::Class::ALL.map(|class| parts.block_vk(class).clone());
        for class in bank::Class::ALL {
            let (vk, used) = match class {
                bank::Class::Small | bank::Class::Native => bank::derive_assembled_block_vk(
                    &current,
                    class,
                    &origin,
                    None,
                    fixture.input::<63>(config.class(class))?,
                ),
                bank::Class::Large => bank::derive_assembled_block_vk(
                    &current,
                    class,
                    &origin,
                    None,
                    fixture.input::<206>(config.class(class))?,
                ),
            }
            .map_err(error)?;
            eprintln!(
                "RETRO BOOMER V1 geometry: pass={pass} class={class:?} used={used} limit={}",
                1usize << config.class(class).outer_m()
            );
            blocks[class.index()] = vk;
        }
        if bank::Class::ALL
            .into_iter()
            .all(|class| blocks[class.index()] == *parts.block_vk(class))
        {
            converged = true;
            break;
        }
        parts = bank::derive_runtime_parts(config, blocks).map_err(error)?;
    }
    if !converged {
        return Err("RETRO BOOMER V1 integrated geometry did not converge".into());
    }
    if qualify {
        eprintln!("RETRO BOOMER V1 geometry qualified; no release artifacts generated");
        return Ok(());
    }

    let mut digests = [[0; 32]; 3];
    let current = runtime(parts.clone(), digests, &directory)?;
    let origin = bank::Origin::from_genesis(current.bank()).map_err(error)?;
    for class in bank::Class::ALL {
        eprintln!("RETRO BOOMER V1: freezing {class:?}");
        let frozen = match class {
            bank::Class::Small | bank::Class::Native => bank::assemble_frozen(
                &current,
                class,
                &origin,
                None,
                fixture.input::<63>(config.class(class))?,
            ),
            bank::Class::Large => bank::assemble_frozen(
                &current,
                class,
                &origin,
                None,
                fixture.input::<206>(config.class(class))?,
            ),
        }
        .map_err(error)?;
        if !frozen.matrix().satisfies(frozen.witness()) {
            return Err("RETRO BOOMER matrix rejects its real witness".into());
        }
        digests[class.index()] = frozen.matrix().statement_digest();
        write_matrix(
            &directory.join(v21_matrix_file_name(class)),
            frozen.matrix(),
        )?;
    }
    let current = runtime(parts.clone(), digests, &directory)?;
    let verified = bank::VerifiedOrigin::from_genesis(current.bank()).map_err(error)?;
    // Re-freezing under the actual bank pins detects accidental pin-dependent
    // matrix constants before any metadata can be treated as release material.
    for class in bank::Class::ALL {
        eprintln!("RETRO BOOMER V1: final-pin correspondence and proof for {class:?}");
        let frozen = match class {
            bank::Class::Small | bank::Class::Native => bank::assemble_frozen(
                &current,
                class,
                verified.origin(),
                None,
                fixture.input::<63>(config.class(class))?,
            ),
            bank::Class::Large => bank::assemble_frozen(
                &current,
                class,
                verified.origin(),
                None,
                fixture.input::<206>(config.class(class))?,
            ),
        }
        .map_err(error)?;
        if frozen.matrix().statement_digest() != digests[class.index()]
            || !frozen.matrix().satisfies(frozen.witness())
        {
            return Err("RETRO BOOMER final-pin matrix correspondence failed".into());
        }
        drop(frozen);
        let prepared = match class {
            bank::Class::Small | bank::Class::Native => bank::prepare_for_pow(
                &current,
                class,
                verified.origin(),
                None,
                fixture.pow_input::<63>(config.class(class))?,
            ),
            bank::Class::Large => bank::prepare_for_pow(
                &current,
                class,
                verified.origin(),
                None,
                fixture.pow_input::<206>(config.class(class))?,
            ),
        }
        .map_err(error)?;
        let built = prepared
            .seal_nonce(&current, fixture.block.header.nonce)
            .map_err(error)?;
        let terminal =
            bank::prove_built(&current, &built, &AtomicBool::new(false)).map_err(error)?;
        eprintln!("RETRO BOOMER V1: checking the real {class:?} terminal");
        let accepted = bank::verify_terminal(
            &current,
            &verified,
            &terminal,
            &fixture.block.header,
            &fixture.genesis,
        )
        .map_err(error)?;
        let expected = fixture
            .start
            .advance(&fixture.genesis, &fixture.block.header)
            .map_err(error)?;
        if accepted.accumulator() != &expected {
            return Err("verified accumulator differs from real block".into());
        }
        let mut applied = fixture.state.clone();
        let root = noid_chain::materialize_accepted_block_state(&mut applied, &fixture.block)
            .map_err(error)?;
        if root != accepted.accumulator().state_root {
            return Err("verified state materialization differs".into());
        }
        drop(applied);
        let bytes = bank::encode_terminal(&current, &terminal).map_err(error)?;
        let restored = bank::decode_terminal(&current, &bytes).map_err(error)?;
        let restored_acceptance = bank::verify_terminal(
            &current,
            &verified,
            &restored,
            &fixture.block.header,
            &fixture.genesis,
        )
        .map_err(error)?;
        if restored_acceptance.accumulator() != &expected {
            return Err("terminal round-trip changed boundary".into());
        }
        fs::write(
            directory.join(format!("verified-block1-{}.terminal", class.wire_id())),
            bytes,
        )?;
    }
    fs::write(
        directory.join(V21_METADATA_FILE),
        encode_v21_runtime_metadata(current.bank(), &parts).map_err(error)?,
    )?;
    fs::write(
        directory.join("pins.env"),
        format!(
            "RETRO_BOOMER_V1_RELEASE_BANK={}\n",
            hex::encode(current.bank().digest())
        ),
    )?;
    eprintln!("RETRO BOOMER V1 pack verified: {}", directory.display());
    Ok(())
}
