# Contributing

Contributions to ProtoV MINI firmware, hardware, documentation, and tooling are
welcome. Keep each pull request focused and explain what changed, why it is
needed, and how it was validated.

For significant or compatibility-sensitive work, open a
[GitHub issue](https://github.com/flakeblade/protov/issues) or discussion before
implementation.

## Patch changes (`1.0.x`)

Bug fixes, documentation improvements, and quality-of-life changes should:

- Preserve existing behavior and interfaces.
- Include reproduction and validation steps.
- Update affected documentation.
- Include measurements or test results when hardware is involved.

Small fixes may be merged without receiving a standalone release.

## Feature changes (`1.x.0`)

Backward-compatible features are welcome with the following requirements:

- Changes to `protov-core` must include test coverage for new behavior.
- Changes involving both this repository and
  [`protov_app`](https://github.com/flakeblade/protov_app) must define compatible
  versions for both projects before merge.
- Changes to `protov-hal` should cite the relevant component datasheet and
  include hardware validation where practical.
- Changes to current limiting, voltage or power limits, thermal protection,
  calibration, fault handling, or other safety-related hardware behavior are
  feature changes and must include electrical justification, relevant
  datasheet references, tests, and measured validation.
- New SCPI commands are allowed, but the
  [protocol reference](protov-scpi/PROTOCOL.md) must be updated. Existing
  commands may not be removed or reinterpreted incompatibly. Adding a
  backward-compatible SCPI command is a minor-version change.

> [!IMPORTANT]
> Safety and protection changes must cover normal operation, boundary
> conditions, and relevant failure modes on every affected hardware revision.
> Changes that bypass or weaken protection will receive additional review and
> may be refused.

## Breaking changes (`2.x.x`)

Changes to `protov-bootloader`, `protov-nvm`, the linker scripts, partition
layout, or core DFU behavior can break software-update compatibility. They are
reviewed case by case, require a strong justification, and will generally
require a major `2.x.x` release.

Changes that do not affect storage, boot, DFU, or protocol compatibility—such
as an isolated UI adjustment—can follow the normal patch or feature process.
Changes to core boot or update behavior are unlikely to be merged without prior
agreement and a complete migration plan.

> [!IMPORTANT]
> The `protov-nvm` `RESERVED` partition belongs to users and must not be
> allocated to new product features. Proposals requiring another NVM region
> must be discussed in an issue or discussion before implementation. Changing
> the flash layout is a major-version change.

## Signing and CI

> [!CAUTION]
> Pull requests that add or replace trusted public keys, weaken the CI signing
> chain, or modify CI workflows—especially signing and release automation—
> receive additional security review and may be refused outright.

Never commit private keys, credentials, or secrets. Report vulnerabilities
privately according to the [security policy](SECURITY.md).

## Pull-request checklist

- Keep the PR limited to one coherent change.
- Explain compatibility and versioning impact.
- Add tests, measurements, screenshots, or logs as appropriate.
- Update documentation and protocol references.
- Call out limitations, migrations, and follow-up work.
