#!/usr/bin/env bash
# Build Retro Boomer locally, or stage one native candidate; never publish or generate proofs.
set -Eeuo pipefail
IFS=$'\n\t'
source "$(dirname -- "${BASH_SOURCE[0]}")/release_common.sh"
PACK= PINS= OUTPUT= FETCH=0
while (($#)); do
  case "$1" in
    --v1-pack) PACK=${2:?missing bank directory}; shift 2 ;;
    --v1-pins) PINS=${2:?missing reviewed pin file}; shift 2 ;;
    --output) OUTPUT=${2:?missing fresh output directory}; shift 2 ;;
    --with-gui) shift ;; # Retained compatibility; official native products always include GUI.
    --fetch) FETCH=1; shift ;;
    *) release_die 'Usage: build_release.sh --v1-pack DIR --v1-pins FILE [--output NEW_DIR] [--with-gui] [--fetch]' ;;
  esac
done
[[ -n $PACK && -n $PINS ]] || release_die 'A real V1 pack and independent pin file are required.'
[[ -d $PACK && ! -L $PACK && -f $PINS && ! -L $PINS ]] || release_die 'Invalid bank directory or pin file.'
PACK=$(cd -- "$PACK" && pwd -P)
PINS=$(release_python -c 'import pathlib,sys; print(pathlib.Path(sys.argv[1]).resolve())' "$PINS")
cd -- "$RELEASE_ROOT_DIR"
# Parse, never source or evaluate, the reviewed pin. Check it before any Cargo invocation.
release_python - "$PINS" <<'PY'
import sys
sys.path.insert(0, 'scripts')
import release_material as material
try:
    material.reviewed_pin_file(sys.argv[1], material.policy())
except (OSError, ValueError) as error:
    raise SystemExit(str(error))
PY
if [[ -f $PACK/SHA256SUMS ]]; then
  release_python scripts/release_material.py verify --root "$PACK"
else
  release_python scripts/release_material.py raw --root "$PACK"
fi
for obsolete in NOID_HISTORY_STEP_PACK_DIR NOID_HISTORY_STEP_RUNTIME_METADATA_RELEASE_DIGEST NOID_V2_PACK_DIR NOID_V2_RELEASE_BANK NOID_V21_PACK_DIR NOID_V21_RELEASE_BANK NOID_AUTHORIZATION_PACK_DIR NOID_AUTHORIZATION_RELEASE_BANK NOID_RETIREMENT_KEYS_DIR NOID_RETIREMENT_KEY_0_PIN NOID_RETIREMENT_KEY_1_PIN; do
  [[ -z ${!obsolete+x} ]] || release_die "historical proof environment is forbidden: $obsolete"
done
# Always use the full build.rs semantic matrix scan, never caller-supplied receipts.
unset NOID_PRECHECKED_MATRIX_PACKS NOID_V21_MATRIX_CHECKS_SHA256
export RETRO_BOOMER_V1_PACK_DIR=$PACK
RETRO_BOOMER_V1_RELEASE_BANK=$(release_python -c 'import json; print(json.load(open("scripts/release/retro_boomer_material.json"))["bank"])')
export RETRO_BOOMER_V1_RELEASE_BANK
for command in cargo rustc; do release_require_command "$command"; done
release_host
TOOLCHAIN=$(release_python scripts/release/release_products.py source-value --name toolchain)
VERSION=$(release_python scripts/release/release_products.py source-value --name version)
[[ $(rustc --version) == "rustc $TOOLCHAIN "* ]] || release_die "use the pinned Rust toolchain: $TOOLCHAIN"
CARGO_RELEASE_ARGS=(--offline --locked --release --timings)
if [[ -n $OUTPUT ]]; then
  [[ ! -e $OUTPUT && ! -L $OUTPUT ]] || release_die 'Output already exists; refusing to overwrite or delete it.'
  [[ ${SOURCE_DATE_EPOCH:-} =~ ^[1-9][0-9]*$ ]] || release_die 'Set SOURCE_DATE_EPOCH to the reviewed source commit timestamp.'
  export SOURCE_DATE_EPOCH
  mkdir -p -- "$OUTPUT"
  OUTPUT=$(cd -- "$OUTPUT" && pwd -P)
  export CARGO_TARGET_DIR="$OUTPUT/work/target"
  CARGO_RELEASE_ARGS+=(--target "$RELEASE_TARGET")
  BIN_DIR="$CARGO_TARGET_DIR/$RELEASE_TARGET/release"
  mkdir -p "$OUTPUT/products" "$OUTPUT/work"
