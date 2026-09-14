# Fake AI Harness

A deterministic, offline agent streams a bundled adaptation of Wikipedia's
Large language model article as a 6,000-token response at a target rate of
1,000 tokens per second. The app appends every token due in a frame as one
batch. The variable-height VList keeps a stable window of visible messages
mounted while input, scrolling and rendering continue.

The model and its throughput are simulated; no prompt leaves the process and no
API key is needed. Compact telemetry reports the measured rate, time to first
token, UI batch count and generated context on layouts wide enough to show it.

```sh
cargo run --manifest-path app_examples/Cargo.toml -p argui-example-ai-harness
```

The dependency enables only the Argui task runtime, the four visible controls
and selectable text used by the application. It does not pull in `widgets-all`,
i18n, mobile adapters or desktop integrations.
