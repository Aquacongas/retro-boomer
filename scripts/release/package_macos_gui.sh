#!/usr/bin/env bash

set -Eeuo pipefail
IFS=$'\n\t'
umask 022

SCRIPT_DIR="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
RELEASE_ROOT_DIR="$(CDPATH='' cd -- "$SCRIPT_DIR/../.." && pwd -P)"

usage() {
  cat <<'EOF'
Usage: package_macos_gui.sh BIN_DIR OUTPUT_DIR VERSION PLATFORM

Build a native Retro Boomer .app and compressed DMG.
PLATFORM must be macos-aarch64 or macos-x86_64.
EOF
}

if (( $# != 4 )); then
  usage >&2
  exit 2
fi

BIN_DIR=$1
OUTPUT_DIR=$2
VERSION=$3
PLATFORM=$4
[[ $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+([+-][0-9A-Za-z.-]+)?$ ]] || {
  echo "invalid semantic version: $VERSION" >&2
  exit 1
}
case "$PLATFORM" in
  macos-aarch64|macos-x86_64) ;;
  *)
    echo "unsupported macOS GUI platform: $PLATFORM" >&2
    exit 1
    ;;
esac

for command in codesign hdiutil iconutil install mktemp plutil python3 sed; do
  command -v "$command" >/dev/null 2>&1 || {
    echo "required command is missing: $command" >&2
    exit 1
  }
done

BIN_DIR="$(CDPATH='' cd -- "$BIN_DIR" && pwd -P)"
mkdir -p -- "$OUTPUT_DIR"
OUTPUT_DIR="$(CDPATH='' cd -- "$OUTPUT_DIR" && pwd -P)"
for binary in retro-boomer-gui retro-boomerd retro-boomer-miner; do
  [[ -f $BIN_DIR/$binary && -x $BIN_DIR/$binary ]] || {
    echo "release binary is missing or not executable: $BIN_DIR/$binary" >&2
    exit 1
  }
done

TEMPORARY=$(mktemp -d "${TMPDIR:-/tmp}/retro-boomer-macos-gui.XXXXXX")
MOUNTED=0
cleanup() {
  local status=$?
  if [[ $MOUNTED == 1 ]]; then
    hdiutil detach "$TEMPORARY/mount" -quiet || true
  fi
  if [[ -d $TEMPORARY && $TEMPORARY == "${TMPDIR:-/tmp}"/retro-boomer-macos-gui.* ]]; then
    rm -r -- "$TEMPORARY" || true
  fi
  exit "$status"
}
trap cleanup EXIT

APP="$TEMPORARY/Retro Boomer.app"
CONTENTS="$APP/Contents"
MACOS="$CONTENTS/MacOS"
RESOURCES="$CONTENTS/Resources"
ICONSET="$TEMPORARY/RetroBoomer.iconset"
DMG_ROOT="$TEMPORARY/dmg"
mkdir -p -- "$MACOS" "$RESOURCES" "$ICONSET" "$DMG_ROOT"

install -m 0755 "$BIN_DIR/retro-boomer-gui" "$MACOS/retro-boomer-gui"
install -m 0755 "$BIN_DIR/retro-boomerd" "$MACOS/retro-boomerd"
install -m 0755 "$BIN_DIR/retro-boomer-miner" "$MACOS/retro-boomer-miner"
install -m 0644 "$RELEASE_ROOT_DIR/LICENSE" "$RESOURCES/LICENSE.txt"
install -m 0644 "$RELEASE_ROOT_DIR/NOTICE" "$RESOURCES/NOTICE.txt"
install -m 0644 "$RELEASE_ROOT_DIR/noid_gui/assets/fonts/OFL-1.1.txt" "$RESOURCES/OFL-1.1.txt"
install -m 0644 "$SCRIPT_DIR/README.txt" "$RESOURCES/README.txt"
install -m 0644 "$RELEASE_ROOT_DIR/docs/reference/contracts.md" "$RESOURCES/CONTRACTS.md"

BUNDLE_VERSION=${VERSION%%[-+]*}
sed \
  -e "s/@VERSION@/$VERSION/g" \
  -e "s/@BUNDLE_VERSION@/$BUNDLE_VERSION/g" \
  "$SCRIPT_DIR/gui/macos/Info.plist.in" \
  > "$CONTENTS/Info.plist"
plutil -lint "$CONTENTS/Info.plist" >/dev/null

for specification in \
  "16 icon_16x16.png" \
  "32 icon_16x16@2x.png" \
  "32 icon_32x32.png" \
  "64 icon_32x32@2x.png" \
  "128 icon_128x128.png" \
  "256 icon_128x128@2x.png" \
  "256 icon_256x256.png" \
  "512 icon_256x256@2x.png" \
  "512 icon_512x512.png" \
  "1024 icon_512x512@2x.png"
