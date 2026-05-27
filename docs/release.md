# Crustkit Release Runbook

Crustkit is the shared TUI primitive crate. Publish it before publishing apps
that depend on a new Crustkit version.

## One-Time GitHub Setup

Create and push the GitHub repository from this checkout:

```bash
gh repo create arvidnilber/crustkit --public --source=. --remote=origin --push
```

In GitHub, create an environment named `crates-io` and require manual approval
before deployments. Keep `contents: write` and `id-token: write` limited to the
release workflow.

## crates.io Setup

Preferred steady-state publishing uses crates.io Trusted Publishing through
`.github/workflows/release.yml`.

For the very first publish, crates.io may not have a crate settings page yet.
Use one of these bootstrap options:

1. Run `cargo publish --locked -p crustkit` locally after `cargo login`.
2. Add a one-time crates.io API token scoped to only `crustkit` as
   `CRATES_IO_TOKEN` in the protected `crates-io` environment, run the release,
   then delete the secret and configure Trusted Publishing.

After the first publish, configure crates.io Trusted Publishing for:

- repository: `arvidnilber/crustkit`
- workflow: `.github/workflows/release.yml`
- environment: `crates-io`

## Preflight

```bash
cargo fmt --check
cargo check --locked
cargo check --locked --no-default-features
cargo clippy --locked -- -D warnings
cargo test --locked
cargo test --locked --no-default-features
cargo run -p crustkit-demo -- --help
cargo publish --dry-run --locked -p crustkit
cargo package --list -p crustkit
```

The package must contain source, manifest, README, and license files only. It
must not contain `target/`, local config, package archives, or editor files.

## Release

1. Update `Cargo.toml`, crate README, and `CHANGELOG.md`.
2. Commit the release changes.
3. Create a signed or annotated tag:

```bash
git tag -s v0.1.0 -m "crustkit v0.1.0"
```

If signing is not configured, use an annotated tag instead:

```bash
git tag -a v0.1.0 -m "crustkit v0.1.0"
```

4. Push the branch and tag:

```bash
git push origin main
git push origin v0.1.0
```

The tag starts the protected release workflow, publishes to crates.io, and
creates the GitHub release.

## Rollback

Crates.io versions are permanent. If a bad version is published, fix forward and
yank the affected version:

```bash
cargo yank --version 0.1.0 crustkit
```
