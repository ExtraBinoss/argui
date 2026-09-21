# Argui Live Studio

A small DSL app for checking hot reload and release generation. The editable
source is [`ui/main.argui`](ui/main.argui). It contains a title, a badge,
two cards, an input, a button and a separator. The button and input update
real component state.

From the repository root, build the CLI once:

```sh
cargo build -p argui-cli --bin argui
```

For hot reload, run this from the example directory:

```sh
cd app_examples/dsl-live-demo
../../target/debug/argui dev
```

Edit the `title` value or the `text` of `Button #greet` in `ui/main.argui`.
The CLI reports the file, line, source change, compilation result and client
acknowledgement. The open window should update without clicking. Stop with
Ctrl+C.

To test the generated, non-live build:

```sh
cargo run --manifest-path app_examples/Cargo.toml -p argui-example-dsl-live-demo --release
```

The widget gallery migration is separate: its existing per-widget pages must
adopt the corresponding DSL component one by one, starting with Button.
