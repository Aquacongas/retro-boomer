#!/usr/bin/env bash
# Shared Retro Boomer release helpers; no publication or proof generation.
if [[ ${BASH_SOURCE[0]} == "$0" ]]; then
  printf 'Source this file from a Retro Boomer release script.\n' >&2
  exit 2
fi
RELEASE_ROOT_DIR="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)"
release_die() { printf 'error: %s\n' "$*" >&2; exit 2; }
release_require_command() { command -v "$1" >/dev/null 2>&1 || release_die "missing command: $1"; }
release_python() {
  # Captured stdout is text; normalize Windows CRLF for shell variables.
  if [[ ${OS:-} == Windows_NT ]]; then python "$@" | tr -d '\r';
  elif command -v python3 >/dev/null 2>&1; then python3 "$@" | tr -d '\r';
  else python "$@" | tr -d '\r'; fi
}
release_host() {
  RELEASE_TARGET=$(rustc -vV | sed -n 's/^host: //p')
  case "$RELEASE_TARGET" in
    x86_64-unknown-linux-gnu) RELEASE_PLATFORM=linux-x86_64 ;;
    aarch64-unknown-linux-gnu) RELEASE_PLATFORM=linux-aarch64 ;;
    x86_64-pc-windows-msvc) RELEASE_PLATFORM=windows-x86_64 ;;
    x86_64-apple-darwin) RELEASE_PLATFORM=macos-x86_64 ;;
    aarch64-apple-darwin) RELEASE_PLATFORM=macos-aarch64 ;;
    *) release_die "unsupported native release host: $RELEASE_TARGET" ;;
  esac
  export RUSTFLAGS=
  [[ $RELEASE_TARGET != x86_64-* ]] || export RUSTFLAGS='-C target-cpu=x86-64'
  unset CARGO_ENCODED_RUSTFLAGS
  [[ $RELEASE_TARGET != *-apple-darwin ]] || export MACOSX_DEPLOYMENT_TARGET=11.0
}
