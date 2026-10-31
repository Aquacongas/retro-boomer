// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2026 Paranoid Zero.

//! Authenticate the complete RETRO BOOMER V1 matrix bank before embedding its bytes.
use std::{
    env,
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
};
mod matrix_checks;
const V21_PACK_ENV: &str = "RETRO_BOOMER_V1_PACK_DIR";
const V21_BANK_ENV: &str = "RETRO_BOOMER_V1_RELEASE_BANK";
// Historical no-burn bank for the former 210-million schedule; never approve
// it for the new ten-billion schedule. Keep its pin for reference.
const PREVIOUS_SUBSIDY_BANK: [u8; 32] = [
    0x71, 0x43, 0xe7, 0x1c, 0x32, 0x12, 0x36, 0x54, 0xfa, 0x41, 0xef, 0x33, 0x01, 0x5c, 0xc3, 0x7f,
    0x6a, 0x53, 0xae, 0x06, 0xf8, 0x8d, 0xaf, 0x6e, 0x0d, 0x48, 0x21, 0x36, 0x24, 0x35, 0x61, 0xee,
];

// Native RETRO BOOMER proof-boundary materials require independent generation and approval.
// None deliberately blocks production embedding; historical materials are not approved.
const RETRO_BOOMER_V1_APPROVED_BANK: Option<[u8; 32]> = Some([0x84, 0x1c, 0x49, 0x6f, 0x9b, 0x1b, 0x24, 0xdb, 0x58, 0x6f, 0xb3, 0xa9, 0x76, 0xe2, 0xc2, 0x01, 0xc3, 0x35, 0x63, 0x4b, 0x84, 0xa9, 0x11, 0x93, 0xbb, 0xe4, 0xe4, 0xf5, 0xd7, 0x53, 0x7f, 0x58]);

fn main() {
    for name in [
        V21_PACK_ENV,
        V21_BANK_ENV,
        "NOID_PRECHECKED_MATRIX_PACKS",
        "NOID_V21_MATRIX_CHECKS_SHA256",
    ] {
        println!("cargo:rerun-if-env-changed={name}");
    }
    for name in [
        "NOID_HISTORY_STEP_PACK_DIR",
        "NOID_HISTORY_STEP_RUNTIME_METADATA_RELEASE_DIGEST",
        "NOID_V2_PACK_DIR",
        "NOID_V2_RELEASE_BANK",
        "NOID_AUTHORIZATION_PACK_DIR",
        "NOID_AUTHORIZATION_RELEASE_BANK",
        "NOID_RETIREMENT_KEYS_DIR",
        "NOID_RETIREMENT_KEY_0_PIN",
        "NOID_RETIREMENT_KEY_1_PIN",
        "NOID_PRECHECKED_MATRIX_PACKS",
        "NOID_V21_MATRIX_CHECKS_SHA256",
    ] {
        println!("cargo:rerun-if-env-changed={name}");
        assert!(
            env::var_os(name).is_none(),
            "RETRO BOOMER V1 does not accept historical material: {name}"
        );
    }
    assert!(
        env::var_os("CARGO_FEATURE_RETIRED_HISTORY").is_none(),
        "RETRO BOOMER V1 has no predecessor-chain retirement profile"
    );
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo sets OUT_DIR"));
    embed_v21_pack(&out, env::var("PROFILE").as_deref() == Ok("release"));
}

