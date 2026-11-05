# Retro Boomer v1.0.1 release process

Publisher: Retro Boomer. Contact: aquacongass@proton.me.
Repository: https://github.com/Aquacongas/retro-boomer.

This process adapts the upstream Parano1d release architecture for RETRO BOOMER.
It does not publish to upstream repositories or mirrors.
`scripts/publish_release.sh` remains unconditionally disabled.

## Boundaries and current qualification

Stage 2 prepares local scripts, workflows and one transport archive. No native
build, semantic matrix authentication, terminal verification, node, miner,
GitHub operation or public release is executed in this stage. A prepared workflow
is not evidence that its five native jobs have passed.

Native materials are present in artifacts/retro-boomer-v1-20261031 and approved in noid_node/build.rs with bank pin 841c496f9b1b24db586fb3a976e2c201c335634b84a91193bbe4e4f5d7537f58. All eight source SHA-256 values match the committed policy. The existing packaging and extraction/manifest procedures verified dist/materials-v1.0.1/retro-boomer-history-step-pack-v1.0.1.tar.gz (55270717 bytes; SHA-256 7574cde0dc36f25bea5bc8ee084f047caf20702a5034f169116c1baff7b962d3). The eight archived payloads are byte-identical to the approved v1.0.0 files;
the bank pin and original generation provenance are preserved. No proofs or matrices were regenerated during packaging. Generation verification is owner-reported; mathematical terminal verification was not repeated during packaging. Five-platform builds, native installer qualification and signing remain unverified. Generation commit and independently authenticated generation-log attestation remain unknown.

## 1. Existing approved proof archive

The existing scripts/release_material.py create procedure has packaged the approved raw directory without altering its eight files. Existing extract and verify procedures validated the archive, exact member set, internal SHA256SUMS, native BANK.json, recorded PROVENANCE.json and framing. They do not replace semantic matrix authentication or mathematically reverify terminal proofs.

Do not recreate or overwrite the archive for upload. Its committed archive_sha256 is mandatory in candidate inputs. A fresh extraction directory is required.

## 2. Local native build — compilation, no publication

Use the pinned Rust toolchain, native C/C++ tools, Clang/libclang, CMake and
pkg-config. Linux packaging additionally needs AppStream and dpkg-dev. Windows
needs MSVC/SDK, PowerShell and Inno Setup 6.7.1. macOS needs Xcode command-line
and signing/DMG tools. Use a native host matching one supported target.

### Standard local compilation (no packaging)

```bash
CARGO_BUILD_JOBS=8 ./scripts/build_release.sh \
  --v1-pack artifacts/retro-boomer-v1-20261031 \
  --v1-pins artifacts/retro-boomer-v1-20261031/pins.env \
  --with-gui
```

Without --output, this compiles the four native binaries to target/release/ using
cached dependencies, --offline --locked and the existing release profile.
CARGO_BUILD_JOBS is inherited. No Git identity or SOURCE_DATE_EPOCH is required
for local compilation; no packager, runtime smoke check or publisher runs.
The approved bank/pin and full production matrix checks remain mandatory.

### Native release-candidate staging (explicit --output)

A future candidate build must have a real reviewed source identity and timestamp. In an
actual Git checkout, the following read-only commands derive them. Commit all
reviewed release changes before building an official candidate so its source
identity includes those changes.

```bash
RETRO_BOOMER_SOURCE_COMMIT=$(git rev-parse HEAD) || exit 1
SOURCE_DATE_EPOCH=$(git show -s --format=%ct HEAD) || exit 1
export RETRO_BOOMER_SOURCE_COMMIT SOURCE_DATE_EPOCH
bash scripts/build_release.sh \
  --v1-pack "$PWD/artifacts/retro-boomer-v1-20261031" \
  --v1-pins "$PWD/artifacts/retro-boomer-v1-20261031/pins.env" \
  --output "$PWD/dist/native-candidate" --with-gui --fetch
```

`--fetch` explicitly downloads locked Cargo dependencies before the offline build;
omit it only when the correct target dependencies are already cached. All Cargo
operations retain `--locked`. The candidate mode uses explicit native targets, a fresh
target/staging directory, portable x86-64 flags and macOS deployment target 11.0.
It compiles node, CLI, external miner and desktop GUI on every target. Linux
x86_64 candidate staging additionally compiles and packages `retro-boomer-media`
in Core only.

