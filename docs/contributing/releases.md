# Releases and GitHub automation

The [CI workflow](../../.github/workflows/ci.yml) checks every pull request and push
to `main`. **Security** scans the full Git history with Gitleaks, validates the
workflow with Actionlint and tests the release policy. **Rust quality** runs the
complete quality gate, including native tests on a private Linux display and the
85% floor for all four coverage metrics in every crate. **Desktop compile**
checks every target and feature on native Windows and macOS runners. **Mobile
cross-check** compiles the supported Android and iOS feature sets plus their
Widget Gallery entry points. **Crates.io archives** runs `cargo package` for
every public crate. **Website** builds the real WASM gallery and generates and
checks every Nuxt page under `/argui/`. Cargo and WebAssembly optimization use
at most six workers.

GitHub Pages deploys successful website builds from `main` to
<https://extrabinoss.github.io/argui/>. PRs build the site without deployment
permissions or registry credentials. Crate publication additionally requires the
quality, Windows, macOS, Android, iOS, archive and website checks to pass.
Third-party actions are pinned to commit SHAs; Gitleaks and Actionlint downloads
are verified against pinned SHA-256 digests.

## Publish a version

All 23 public library crates share the workspace version. Demo applications and
the widget gallery remain `publish = false`. Internal production dependencies
have matching registry versions; repository-only development dependencies stay
path-only so Cargo strips them from published manifests.

```sh
python3 scripts/release.py bump 0.1.1
python3 scripts/release.py check
python3 scripts/release.py package
./scripts/linux-hidden-display.sh env ARGUI_NATIVE_TESTS=1 ./scripts/quality.sh
# Stage the reviewed changes, then check staged files and all existing commits.
git add Cargo.toml Cargo.lock
./scripts/check-secrets.sh
git commit -m '[PUBLISH] Argui 0.1.1'
git push origin main
```

Choose a SemVer version greater than the current workspace version. Prereleases
such as `0.2.0-beta.1` are supported; build metadata is not used for releases.
`bump` updates the workspace, internal dependency requirements and lockfile.
For a multi-commit push, the **last commit** must contain `[PUBLISH]` and the version
must exceed the version at the pre-push commit. A marker without a version change
finishes without publishing. An ordinary commit never publishes, even if it
changes the version.

The release job accepts only a push to this repository's `main` by `ExtraBinoss`.
It checks the registry before uploading, rejects downgrades, yanked versions,
foreign crate ownership and tags belonging to other commits. Registry errors fail
the job; they are never treated as permission to publish.

The release script reads Cargo metadata and computes a deterministic topological
order. `package` creates and compiles all 23 archives together with all features,
using Cargo's temporary registry for internal dependencies that are not public
yet. The protected CI publisher performs another Cargo dry run, then invokes
`cargo publish` once per missing crate in that same order and keeps the `argui`
facade last. The complete crate graph and current order are recorded in
[repository structure](../repo/structure.md).

The token comes from the GitHub Actions secret `CRATE_REGISTRY_TOKEN`, or the
existing `CARGO_REGISTRY_TOKEN`. It is only exposed to the publishing step.
`cargo login` credentials on a developer machine are not copied into GitHub or
the repository. The token must authorize publishing all public `argui-*` crates
as well as `argui`.

Once every crate is visible in crates.io, automation creates tag `vVERSION` at the
exact checked commit and a GitHub release with generated notes, installation
instructions and crate links. Prerelease versions produce GitHub prereleases.
Nothing is published merely by adding these workflows to the repository.

## Recover a partial release

Re-run the failed original workflow in GitHub Actions. The script skips package
versions already published by `ExtraBinoss`, publishes the remaining packages and
creates the missing GitHub release. Existing matching tags/releases are retained.
A tag pointing to a different commit fails instead of being overwritten. Crates.io
versions are immutable, so changed crate contents always need a new version.

## Contributions and main protection

Fork the repository and open a pull request against `main`. The repository ruleset
requires **Security**, **Rust quality**, both **Desktop compile** variants,
**Mobile cross-check**, **Crates.io archives** and **Website**, a current branch,
resolved review threads and one code-owner approval. `.github/CODEOWNERS` assigns
ownership to `ExtraBinoss`. Force pushes and branch deletion are blocked by the
same ruleset. Only the GitHub user `ExtraBinoss` is on its bypass list. This is
enforced by GitHub repository rules, independently of any workflow's
`github.actor` condition.

The configuration is recorded in [.github/main-ruleset.json](../../.github/main-ruleset.json).
Changing collaborator permissions or the bypass list is a repository administration
action; PR workflows have no administration credentials.

The coverage toolchain is pinned in CI through `ARGUI_COVERAGE_TOOLCHAIN`.
Local checks continue using the installed `nightly` unless this variable is set.
