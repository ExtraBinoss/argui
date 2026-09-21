# DSL development, reload and tooling

## One semantic pipeline

The parser produces a lossless, error-recovering CST. The incremental semantic
database resolves modules, types and native schemas. A typed, ID-only IR then
feeds both Rust AOT generation and live bytecode preparation. Parity tests use
the same fixture in both backends.

Release applications link generated Rust plus the normal Argui engine. The
parser, semantic compiler, IR interpreter, watcher, protocol server and LSP are
development dependencies and are absent from the normal dependency graph.

## Transactional generations

`argui dev` watches `.argui`, `.wgsl` and reachable assets. Each accepted edit
produces a complete package containing explicit protocol, IR and engine
compatibility versions, a public ABI hash, entry roots, pruned IR and revisioned
asset bytes.

The client performs four steps before changing visible state:

1. validate protocol and engine compatibility;
2. prepare expression bytecode, assets and WGSL pipelines;
3. migrate compatible component/theme/animation state;
4. commit the complete generation atomically.

An untouched property receives the new DSL default on reload. A property
explicitly supplied by the host or changed at runtime keeps its value, even
when that value happened to equal the old default.

Diagnostics, invalid shaders, missing assets or preparation failures leave the
last valid generation untouched. Public property/callback/slot changes produce
a restart request rather than unsafe state reuse.

## Transports

Native desktop and development-device clients use framed TCP. Browser clients
use binary WebSocket frames and the browser `WebSocket` API. Both exchange the
same serialized messages and enforce the same compatibility handshake.

Defaults:

```text
TCP        127.0.0.1:4777
WebSocket  ws://127.0.0.1:4778
```

Set `ARGUI_DEV_ADDRESS` for a native/mobile client. Browser applications call
`LiveRuntime::connect_web(url)` and remain idle until the socket delivers a
generation. These endpoints are development-only and have no authentication;
bind to a trusted interface.

## Editor server

`argui-dsl-lsp` speaks standard JSON-RPC over stdio using `Content-Length`
framing. It delegates to the same incremental semantic database as builds and
supports:

- diagnostics and schema-aware completion;
- hover, definition, references and rename;
- document/workspace symbols;
- semantic tokens;
- full-document formatting;
- document colors and color presentation;
- source actions for supported diagnostics.

Configure an editor to start `argui-dsl-lsp` for `*.argui` files with the
project directory as its workspace root.

## CLI automation

The CLI’s JSON commands are stable interfaces for agents and editor tooling:

```sh
argui check [ENTRY] --json
argui schema [COMPONENT] --json
argui complete PATH LINE:COLUMN --json
argui symbols [PATH] --json
argui fmt [PATH ...] --check
```

`argui dev --no-run` starts only the compiler/watcher/transports, which is
useful when a browser or a development device owns the presentation process.
In an interactive terminal, `argui dev` shows a read-only Ratatui dashboard:
the changed file and location, before/after source lines, stage-based update
gauge, diagnostics, connections, and actual client acknowledgements. It does
not capture mouse input or redraw while idle. Redirected output remains
line-oriented for logs and automation.

## Compatibility and performance rules

- Stable identities derive from declaration names and structural source sites,
  never line numbers.
- Runtime property, event, token, asset and effect access uses resolved IDs.
- Expression programs compile once per accepted package and memoize clean
  property/token dependencies.
- Idle clients block on transport/task wakeups; they do not request perpetual
  frames or poll the compiler.
- Reachability removes unused components, user types, themes, styles, effects
  and assets before release generation and live transport.
- UI, WGSL and assets commit as a single generation.

The detailed design history and phase exit criteria remain in the
[implementation plan](ARGUI_DSL_IMPLEMENTATION_PLAN.md) and [ADRs](adr/).
