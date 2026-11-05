# RETRO BOOMER — native material generation and subsequent builds

## Current state

Native materials are present in artifacts/retro-boomer-v1-20261031 and approved in noid_node/build.rs with bank pin 841c496f9b1b24db586fb3a976e2c201c335634b84a91193bbe4e4f5d7537f58. All eight source SHA-256 values match the committed policy. The existing packaging and extraction/manifest procedures verified dist/materials-v1.0.1/retro-boomer-history-step-pack-v1.0.1.tar.gz (55270717 bytes; SHA-256 7574cde0dc36f25bea5bc8ee084f047caf20702a5034f169116c1baff7b962d3). The eight archived payloads are byte-identical to the approved v1.0.0 files, with the bank pin and original generation provenance preserved. No proofs or matrices were regenerated during packaging. Generation verification is owner-reported; mathematical terminal verification was not repeated during packaging. Five-platform builds, native installer qualification and signing remain unverified. Generation commit and independently authenticated generation-log attestation remain unknown.

## Prerequisites

Use the project root, Bash, Rust 1.96.0 from rust-toolchain.toml, Cargo.lock and a complete cached dependency set: the existing build uses --offline --locked. The build environment requires the native C/C++ toolchain and applicable workspace build dependencies, including Clang/libclang, CMake and pkg-config. No dependency installation is performed by the command below. Ensure sufficient free RAM, CPU time and disk space for real Small, Large and Native relations and proofs; no resource measurements have been made for these rebranded sources.

## Approved material reuse

Use the existing approved directory. Do not regenerate it for a build or upload. The archived payload includes Small, Large and Native matrices, runtime metadata, pins.env and three block-one terminals. The pin variable is RETRO_BOOMER_V1_RELEASE_BANK. Existing v21 filenames describe the retained original material format.

The distribution archive is already created and verified. Its committed archive_sha256 is the source of truth for transport. Use scripts/release_material.py extract with that digest and a fresh extraction destination; full build-time semantic matrix authentication remains mandatory. See .github/RELEASING.md for the exact upload and provisioning procedure.

## Local build — only after native approval

```bash
cd /home/oms/retro-boomer
CARGO_BUILD_JOBS=8 ./scripts/build_release.sh \
  --v1-pack artifacts/retro-boomer-v1-20261031 \
  --v1-pins artifacts/retro-boomer-v1-20261031/pins.env \
  --with-gui
```

The existing script preserves the release profile, --offline --locked and CARGO_BUILD_JOBS. Without --output it produces target/release/retro-boomerd, retro-boomer-cli, retro-boomer-miner and retro-boomer-gui. --output remains optional for candidate staging. Missing, altered or unapproved material is rejected before production embedding.

See [.github/RELEASING.md](.github/RELEASING.md) for the five-platform pipeline. No generation, build, node launch, mining, upload or publication is authorized by this document.
