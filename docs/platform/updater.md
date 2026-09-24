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

```rust,no_run
use argui_updater::{
    Updater,
    http::{Config, HttpBackend},
    install::NativeInstaller,
};

let config = Config::new(
    env!("CARGO_PKG_VERSION"),
    "https://updates.example.com/stable/latest.json",
    include_str!("update.pub"),
)?;
let backend = HttpBackend::new(config, NativeInstaller::detect()?)?;
let mut updater = Updater::new(backend);
let available = updater.check(|state| eprintln!("{state:?}"))?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Run `check`, `download`, and `install` on a worker. Keep the updater for the
next operation and forward each `State` to the application.

The valid sequence is:

1. `check(callback)`;
2. `download(&CancellationToken, callback)`;
3. `install(callback)`.

A new check invalidates a previous download. Installation consumes a verified
package. Download cancellation is cooperative between network reads; use a new
token for retry. Installation cannot be cancelled after it starts.

The runnable engine example checks without installing:

```sh
cargo run -p argui-updater --features native --example startup -- \
  https://updates.example.com/stable/latest.json update.pub
```

## Manifest and signatures

```json
{
  "version": "1.2.0",
  "notes": "Faster startup and keyboard fixes.",
  "platforms": {
    "linux-x86_64": {
      "url": "https://updates.example.com/1.2.0/MyApp.AppImage",
      "signature": "FULL .minisig CONTENT",
      "format": "app-image"
    }
  }
}
```

Platform keys default to `OS-ARCH`; `Config::target` can select a package or
ABI variant. SemVer controls ordering. Prereleases require
`allow_prerelease = true`; build metadata alone does not trigger an update.

Sign the exact hosted bytes with Minisign:

```sh
minisign -G -p update.pub -s update.key
minisign -Sm MyApp.AppImage -s update.key
```

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
