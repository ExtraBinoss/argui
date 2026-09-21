# Getting started with Argui DSL

Argui applications author their interface in `.argui` and keep business logic
in Rust. The release build compiles the DSL ahead of time to typed Rust; the
development runtime consumes the same typed IR for transactional reload.

## Create an application

From an Argui checkout:

```sh
cargo run -p argui-cli -- new hello-argui
cd hello-argui
cargo run
```

The scaffold contains:

```text
hello-argui/
├── Cargo.toml
├── build.rs
├── src/main.rs
└── ui/main.argui
```

`build.rs` calls `argui_dsl_build::compile("ui/main.argui")` and
`argui::include_ui!()` includes the generated API. `argui-dsl-build` is a build
dependency only. The live runtime is optional and disabled in normal builds.

## Author the root component

```text
import { Button, Column, Input, Text } from "@argui/ui"

export component Main {
    private property name: string = ""
    private property saved: bool = false
    callback submit(name: string)

    Column #form {
        gap: 12.0
        Input {
            value <=> name
            label: "Name"
            placeholder: "Ada"
            on submit { submit(name); saved = true }
        }
        Button {
            text: "Save"
            enabled: name != ""
            on click { submit(name); saved = true }
        }
        if saved {
            Text { text: "Saved" }
        }
    }
}
```

Imports are explicit. `@argui/ui` is the public component library;
`@argui/native` is reserved for low-level behavior and layout adapters.
Properties and callback payloads are checked before either backend runs.

## Bind Rust logic

Each exported component becomes a Rust type. Required input properties are
constructor parameters; other inputs have typed setters; callbacks have typed
`on_*` methods; and slots have typed element setters.

```rust
let root = Main::new();
root.on_submit(|name| {
    println!("saving {name}");
});
```

Rust API changes are an intentional rebuild boundary. Layout, styles, states,
WGSL and assets remain reloadable when the public component ABI is unchanged.

## Check and format

```sh
argui check ui/main.argui --json
argui fmt
argui fmt --check
argui schema Button --json
```

The JSON commands are stable automation surfaces. The formatter preserves
comments and refuses to rewrite recovering/invalid input.

## Start live development

```sh
argui dev
```

The generated `argui-live` feature connects the application to the external
compiler service. The process is started once; normal `.argui`, `.wgsl` and
asset edits do not invoke Cargo again. If compilation or package preparation
fails, diagnostics are published and the last valid tree stays active.

Use `argui dev ui/main.argui 0.0.0.0:4777` to expose the native TCP endpoint to
a development device. The browser WebSocket endpoint uses the next port
(`4778` in this example). Do not expose either development endpoint to an
untrusted network.

Continue with the [language guide](language.md) and
[development/tooling guide](development.md).