Both modes perform real matrix preflight. Candidate-only smoke checks start an owned, temporary,
loopback-only node with mining and DNS bootstrap off, check genesis/economics and
stop that child. They never use `~/.retro-boomer`. No PoW mining or new proof generation
is part of these short smoke checks. The bank is checked by build authentication;
current RPC does not independently expose its digest. No fictitious RPC is used.

Products are under `OUTPUT/products/`; a native checksum file is `OUTPUT/SHA256SUMS`.
The script refuses existing output directories instead of deleting them. A local
build without a source commit cannot be promoted to a workflow candidate.

## 3. CI — automatic checks, no publication

`.github/workflows/platform-ci.yml` runs on main pushes, pull requests and manual
dispatch. It uses read-only permissions. It tests release tooling, fetches locked
dependencies and compiles the four binaries on all five native targets, then runs
hardware/help/self-checks without starting a node. It also checks graceful x86 CPU
rejection. Debug builds without a pack have no production proof authority.

The existing-archive integrity test runs locally when the approved archive is
available. It never recreates the archive and is explicitly skipped in ordinary
source-only CI because generated proof archives are not in Git.
Production bank authentication happens in the release-candidate build instead.

## 4. Release candidate — manual, no public release

Before the first run, the owner must separately authorize source publication and
staging of the approved proof archive. One supported transport is an owner-uploaded
asset in an unpublished draft release in **this repository**. Stage 2 performs no
such upload or draft creation. A draft input container is not publication of the
software release. The owner records the immutable numeric asset ID.

Dispatch **RETRO BOOMER Release Candidate** on `main` with:

- `pack_asset_id`: that numeric asset ID;
- `pack_sha256`: the exact approved hash above;
- `ci_run_id`: a successful Platform CI run for the exact selected commit.

Preflight reads the same-repository GitHub asset API; no user URL, mirror or moving
`latest` reference is accepted. It checks the committed archive digest and content,
then shares identical input bytes through an immutable Actions artifact. Each job
checks out the exact preflight SHA. All five native jobs must succeed.

The workflow creates Core/GUI packages, architecture checks and fresh-node smoke
reports. It collects the full build.rs matrix records and source/binary hashes in
five `BUILD-PLATFORM.json` files. Assembly checks Core contents, all hashes, bank,
genesis, source and exact product names before emitting `retro-boomer-release-candidate`.

The candidate artifact contains 19 files: ten products, one proof pack,
`SHA256SUMS`, five build records, `RELEASE_NOTES.md` and `candidate.json`.
The summary displays SHA-256 of `candidate.json`; review and preserve it independently.
This workflow has no `contents: write` permission and cannot publish a release.
Artifacts expire after 14 days; diagnostics after 7 days. Publication must occur
while the reviewed artifact remains available, or a new candidate is required.

## 5. Optional signing — before candidate approval

Default candidates are unsigned on Windows. macOS GUI bundles are ad-hoc
signed; default macOS Core binaries have no Developer ID signature. Do not
present these defaults as Authenticode-signed or Apple-notarized distribution.

Windows optional signing uses a real certificate/private key pre-provisioned in
CurrentUser/My, `RETRO_BOOMER_WINDOWS_SIGN_THUMBPRINT`, and an explicitly owner-approved
HTTPS `RETRO_BOOMER_WINDOWS_TIMESTAMP_URL`. No timestamp URL or certificate is supplied
by this project. `sign_windows.ps1` signs/verifies the four binaries before their
hashes are recorded and the installer before its smoke check. No PFX password is
passed by this script. Certificate provisioning on runners is a separate owner
security task; it is not implemented by these workflows.

macOS optional signing uses `RETRO_BOOMER_MACOS_SIGN_IDENTITY` for the Core binaries
and GUI bundle. Notarization below applies to the GUI DMG, not the Core tarball. Optional notarization uses
`RETRO_BOOMER_MACOS_NOTARY_PROFILE`, a real preconfigured keychain profile, and requires a
real signing identity. It submits the completed DMG, waits, staples and validates
before final candidate checksums. No Apple credential is supplied or invented.

