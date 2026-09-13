# Application updates

`argui-updater` checks, downloads and installs signed application updates. It is
independent of the UI and does not require an async runtime. Your application
starts the check, forwards state changes and decides when to install or exit.

| Capability | Feature |
| --- | --- |
| Engine and native backend through the facade | `argui/updater` |
| Engine with your own backend | `argui-updater`, with no features |
| HTTPS downloads and desktop installation | `argui-updater/native` |
| Reusable dialog | `argui/widget-updater` or `argui-widgets/updater` |
| Interactive gallery example | `argui-widget-gallery/updater` |

The dialog does not enable networking or installation. It is explicitly opt-in,
including alongside `widgets-all`. The shared engine and dialog compile to
WebAssembly; native installation is only available outside the browser. Web
applications receive updates through their normal deployment.

## Check at startup

The [startup example](../../crates/argui-updater/examples/startup.rs) checks on a
worker and sends states through a channel, without installing anything:

```sh
cargo run -p argui-updater --features native --example startup -- https://updates.example.com/stable/latest.json update.pub
```

```rust,no_run
use argui::updater::{Updater, http::{Config, HttpBackend}, install::NativeInstaller};

let config = Config::new(
    env!("CARGO_PKG_VERSION"), // Your application's version.
    "https://updates.example.com/stable/latest.json",
    include_str!("update.pub"),
)?;
let mut updater = Updater::new(HttpBackend::new(config, NativeInstaller::detect()?)?);
// Run on a startup worker; forward cloned states to your application's UI thread.
let available = updater.check(|state| eprintln!("{state:?}"))?;
# Ok::<(), argui::updater::Error>(())
```

Operations are **blocking**. Run them on a worker, such as
`Context::spawn_blocking` with `argui/tasks`. Keep its `TaskHandle` in the model
and retain the returned engine for the next operation. Callbacks execute on the
worker: send a cloned `State` to the UI thread instead of capturing an `Entity`.
Start the task once, rather than from each render or directly inside an async task.

After checking, `download(&CancellationToken, callback)` downloads and verifies
the package. `install(callback)` starts installation explicitly. A new check
invalidates the previous download. Installation requires a verified package and
consumes it; download again before retrying a failed installation.

`CancellationToken::cancel()` can run on the UI thread. The backend checks it
between network reads, and the engine also rejects a result delivered after
cancellation. A blocked read may wait for the configured timeout. Use a new
token for a retry. Installation cannot be cancelled once started; closing the
dialog does not stop it.

## Publish an application update

Host the manifest on a CDN, object store, GitHub release or any API returning
this JSON. No Argui account is needed. Use separate URLs for channels, such as
`/stable/latest.json` and `/beta/latest.json`.

```json
{
  "version": "1.2.0",
  "notes": "Faster startup and keyboard fixes.",
  "platforms": {
    "linux-x86_64": {
      "url": "https://updates.example.com/1.2.0/MyApp.AppImage",
      "signature": "FULL CONTENTS OF MyApp.AppImage.minisig",
      "format": "app-image"
    },
    "macos-aarch64": {
      "url": "https://updates.example.com/1.2.0/MyApp.app.tar.gz",
      "signature": "FULL CONTENTS OF MyApp.app.tar.gz.minisig",
      "format": "app-bundle"
    },
    "windows-x86_64": {
      "url": "https://updates.example.com/1.2.0/MyApp.msi",
      "signature": "FULL CONTENTS OF MyApp.msi.minisig",
      "format": "msi"
    }
  }
}
```

Platform keys combine `std::env::consts::OS`, `-` and `std::env::consts::ARCH`.
Set `Config::target` to distinguish an ABI or package variant. An API may return
`204 No Content` when no update is available. A missing target artifact is an error.

Comparison follows SemVer precedence: no downgrades, and no update for a
`+build` metadata difference. Prereleases are ignored unless
`Config::allow_prerelease = true`.

Create a Minisign key and sign the **exact bytes** served by your host:

```sh
minisign -G -p update.pub -s update.key
minisign -Sm MyApp.AppImage -s update.key
```

Bundle `update.pub` with the application and keep the private key in your release
pipeline. `signature` contains the complete `.minisig` text, with newlines escaped
by your JSON serializer. Modern prehashed Minisign signatures support incremental
verification. Missing, legacy or invalid signatures prevent installation.

Downloads require HTTPS, including redirects. HTTP is accepted only on loopback
IP addresses for tests; URLs containing credentials are rejected. Manifests are
limited to 1 MiB. Downloads use private temporary files, 64 KiB chunks and a
1 GiB default limit. Configure `max_download_bytes` and `timeout` as needed;
the default total timeout is 300 seconds, with a 15-second connection limit.
Incomplete downloads and abandoned packages are deleted.

Transport and installation dependencies belong to `argui-updater/native`:
Reqwest, Serde/JSON, SemVer, `minisign-verify`, `tempfile`, `tar`, `flate2` and
`self-replace`.

## Installation

| Format | Supported behavior |
| --- | --- |
| `executable` | Standalone Windows, Linux or macOS binary. Atomic replacement on Unix; `self-replace` handles the running executable on Windows. |
| `app-image` | Raw Linux AppImage. `APPIMAGE` identifies the outer file; permissions are preserved. |
| `app-bundle` | `.app.tar.gz` containing one same-named bundle with `Contents/MacOS`. Replaces the whole bundle, including resources. |
| `nsis` | Launches a Windows `.exe` installer using its publisher-configured interface and installation mode. |
| `msi` | Launches `msiexec /i … /passive /norestart`. |

`NativeInstaller::detect()` recognizes the running binary, AppImages and `.app`
ancestors. Use `Destination` for an explicit installation location. Package
format and destination must match. Bundle extraction uses a temporary directory
on the same filesystem. If the final rename fails, the old bundle is restored;
if restoration also fails, the error reports the retained backup path.

The engine does not terminate the process or request privilege elevation. The
application needs permission to modify its installation. After `RestartRequired`,
save state and restart normally. After `InstallerLaunched`, save state and exit
so the installer can finish. Launch success does not mean installation completed.
A launched Windows installer's temporary file remains available after the caller
exits; operating-system temporary-file cleanup owns its eventual removal.

For DEB/RPM, Flatpak, Snap and stores, implement an `Installer` or `Backend` for
that distribution system. Download signatures do not replace platform code
signing or notarization.

## Optional dialog

`UpdateDialog::new(key, &state, open, trigger).build(theme)` creates a controlled
dialog with release notes, status and accessible progress. Download amounts use
decimal MB (`1 MB = 1,000,000 bytes`). An absent or zero total shows an
indeterminate bar and the received amount, without inventing a percentage.

Forward `EventType::Click` and `EventType::Key` to `dialog.action(event)` and
handle the returned intent in your model:

| Action | Application handling |
| --- | --- |
| `Open` / `Close` | Change dialog visibility. |
| `Check` | Run `updater.check` on the worker. |
| `Download` | Create a token and run `updater.download` on the worker. |
| `Cancel` | Cancel the download token. |
| `Install` | Save application state, then run `updater.install`. |

Escape and backdrop clicks close the dialog. Its primary action follows the
current state; checking and installing do not expose a concurrent primary
action. The dialog itself makes no network or operating-system calls.

The gallery demonstrates clearly labelled simulated progress and does not install
anything on your machine:

```sh
cargo run -p argui-widget-gallery --features updater
```

Engine tests use a local HTTP server and a known signed artifact. Installer
tests replace temporary files and bundles. Windows installation and execution
of an updated macOS app still need validation on those operating systems.
For releasing the Argui crates themselves, see [Releases](../contributing/releases.md).
