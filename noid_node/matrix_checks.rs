use std::{fs, path::Path};

use noid_ivc_core::{field_r1cs::BuildAuthenticatedFieldR1csSeal, proof::FieldShape};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const FILE: &str = "matrix-checks.json";

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Shape {
    m: usize,
    k_log: usize,
    k_skip: usize,
    const_pin: Option<usize>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct MatrixCheck {
    file: String,
    compressed_sha256: String,
    statement_digest: String,
    canonical_bytes: usize,
    shape: Shape,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: u8,
    bank: String,
    metadata_sha256: String,
    matrices: Vec<MatrixCheck>,
}

pub struct PackChecks {
    result: Manifest,
    classes: usize,
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn sha256(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

impl PackChecks {
    pub fn new(
        _directory: &Path,
        _label: &str,
        bank: [u8; 32],
        metadata: &[u8],
        classes: usize,
    ) -> Self {
        let result = Manifest {
            schema: 1,
            bank: hex(&bank),
            metadata_sha256: sha256(metadata),
            matrices: Vec::new(),
        };
        Self { result, classes }
    }

    pub fn authenticate(
        &mut self,
        file: &str,
        compressed: &[u8],
        shape: FieldShape,
        statement_digest: [u8; 32],
        full_check: impl FnOnce() -> BuildAuthenticatedFieldR1csSeal,
    ) -> BuildAuthenticatedFieldR1csSeal {
        let compressed_sha256 = sha256(compressed);
        let digest = hex(&statement_digest);
        // Every TAPE build authenticates the canonical rows. A caller-supplied
        // receipt/checksum cannot establish the matrix statement digest.
        let seal = full_check();
        self.result.matrices.push(MatrixCheck {
            file: file.to_owned(),
            compressed_sha256,
            statement_digest: digest,
            canonical_bytes: seal.canonical_bytes(),
            shape: Shape {
                m: shape.m as usize,
                k_log: shape.k_log as usize,
                k_skip: shape.k_skip as usize,
                const_pin: shape.const_pin,
            },
        });
        seal
    }

    pub fn write(&self, directory: &Path) {
        assert_eq!(
            self.result.matrices.len(),
            self.classes,
            "complete matrix preflight"
        );
        let bytes = serde_json::to_vec_pretty(&self.result).expect("serialize matrix checks");
        fs::write(directory.join(FILE), bytes).expect("write matrix checks");
    }
}