do
  size=${specification%% *}
  name=${specification#* }
  icon="$RELEASE_ROOT_DIR/noid_gui/assets/app-icons/retro-boomer-${size}.png"
  [[ -f $icon ]] || {
    echo "macOS icon source is missing: $icon" >&2
    exit 1
  }
  install -m 0644 "$icon" "$ICONSET/$name"
done
iconutil -c icns "$ICONSET" -o "$RESOURCES/RetroBoomer.icns"

xattr -cr "$APP"
SIGN_IDENTITY=${RETRO_BOOMER_MACOS_SIGN_IDENTITY:--}
if [[ $SIGN_IDENTITY == - ]]; then
  codesign --force --sign - --timestamp=none "$MACOS/retro-boomerd"
  codesign --force --sign - --timestamp=none "$MACOS/retro-boomer-miner"
  codesign --force --sign - --timestamp=none "$MACOS/retro-boomer-gui"
  codesign --force --deep --sign - --timestamp=none "$APP"
else
  codesign --force --options runtime --timestamp --sign "$SIGN_IDENTITY" "$MACOS/retro-boomerd"
  codesign --force --options runtime --timestamp --sign "$SIGN_IDENTITY" "$MACOS/retro-boomer-miner"
  codesign --force --options runtime --timestamp --sign "$SIGN_IDENTITY" "$MACOS/retro-boomer-gui"
  codesign --force --deep --options runtime --timestamp --sign "$SIGN_IDENTITY" "$APP"
fi
codesign --verify --deep --strict "$APP"
"$MACOS/retro-boomer-gui" --release-self-check >/dev/null
"$MACOS/retro-boomerd" --check-hardware >/dev/null

mv "$APP" "$DMG_ROOT/Retro Boomer.app"
ln -s /Applications "$DMG_ROOT/Applications"
ARTIFACT="$OUTPUT_DIR/retro-boomer-gui-v${VERSION}-${PLATFORM}.dmg"
IMAGE_SIZE_MIB=$(python3 "$SCRIPT_DIR/dmg_size.py" "$DMG_ROOT")
hdiutil create \
  -volname "Retro Boomer" \
  -srcfolder "$DMG_ROOT" \
  -fs HFS+ \
  -size "${IMAGE_SIZE_MIB}m" \
  -format UDZO \
  -ov \
  "$ARTIFACT" >/dev/null
# Optional: a preconfigured real keychain profile and Developer ID are required.
if [[ -n ${RETRO_BOOMER_MACOS_NOTARY_PROFILE:-} ]]; then
  [[ $SIGN_IDENTITY != - ]] || { echo 'Notarization requires a real signing identity.' >&2; exit 1; }
  xcrun notarytool submit "$ARTIFACT" --keychain-profile "$RETRO_BOOMER_MACOS_NOTARY_PROFILE" --wait
  xcrun stapler staple "$ARTIFACT"
  xcrun stapler validate "$ARTIFACT"
fi
hdiutil verify "$ARTIFACT" >/dev/null
mkdir "$TEMPORARY/mount"
hdiutil attach -readonly -nobrowse -mountpoint "$TEMPORARY/mount" "$ARTIFACT" >/dev/null
MOUNTED=1
"$TEMPORARY/mount/Retro Boomer.app/Contents/MacOS/retro-boomer-gui" \
  --release-self-check >/dev/null
"$TEMPORARY/mount/Retro Boomer.app/Contents/MacOS/retro-boomerd" \
  --check-hardware >/dev/null
[[ -s "$TEMPORARY/mount/Retro Boomer.app/Contents/Resources/LICENSE.txt" ]]
[[ -s "$TEMPORARY/mount/Retro Boomer.app/Contents/Resources/NOTICE.txt" ]]
[[ -s "$TEMPORARY/mount/Retro Boomer.app/Contents/Resources/OFL-1.1.txt" ]]
[[ -s "$TEMPORARY/mount/Retro Boomer.app/Contents/Resources/README.txt" ]]
[[ -s "$TEMPORARY/mount/Retro Boomer.app/Contents/Resources/CONTRACTS.md" ]]
[[ -s "$TEMPORARY/mount/Retro Boomer.app/Contents/MacOS/retro-boomerd" ]]
[[ ! -e "$TEMPORARY/mount/Retro Boomer.app/Contents/MacOS/retro-boomer-cli" ]]
[[ -x "$TEMPORARY/mount/Retro Boomer.app/Contents/MacOS/retro-boomer-miner" ]]
hdiutil detach "$TEMPORARY/mount" -quiet
MOUNTED=0
printf '%s\n' "$ARTIFACT"