Runner secret/certificate provisioning is intentionally absent. If enabling
signing later, configure it in the candidate jobs, before assembly. Never sign,
repackage or otherwise mutate approved products in the publication job.

## 6. Explicit final draft operations — protected, no rebuild

Configure the retro-boomer-release environment with required reviewers and main-only access. Both operations require a successful manual candidate, its independently reviewed candidate.json SHA-256 and a separately approved annotated v1.0.1 tag pointing to the same candidate commit. Neither operation creates a tag. Only the protected job has contents:write. Ordinary pushes, CI and candidate completion never dispatch either operation.

Dispatch RETRO BOOMER Approved Draft Operations on main:

- create-draft: leave draft_release_id empty and confirm exactly CREATE DRAFT RETRO BOOMER v1.0.1. After approval the workflow refuses an existing release for v1.0.1, creates a draft with --draft --verify-tag, attaches all 19 candidate files, and downloads/checks their actual SHA-256 values. It reports the numeric final draft ID. It never publishes. A partially uploaded draft after a failed job remains unpublished and requires owner review; the workflow never clobbers or repairs it automatically.
- publish-draft: supply that exact draft_release_id and confirm exactly PUBLISH DRAFT RETRO BOOMER v1.0.1. After separate approval it verifies the local candidate again, the draft ID, unpublished state, source/tag, notes, complete asset names and every downloaded asset hash. Only then does it PATCH draft=false on that exact ID. It never creates a release or rebuilds products on this path.

Both operations also require candidate_run_id and candidate_manifest_sha256. The original qualification gate remains mandatory for publishing: candidate notes must contain Release qualification: passed and must not contain preparation only. Current notes intentionally fail this gate. Draft creation can stage a preparation-only candidate for inspection, but that exact candidate cannot subsequently be published while unqualified. Editing remote notes or assets does not qualify it and causes verification failure. A new genuinely qualified candidate requires the existing candidate process and renewed approvals; no binary rebuilding is performed by draft operations.

Use a distinct hosting tag, retro-boomer-materials-v1.0.1, for the input-material draft. It is independent of the final v1.0.1 product draft and is never published by this workflow. Drafts are not public release listings; repository collaborators with appropriate access can see them. No remote operation has been performed during local preparation.

## Product inventory

| Target | Core | GUI |
|---|---|---|
| Linux x86-64 | `retro-boomer-core-v1.0.1-linux-x86_64.tar.gz` | `retro-boomer-gui-v1.0.1-linux-x86_64.deb` |
| Linux ARM64 | `retro-boomer-core-v1.0.1-linux-aarch64.tar.gz` | `retro-boomer-gui-v1.0.1-linux-aarch64.deb` |
| Windows x86-64 | `retro-boomer-core-v1.0.1-windows-x86_64.zip` | `retro-boomer-gui-v1.0.1-windows-x86_64-setup.exe` |
| macOS Intel | `retro-boomer-core-v1.0.1-macos-x86_64.tar.gz` | `retro-boomer-gui-v1.0.1-macos-x86_64.dmg` |
| macOS ARM64 | `retro-boomer-core-v1.0.1-macos-aarch64.tar.gz` | `retro-boomer-gui-v1.0.1-macos-aarch64.dmg` |

Additional attached assets: the HistoryStep pack, SHA256SUMS, candidate.json, RELEASE_NOTES.md and five BUILD records (19 files total).
Hosting-generated source archives are separate. Core includes `retro-boomerd`, `retro-boomer-cli`,
`retro-boomer-miner`, licenses, guide and build identity. Linux x86_64 Core also
includes `retro-boomer-media`; other Core and all GUI packages exclude it. GUI includes `retro-boomer-gui`, `retro-boomerd`,
`retro-boomer-miner`, the supplied artwork, guides and the font license. macOS keeps
`Retro Boomer.app` but its actual GUI executable is `Contents/MacOS/retro-boomer-gui`.

Linux AppStream uses `org.retroboomer.wallet` for the catalog component;
its launchable/icon still refer to the existing `org.retroboomer.wallet` desktop
identity. Window grouping, desktop integration, bundle ID, data paths and Windows
AppId remain unchanged. The older-compatible `developer_name` metadata produces
two informational notices on AppStream 1.1.2, not validation failures.

