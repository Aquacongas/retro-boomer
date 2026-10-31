# Retro Boomer v1.0.0 release checklist

An unchecked item is outstanding, not an implied success. Stages 2 and 3 perform local release-tool preparation and lightweight checks;
Stage 3 validates the existing proof archive without recreating it. See the
[pre-push checklist](pre-push-checklist.md) for owner decisions and deferred work.

## Before the first separately authorized source upload

- [x] Disable the legacy publisher and remove active upstream publication targets.
- [x] Preserve licenses, attribution, historical material, icons and wallet behavior.
- [x] Record the real official contact: aquacongass@proton.me.
- [x] Implement source-only CI, manual candidate build and gated publication separately.
- [x] Package the existing bank deterministically, without proof generation.
- [x] Perform a local inventory, ignore-rule review and best-effort content scan.
- [ ] Obtain owner approval of the exact import set and actual staged contents.
- [ ] Review all workflows and action commit pins with the owner.
- [ ] Recover/record any genuine generation source/log evidence, or explicitly review
      and disclose its absence; never replace it with an invented attestation.
- [ ] Authorize repository initialization/import and the first push separately.

## Before native qualification

- [ ] Provide the pinned Rust/native dependencies on each actual runner.
- [ ] Separately authorize staging the approved pack as an immutable asset in an
      unpublished same-repository draft, then record its asset ID and SHA-256.
- [ ] Confirm private security reporting and the support reporting process.
- [ ] Measure fresh-build RAM, disk and time, especially macOS ARM64 (7 GB RAM).
- [ ] Decide whether real Windows signing/macOS Developer ID and notarization are
      required; provision genuine credentials only through an approved process.

## Candidate qualification

- [ ] Platform CI succeeds on all five targets for one exact source commit.
- [ ] Existing terminal proofs receive genuine recorded verification in an authorized
      qualification stage; Stage 2 packaging does not reverify them.
- [ ] Production preflight authenticates Small, Large and Native on every target.
- [ ] All ten packages are created; native architectures and exact payloads pass.
- [ ] Smoke nodes confirm genesis, runtime, emission and zero genesis supply.
- [ ] Clean-system dependencies and minimum OS versions are tested.
- [ ] Actual GUI launch, F5 START/STOP, existing-node handling and key protection
      are tested natively; no private production wallet is used as a test fixture.
- [ ] Cross-platform acceptance, actual mining and synchronization are qualified
      separately, without weakening validation or manufacturing blocks.
- [ ] Installation, update and removal preserve user data.
- [ ] Review signatures/notarization, or explicitly label unsigned/ad-hoc products.
- [ ] Preserve and review the exact candidate.json SHA-256 independently.

## Before publication — requires separate owner approval

- [ ] Finalize notes from real results, with `Release qualification: passed` only
      after actual qualification, and build the final source candidate.
- [ ] Create/approve the annotated v1.0.0 tag at exactly the candidate commit.
- [ ] Configure retro-boomer-release required reviewers and main-only deployment policy.
- [ ] Confirm that no products changed after checksums and approval.
- [ ] Dispatch publication separately with run ID, reviewed manifest hash and the
      exact confirmation phrase; approve the protected environment.
- [ ] Download all 12 published assets and verify SHA256SUMS independently.

No item here authorizes a GitHub write, tag, build, miner or proof generator now.
