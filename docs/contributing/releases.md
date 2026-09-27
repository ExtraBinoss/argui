# Releases

The [CI workflow](../../.github/workflows/ci.yml) runs independent jobs for
security, Rust quality, public API compatibility, coverage, desktop targets,
mobile cross-checks, Android packages, and crate archives. Keeping static
checks and instrumented coverage separate gives each expensive build its own
timeout without weakening the local combined gate. The release job waits for
the required checks. Rust jobs use `sccache` for compiler outputs and cache
Cargo's registry separately instead of uploading complete `target/`
directories.

The public API job compares the Rust workspace APIs with their latest
crates.io releases.

## Prepare a version

Publishable crates in the root Rust workspace share one workspace version.
Application-only crates under `apps/` use their own manifests and release
policy.

```sh
python3 scripts/release.py bump VERSION
python3 scripts/release.py check
python3 scripts/release.py package
```

`bump` updates the workspace version, internal requirements, and lockfile.
`check` rejects mismatched or invalid manifests. `package` computes the
dependency order from Cargo metadata, creates every archive without resolving
older published workspace versions, and then compiles the extracted archives
against one another through a temporary local Cargo patch. Packaging and verification use
temporary target directories, so this command never cleans or pollutes a target
directory shared with an editor or another worktree.

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
4. creates `vVERSION` and a GitHub release at the checked commit;
5. builds and attaches the Linux, macOS, and Windows `argui` CLI archives and
   SHA-256 files after the release is created.

The website job also uploads the static Nuxt output to GitHub Pages after its
SSR and WebAssembly checks pass. Only the 21 crates currently in the 0.4 Rust
workspace are included in the 0.4 publication graph; old crates removed from
the workspace keep their immutable earlier crates.io releases.

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

The checked-in repository rules require Security, Rust quality, Public API,
Rust coverage, both desktop compile checks, Mobile cross-check, Solid and React
gallery, and Crates.io archives, along with a current branch, resolved review
threads, and code-owner approval. Force pushes and branch deletion are blocked.
The checked-in policy is [.github/main-ruleset.json](../../.github/main-ruleset.json).

The current crate graph is documented in
[repository structure](../repo/structure.md); the script remains the source of
truth for publication order.
