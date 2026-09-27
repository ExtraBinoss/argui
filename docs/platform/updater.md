# Application updates

`argui-updater` checks, downloads, verifies, and installs application updates.
It is UI-independent and blocking; run operations on a worker and forward cloned
`State` values to the UI thread.

| Capability | Feature |
| --- | --- |
| Custom backend | `argui-updater` with no feature |
| HTTPS and desktop installation | `argui-updater/native` |

The engine contains no Rust UI. A TSX application can show update state with
its own components; the native host must call the engine on a worker and send
state changes to the UI. Web deployment owns browser application updates.

## Application flow

Build `Config` from your deployed HTTPS feed, current semantic version, and
Minisign public key. The following function validates those inputs and sends
cloned states back to the caller while checking on a worker. Its receiver can
be connected to an Argui model's event channel.

```rust
use std::{sync::mpsc, thread};
use argui_updater::{Result, State, Updater, http::{Config, HttpBackend}, install::NativeInstaller};

/// Starts a release check against `endpoint` using `public_key`.
/// Returns a state receiver and worker; validation errors are returned immediately.
pub fn check_on_worker(
    endpoint: &str,
    public_key: &str,
) -> Result<(mpsc::Receiver<State>, thread::JoinHandle<Result<bool>>)> {
    let config = Config::new(env!("CARGO_PKG_VERSION"), endpoint, public_key)?;
    let (sender, receiver) = mpsc::channel();
    let worker = thread::spawn(move || {
        let backend = HttpBackend::new(config, NativeInstaller::detect()?)?;
        let mut updater = Updater::new(backend);
        updater.check(|state| { let _ = sender.send(state.clone()); })
    });
    Ok((receiver, worker))
}
```

Keep the `Updater` on the worker if the user may later approve a download and
installation; this check-only function drops its transaction when it returns.
Run `check`, `download`, and `install` on a worker and forward each `State` to
the application.

The valid sequence is:

1. `check(callback)`;
2. `download(&CancellationToken, callback)`;
3. `install(callback)`.

A new check invalidates a previous download. Installation consumes a verified
package. Download cancellation is cooperative between network reads; use a new
token for retry. Installation cannot be cancelled after it starts.

The repository's `startup` example accepts a real deployed feed URL and its
matching public-key file as two positional arguments. The signed local-server
tests can be run without a deployed feed:

```sh
cargo nextest run -p argui-updater --all-features
```

## Manifest and signatures

The JSON manifest contains a semantic `version`, optional `notes`, and a
`platforms` map. Each platform entry has the artifact `url`, the full Minisign
`signature`, and an installer `format` from the table below. The map key
defaults to `OS-ARCH`, such as `linux-x86_64` on that target. See
`crates/argui-updater/tests/http.rs` for the repository's signed local-server
fixture and parsed manifest.

`Config::target` can select a package or
ABI variant. SemVer controls ordering. Prereleases require
`allow_prerelease = true`; build metadata alone does not trigger an update.

Sign the exact hosted bytes with Minisign's `-Sm` command and publish the
complete `.minisig` text in the manifest. `Config::new` validates the trusted
public key before any request.

Embed the public key and protect the private key in the release pipeline.
Missing, legacy, or invalid signatures stop installation. Platform signing and
notarization remain separate requirements.

Downloads require HTTPS, including redirects. Loopback HTTP is accepted for
tests. URLs with credentials are rejected. Defaults limit manifests to 1 MiB,
downloads to 1 GiB, connection time to 15 seconds, and total time to 300 seconds.
Temporary and incomplete packages are deleted.

## Native installation

| Format | Behavior |
| --- | --- |
| `executable` | replace a standalone Linux, Windows, or macOS executable |
| `app-image` | replace the running Linux AppImage |
| `app-bundle` | replace a same-named macOS `.app` from `.tar.gz` |
| `nsis` | launch a Windows NSIS executable |
| `msi` | launch `msiexec /passive /norestart` |

`NativeInstaller::detect()` finds the running executable, AppImage, or macOS
bundle. `Destination` selects an explicit path. App bundle replacement keeps a
backup and reports its path if rollback also fails.

The engine does not elevate privileges or exit the application. After
`RestartRequired`, save state and restart. After `InstallerLaunched`, save
state and exit so the installer can finish. Other package systems implement the
`Installer` or `Backend` trait.

Engine tests use a local server and signed fixture. Installer tests replace
temporary files and bundles. Windows installer completion and execution of an
updated macOS bundle still require validation on those systems.
