# Releases

The [CI workflow](../../.github/workflows/ci.yml) runs independent jobs for
security, static Rust quality, public API/SemVer compatibility, Rust coverage,
desktop targets, mobile targets, package archives, and the website. Keeping static checks and instrumented
coverage separate gives each expensive build its own timeout without weakening
the local combined gate. GitHub Pages depends only on the website job. Crate
publication waits for every required job.

## Prepare a version

All 24 public crates share one workspace version. Application and showcase
crates set `publish = false`.

```sh
python3 scripts/release.py bump VERSION
python3 scripts/release.py check
python3 scripts/release.py package
```

`bump` updates the workspace version, internal requirements, and lockfile.
`check` rejects mismatched or invalid manifests. `package` computes the
dependency order from Cargo metadata, creates every archive without resolving
older published workspace versions, and then compiles each extracted archive
against the previously staged 0.3 sources.

Review the generated diff and run the final
[quality gate](code-quality.md#final-gate). The final commit on the push must
contain `[PUBLISH]`:

```sh
git add --all
./scripts/check-secrets.sh
./scripts/linux-hidden-display.sh env ARGUI_NATIVE_TESTS=1 ./scripts/quality.sh
git commit -m '[PUBLISH] Argui VERSION'
git push origin main
```

An ordinary commit never publishes, even when it changes the version.

## CI publication

Publication runs only for a push to this repository's `main` branch by
`ExtraBinoss`. It:

1. repeats manifest and package validation;
2. checks ownership and existing versions on crates.io;
3. publishes missing crates in dependency order;
4. publishes the `argui` facade last;
5. creates `vVERSION` and a GitHub release at the checked commit.

The token comes from `CRATE_REGISTRY_TOKEN`, falling back to
`CARGO_REGISTRY_TOKEN`, and is exposed only to the publish step. Prerelease
versions create GitHub prereleases.

The script is idempotent. If crates.io rate-limits a partial release, rerun the
same failed workflow. Versions already published by the project are skipped and
the remaining crates continue in dependency order. Crates.io versions are
immutable; changed contents require a new version.

A tag pointing at another commit, a downgrade, a yanked version, foreign crate
ownership, or a registry error stops the release.

## Protected main

The repository rules require the security, quality, public API, desktop,
mobile, package, and website checks, a current branch, resolved review threads,
and code-owner approval. Force pushes and branch deletion are blocked. The checked-in policy is
[.github/main-ruleset.json](../../.github/main-ruleset.json).

The current crate graph is documented in
[repository structure](../repo/structure.md); the script remains the source of
truth for publication order.