fn embed_v21_pack(out_directory: &Path, required: bool) {
    use noid_miner::v21_artifacts::*;
    use noid_recursive::acceptance::history_step::v2::capacity_banked::Class;
    let generated = out_directory.join("v21_pack.rs");
    match (env::var_os(V21_PACK_ENV), env::var_os(V21_BANK_ENV)) {
        (None, None) => {
            assert!(
                !required,
                "RETRO BOOMER V1 release builds require the approved ten-billion subsidy bank, {V21_PACK_ENV} and {V21_BANK_ENV}"
            );
            write_if_changed(
                &generated,
                b"static GENERATED_V21_PACK: Option<EmbeddedV21Pack> = None;\n",
            );
        }
        (Some(directory), Some(pin)) => {
            let bank = parse_hex_digest(&pin.into_string().expect("UTF-8 v2.1 bank"), V21_BANK_ENV);
            assert_ne!(bank, OBSOLETE_BURN_BANK, "obsolete RETRO BOOMER bank");
            assert_ne!(bank, PREVIOUS_SUBSIDY_BANK,
                "the former 210-million subsidy bank is incompatible with ten-billion RETRO BOOMER V1");
            let approved = RETRO_BOOMER_V1_APPROVED_BANK.expect(
                "ten-billion RETRO BOOMER V1 has no approved HistoryStep bank in this source"
            );
            assert_eq!(
                bank, approved,
                "RETRO BOOMER V1 bank differs from the approved source pin"
            );
            let directory = PathBuf::from(directory);
            let metadata_path = directory.join(V21_METADATA_FILE);
            let metadata = read_bounded(&metadata_path, V21_METADATA_MAX_BYTES as u64);
            let runtime_metadata = decode_v21_runtime_metadata_pinned(&metadata, bank)
                .expect("pinned v2.1 runtime metadata");
            assert_eq!(
                runtime_metadata.bank().config(),
                noid_recursive::acceptance::history_step::v2::capacity_banked::Config::retro_boomer_v1()
                    .expect("source RETRO BOOMER configuration"),
                "matrix metadata does not describe source RETRO BOOMER V1"
            );
            assert_eq!(
                runtime_metadata.bank().config().schedule(),
                noid_chain::consensus::forks::ACTIVE_SCHEDULE,
                "v2.1 artifact schedule differs from the executable profile"
            );
            let activation = noid_chain::consensus::params::V2_1_ACTIVATION_HEIGHT;
            assert!(
                activation.is_some(),
                "v2.1 artifacts require a scheduled activation"
            );
            for class in Class::ALL {
                assert_eq!(
                    runtime_metadata
                        .bank()
                        .config()
                        .class(class)
                        .authorization_activation(),
                    activation,
                    "v2.1 artifact activation differs from the executable profile"
                );
            }
            println!("cargo:rerun-if-changed={}", metadata_path.display());
            let staged = out_directory.join("embedded-v21");
            fs::create_dir_all(&staged).expect("v2.1 staging directory");
            write_if_changed(&staged.join(V21_METADATA_FILE), &metadata);
            let mut sources = Vec::new();
            let mut seals = Vec::new();
            let mut checks = matrix_checks::PackChecks::new(
                &directory,
                "v21",
                bank,
                &metadata,
                Class::ALL.len(),
            );
            for class in Class::ALL {
                let name = v21_matrix_file_name(class);
                let path = directory.join(name);
                let compressed = read_bounded(&path, V21_MATRIX_MAX_BYTES as u64);
                let seal = checks.authenticate(
                    name,
                    &compressed,
                    runtime_metadata.bank().config().class(class).shape(),
                    runtime_metadata.bank().matrix_digest(class),
                    || {
                        runtime_metadata
                            .preflight_build_matrix(class, &compressed)
                            .expect("v2.1 matrix semantic authentication before embedding")
                    },
                );
                println!("cargo:rerun-if-changed={}", path.display());
                write_if_changed(&staged.join(name), &compressed);
                sources.push(format!(
                    "include_bytes!(concat!(env!(\"OUT_DIR\"), \"/embedded-v21/{name}\"))"
                ));
                seals.push(format!(
                    "unsafe {{ noid_ivc_core::field_r1cs::BuildAuthenticatedFieldR1csSeal::from_release_build(noid_ivc_core::proof::FieldShape {{ m: {}, k_log: {}, k_skip: {}, const_pin: {} }}, {}, {}) }}",
                    seal.shape().m, seal.shape().k_log, seal.shape().k_skip,
                    render_const_pin(seal.shape().const_pin), render_digest(seal.statement_digest()), seal.canonical_bytes(),
                ));
            }
            checks.write(&staged);
            write_if_changed(&generated, format!(
                "static GENERATED_V21_PACK: Option<EmbeddedV21Pack> = Some(EmbeddedV21Pack {{\n\
                metadata: include_bytes!(concat!(env!(\"OUT_DIR\"), \"/embedded-v21/{V21_METADATA_FILE}\")),\n\
                compressed_matrices: [{}],\n\
                release_bank: {},\n\
                build_seals: [{}],\n}});\n", sources.join(", "), render_digest(bank), seals.join(", ")
            ).as_bytes());
        }
        _ => panic!("{V21_PACK_ENV} and {V21_BANK_ENV} must be set together"),
    }
}

fn read_bounded(path: &Path, max_bytes: u64) -> Vec<u8> {
    let metadata = fs::metadata(path)
        .unwrap_or_else(|error| panic!("inspect release artifact {}: {error}", path.display()));
    assert!(
        metadata.is_file() && metadata.len() <= max_bytes,
        "release artifact {} is not a bounded regular file",
        path.display()
    );
    fs::read(path)
        .unwrap_or_else(|error| panic!("read release artifact {}: {error}", path.display()))
}

fn parse_hex_digest(encoded: &str, variable: &str) -> [u8; 32] {
    assert_eq!(
        encoded.len(),
        64,
        "{variable} must be exactly 64 lowercase hexadecimal characters"
    );
    let mut digest = [0u8; 32];
    for (index, byte) in digest.iter_mut().enumerate() {
        let high = decode_lower_hex(encoded.as_bytes()[index * 2])
            .unwrap_or_else(|| panic!("{variable} is not lowercase hexadecimal"));
        let low = decode_lower_hex(encoded.as_bytes()[index * 2 + 1])
            .unwrap_or_else(|| panic!("{variable} is not lowercase hexadecimal"));
        *byte = (high << 4) | low;
    }
    digest
}

const fn decode_lower_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

fn render_const_pin(pin: Option<usize>) -> String {
    pin.map_or_else(|| "None".to_owned(), |column| format!("Some({column})"))
}

fn render_digest(digest: [u8; 32]) -> String {
    let mut rendered = String::from("[");
    for (index, byte) in digest.iter().enumerate() {
        if index != 0 {
            rendered.push_str(", ");
        }
        write!(&mut rendered, "0x{byte:02x}").expect("writing to String cannot fail");
    }
    rendered.push(']');
    rendered
}

fn write_if_changed(path: &Path, bytes: &[u8]) {
    if matches!(fs::read(path), Ok(existing) if existing == bytes) {
        return;
    }
    fs::write(path, bytes)
        .unwrap_or_else(|error| panic!("write generated artifact {}: {error}", path.display()));
}
