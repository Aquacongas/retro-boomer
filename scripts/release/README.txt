Retro Boomer Native Distribution
=============================

Publisher: Retro Boomer
Maintainer and support: aquacongass@proton.me
Website: https://retroboomer.org
Project: https://github.com/Aquacongas/retro-boomer

The source defines protocol V1, six decimal places, a 60-second target and ten
issuance stages totaling 10 billion TAPE. All paid transaction fees go to miners.
There is no premine, developer allocation, native fee burn or tail subsidy.
Source parameters are not a substitute for release qualification.

Core archives contain retro-boomerd, retro-boomer-cli and retro-boomer-miner (with .exe on Windows),
LICENSE, NOTICE, this guide and BUILD.json. GUI packages contain retro-boomer-gui,
retro-boomerd, retro-boomer-miner, application icons, guides and the embedded-font license.
retro-boomer-media is not a desktop release component. On macOS the application bundle
is Retro Boomer.app; its executable is Contents/MacOS/retro-boomer-gui.

Start the GUI through its installed application shortcut. It supervises its
bundled local node. F5 Mining controls the external miner; mining is not
started automatically. Do not run another node on the same data directory.
Operator tools document their arguments through retro-boomerd --help, retro-boomer-cli --help
and retro-boomer-miner --help. Check production CPU support with --check-hardware on
the node or miner before use.

Default data directory: ~/.retro-boomer. Addresses: Bech32m tape1. RPC prefix: retro_boomer_.
Default P2P/RPC ports: 19600/19601. The default DNS seeds are
seed1.retroboomer.org and seed2.retroboomer.org, using TCP 19600. Manually
configured peers and persisted peers remain supported. Unavailable seeds use
the existing DNS fallback and retry behavior; do not use the Parano1d network
as a fallback. DNS/server availability still requires deployment verification.
Installers do not remove wallet or chain data. Keep private backups and never
send wallet keys, mining tokens or wallet backups in a support request.

Check downloaded assets against the independently reviewed SHA256SUMS. The
approved native bank is 841c496f9b1b24db586fb3a976e2c201c335634b84a91193bbe4e4f5d7537f58.
The distribution archive and eight source hashes are pinned in scripts/release/retro_boomer_material.json. Five-platform qualification remains pending. See RETRO_BOOMER_BUILD_INSTRUCTIONS.md.
BUILD.json identifies the source, target, binary hashes and matrix build record.
The proof archive's PROVENANCE.json discloses unavailable generation-commit/log
attestation. SHA-256 transport checks do not verify a cryptographic proof.

Unsigned Windows and ad-hoc signed macOS builds must not be described as trusted
signed/notarized releases. Read the actual release qualification and signing
status before using a candidate. A local build or archive is not by itself a
certified public release.

Report non-sensitive bugs through the project issues page. For vulnerabilities,
follow the repository's .github/SECURITY.md or privately contact the maintainer.
Upstream Parano1d has no implied support or bounty obligation for this fork.

Retro Boomer derives from Parano1d. Original copyrights and third-party attribution
are preserved in LICENSE and NOTICE. The bundled Noto font license is OFL-1.1.
Original source: https://git.parano1d.org/ignotusnemo/parano1d

Optional Linux service: retro-boomerd.service is installed as a user unit by
Debian packages. It runs only the node, with mining disabled. It is never
automatically enabled. Choose either GUI supervision or manual service
supervision; do not start two nodes against one data directory. Core archive
users must install the node and adapt the unit ExecStart to their actual binary
location before using a user service. Existing legacy data is never migrated.
Windows data directory: %APPDATA%\RetroBoomer.
