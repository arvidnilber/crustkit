# Security Policy

## Supported Versions

Security fixes are released for the latest published `0.x` version.

## Reporting

Do not open a public issue for a suspected vulnerability. Email the maintainer
or use GitHub private vulnerability reporting once the GitHub repository is
created.

Include:

- affected crate and version
- reproduction steps
- expected impact
- whether a proof of concept can be shared privately

## Release Security

Crates.io publishing should use Trusted Publishing from the `Release` workflow
after the first crate version exists. For the initial bootstrap publish, use
either a local `cargo publish` from a trusted machine or a one-time crates.io
token scoped to this crate, stored only as the `CRATES_IO_TOKEN` GitHub Actions
secret in the protected `crates-io` environment.

Never commit crates.io tokens, Plex tokens, API keys, `.env` files, local Cargo
patch files, package archives, cache databases, or logs.