See the [historical v1.0.0 release checklist](../docs/release-checklist-v1.0.0.md)
for qualification categories; use the v1.0.1 tag and asset names in this guide. This guide does
not authorize any upload, tag, workflow dispatch or publication now.

## Native distribution identity and provisioning

The approved local distribution archive and digest are recorded above. Local ignored dist/ files do not exist on runners. The owner must separately authorize and perform the input-material upload described below. Candidate preflight continues to use pack_asset_id, pack_sha256 and ci_run_id; it verifies the same-repository asset and sends identical checked bytes to all five native jobs. No generator runs on hosted runners.

### Owner-run input-material upload — future authorization required

These are remote WRITE operations. They were not executed during local preparation. Run only after explicit upload authorization and after the source repository has been prepared. Keep this hosting draft separate from v1.0.1. If the hosting tag already has a release, stop and inspect it; do not use --clobber or overwrite assets.

Create and push the separate annotated hosting tag only after owner approval.
The upload below requires that remote tag to exist and never creates it implicitly.

```bash
cd /home/oms/retro-boomer
sha256sum dist/materials-v1.0.1/retro-boomer-history-step-pack-v1.0.1.tar.gz
gh release create retro-boomer-materials-v1.0.1 \
  dist/materials-v1.0.1/retro-boomer-history-step-pack-v1.0.1.tar.gz \
  --repo Aquacongas/retro-boomer --draft --verify-tag \
  --title 'RETRO BOOMER approved HistoryStep input v1.0.1' \
  --notes 'Input-material hosting only. Not the final software release. Keep unpublished.'
gh api --paginate 'repos/Aquacongas/retro-boomer/releases?per_page=100' \
  --jq '.[] | select(.tag_name == "retro-boomer-materials-v1.0.1") | {id, draft, assets: [.assets[] | {id, name, size}]}'
```

After inspecting the listing, enter the archive asset ID (not the release ID) and independently verify the uploaded bytes. These commands download only, but are still deferred until upload authorization:

```bash
read -r -p 'Approved archive asset ID: ' PACK_ASSET_ID
[[ $PACK_ASSET_ID =~ ^[1-9][0-9]*$ ]] || exit 1
VERIFY_DIR=$(mktemp -d)
gh api "repos/Aquacongas/retro-boomer/releases/assets/$PACK_ASSET_ID" \
  -H 'Accept: application/octet-stream' > "$VERIFY_DIR/retro-boomer-history-step-pack-v1.0.1.tar.gz"
printf '%s  %s\n' \
  '7574cde0dc36f25bea5bc8ee084f047caf20702a5034f169116c1baff7b962d3' \
  "$VERIFY_DIR/retro-boomer-history-step-pack-v1.0.1.tar.gz" | sha256sum --check
```

The first digest must match the committed archive_sha256. Record the numeric archive asset ID, not the release ID. Independently download that asset and compare its SHA-256 before dispatch. Supply that asset ID, the committed archive SHA-256 and a successful same-commit Platform CI run ID to Release Candidate. Source-only CI needs no bank upload. Verify that the workflow token can read the hosting draft; this has not been tested remotely. Default Actions input retention is 7 days and candidate retention is 14 days. Preserve approved candidates locally if long review is expected; expired Actions candidates cannot be published by this workflow.

macOS uses the existing Bash workflow steps and script. The obsolete environment check now uses Bash 3.2-compatible indirect parameter expansion rather than [[ -v ... ]]. No new shell installation or alternative build path is required.

## Technical namespaces

RPC uses the generated retro_boomer_ contract throughout clients and authorization checks. P2P uses /retro-boomer/mainnet/cede1246070229c7/1 and /retro-boomer/1.0.0 Identify. The project marker is four bytes RTBM. Genesis construction, economic arithmetic, codecs and original Parano1d networking architecture are unchanged.

The native proof-boundary domain is RETRO_BOOMER/V1/GENESIS-PROOF-BOUNDARY. The approved native output is the 20261031 directory described above; older historical terminals must not be substituted. Builds export only RETRO_BOOMER_V1_PACK_DIR and RETRO_BOOMER_V1_RELEASE_BANK; pins.env accepts only the native key.
