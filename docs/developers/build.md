> Native materials in artifacts/retro-boomer-v1-20261031 are approved in source. The eight file hashes and packaged archive transport integrity have been checked locally; five-platform build and installer qualification remain pending. See [build and material status](../../RETRO_BOOMER_BUILD_INSTRUCTIONS.md).

# Building Retro Boomer

Use [the current WSL build guide](../../RETRO_BOOMER_BUILD_INSTRUCTIONS.md) and
[the five-platform release procedure](../../.github/RELEASING.md). These are the
authoritative local build and packaging instructions for the renamed binaries.

```bash
cargo check --offline --locked -p noid_node -p noid_gui -p noid-extminer
```

Production builds require the approved existing bank and full build-time matrix
authentication. There is one bank with Small, Large and Native classes. Do not
execute old three-bank generation or retirement instructions. No production
release is certified by this guide. The original upstream build reference is
preserved [as historical material](../upstream/legacy-build-guide.md).

The standard native release command does not need --output or a source timestamp:

```bash
CARGO_BUILD_JOBS=8 ./scripts/build_release.sh \
  --v1-pack artifacts/retro-boomer-v1-20261031 \
  --v1-pins artifacts/retro-boomer-v1-20261031/pins.env \
  --with-gui
```

Its output is target/release/. Explicit --output remains the release-candidate
packaging mode and uses separate staging. Both modes authenticate the same bank.
