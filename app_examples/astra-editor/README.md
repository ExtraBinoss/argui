# Astra Editor

Astra Editor is an application-scale Argui example: a responsive Rust-oriented
text editor that starts on an embedded workspace and can open a real folder on
desktop. It demonstrates controlled text editing, multiple files, dirty state,
native saves, Rust syntax highlighting, virtualized project navigation and
search results, resizable panels, display-linked transitions, light and dark
themes, keyboard shortcuts, safe-area layout, and an adaptive phone-sized
experience. Its compact activity rail keeps project actions discoverable
without consuming editor height.

Run the native app from the repository root:

```sh
cargo run --manifest-path app_examples/Cargo.toml -p argui-example-astra-editor
```

Build the WebAssembly version:

```sh
wasm-pack build app_examples/astra-editor --target web --dev \
  --out-dir ../../web/examples/astra-editor/pkg
```

To verify the built editor in a real WebGPU browser, serve `web/` and run the
private-display scenario in another terminal:

```sh
python3 scripts/dev_server.py 8795 --directory web \
  --entry /examples/astra-editor/

CHROME_PATH=/path/to/chrome ./scripts/linux-hidden-display.sh \
  python3 app_examples/astra-editor/tests/browser.py
```

The browser scenario checks the desktop and compact layouts, `Ctrl+B`, the
global-search dialog, canvas sizing, non-blank output, and console errors.

Desktop builds open and index a selected directory away from the UI thread.
The browser build imports selected UTF-8 files into its in-memory workspace;
browser saves update that session's checkpoint because browsers do not expose a
portable folder-write API.

The editor keeps the keystroke path deliberately small: the engine delivers one
UTF-8 range replacement instead of cloning the complete document, retained input
state is painted without rebuilding the whole application, and syntax/search
derivation is coalesced before a source snapshot is taken. Non-wrapping code
shapes only the visible lines plus overscan while retaining the full document
scroll extent. Folder scans also defer syntax work until a document is opened.

## Reproducible performance checks

The edit-delivery profile isolates the cost removed by `TextArea::on_edit`. It
applies the same fixed-width edits to an engine buffer and a 1 MiB controlled
document, then compares incremental `TextEdit` delivery with cloning the complete
value for every callback:

```sh
nice -n 15 ionice -c 3 taskset -c 0 \
  cargo run --manifest-path app_examples/Cargo.toml \
  -p argui-example-astra-editor --release \
  --example edit_delivery_profile -- 1048576 2000
```

The native profile measures the complete application instead: first present,
idle CPU and memory, directory animation frames, `Ctrl+B`, a visible character,
a queued typing burst, and scrolling after growing the document. Build once,
then run the saved binary on the repository's private display. Pass `--check` to
enforce the conservative interaction budgets recorded in `profile.json`.

```sh
cargo build --manifest-path app_examples/Cargo.toml \
  -p argui-example-astra-editor --release

ARGUI_TEST_BACKEND=x11 ./scripts/linux-hidden-display.sh \
  nice -n 10 ionice -c 3 \
  python3 scripts/profile-astra-editor.py \
  --binary /path/to/cargo-target/release/argui-example-astra-editor \
  --output target/astra-editor-profile \
  --sampler-cpu 0 --app-cpu 1 --check
```

Keep the viewport, backend, CPU affinity, event counts, and build profile
identical when comparing revisions. The script saves every measurement as JSON
and validates captures instead of treating a mapped or blank window as success.
Omit the two CPU options on a single-core test environment.

Useful shortcuts:

- `Ctrl/Cmd+Shift+F` searches every indexed line.
- `Ctrl/Cmd+P` opens a file by path.
- `Ctrl/Cmd+S` saves the active document.
- `Ctrl/Cmd+B` toggles the explorer.
- `Ctrl/Cmd+J` opens the system terminal in the project directory on desktop.
- `Escape` dismisses search.