else
  # Ordinary native builds keep the established workspace target/release path.
  # CARGO_BUILD_JOBS is inherited unchanged; candidate-only tools are not run.
  export CARGO_TARGET_DIR="$RELEASE_ROOT_DIR/target"
  unset CARGO_BUILD_TARGET
  BIN_DIR="$CARGO_TARGET_DIR/release"
fi
if ((FETCH)); then cargo fetch --locked --target "$RELEASE_TARGET"; fi
cargo build "${CARGO_RELEASE_ARGS[@]}" \
  -p noid_node --bin retro-boomerd --bin retro-boomer-cli
cargo build "${CARGO_RELEASE_ARGS[@]}" \
  -p noid-extminer --bin retro-boomer-miner
cargo build "${CARGO_RELEASE_ARGS[@]}" \
  -p noid_gui --bin retro-boomer-gui
if [[ -z $OUTPUT ]]; then
  printf 'Local release binaries: %s\nNo installers, runtime smoke tests or publication were requested.\n' "$BIN_DIR"
  exit 0
fi
EXE=
[[ $RELEASE_PLATFORM != windows-* ]] || EXE=.exe
if [[ $RELEASE_PLATFORM == windows-* && -n ${RETRO_BOOMER_WINDOWS_SIGN_THUMBPRINT:-} ]]; then
  pwsh -NoProfile -File scripts/release/sign_windows.ps1 -BinDir "$BIN_DIR"
fi
if [[ $RELEASE_PLATFORM == macos-* && -n ${RETRO_BOOMER_MACOS_SIGN_IDENTITY:-} && ${RETRO_BOOMER_MACOS_SIGN_IDENTITY:-} != - ]]; then
  for binary in retro-boomerd retro-boomer-cli retro-boomer-miner retro-boomer-gui; do
    codesign --force --options runtime --timestamp --sign "$RETRO_BOOMER_MACOS_SIGN_IDENTITY" "$BIN_DIR/$binary"
    codesign --verify --strict "$BIN_DIR/$binary"
  done
fi
release_python scripts/release/release_products.py check-binaries --bin-dir "$BIN_DIR" --platform "$RELEASE_PLATFORM"
"$BIN_DIR/retro-boomerd$EXE" --check-hardware
"$BIN_DIR/retro-boomerd$EXE" --help >/dev/null
"$BIN_DIR/retro-boomer-cli$EXE" --help >/dev/null
"$BIN_DIR/retro-boomer-miner$EXE" --check-hardware
"$BIN_DIR/retro-boomer-miner$EXE" --help >/dev/null
"$BIN_DIR/retro-boomer-gui$EXE" --release-self-check
release_python scripts/release/smoke_node.py --node "$BIN_DIR/retro-boomerd$EXE" --output "$OUTPUT/work/smoke.json"
RECORD="$OUTPUT/products/BUILD-$RELEASE_PLATFORM.json"
release_python scripts/release/release_products.py record --bin-dir "$BIN_DIR" --platform "$RELEASE_PLATFORM" \
  --target-dir "$CARGO_TARGET_DIR" --smoke "$OUTPUT/work/smoke.json" --output "$RECORD"
release_python scripts/release/release_products.py core --bin-dir "$BIN_DIR" --platform "$RELEASE_PLATFORM" \
  --record "$RECORD" --output "$OUTPUT/products"
case "$RELEASE_PLATFORM" in
  linux-*) bash scripts/release/package_linux_gui.sh "$BIN_DIR" "$OUTPUT/products" "$VERSION" "$RELEASE_PLATFORM" ;;
  macos-*) bash scripts/release/package_macos_gui.sh "$BIN_DIR" "$OUTPUT/products" "$VERSION" "$RELEASE_PLATFORM" ;;
  windows-*) pwsh -NoProfile -File scripts/release/package_windows_gui.ps1 -BinDir "$BIN_DIR" -OutputDir "$OUTPUT/products" -Version "$VERSION" -Platform "$RELEASE_PLATFORM" ;;
esac
release_python - "$OUTPUT/products" "$RELEASE_PLATFORM" <<'PY'
import pathlib,sys
sys.path.insert(0,'scripts/release')
import release_products as rp
p=pathlib.Path(sys.argv[1]); platform=sys.argv[2]
expected=set(rp.names(platform,rp.material.policy()['version']))|{f'BUILD-{platform}.json'}
rp.require({f.name for f in p.iterdir()}==expected,'unexpected or missing native product')
(p.parent/'SHA256SUMS').write_text(rp.checksums(p,expected),encoding='ascii')
PY
printf 'Native candidate products: %s/products\nNothing was published.\n' "$OUTPUT"
