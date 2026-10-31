#!/usr/bin/env bash
# Publication is intentionally disabled during local Retro Boomer preparation.
# No external commands, credentials, Git operations or network access are used.
set -euo pipefail
printf '%s\n' \
  'Retro Boomer publication is disabled.' \
  'The upstream Parano1d publisher has been removed from this entry point.' \
  'No code, tags, workflows or release assets have been sent anywhere.' \
  'A separately reviewed Retro Boomer release process is required before publication.' >&2
exit 2
