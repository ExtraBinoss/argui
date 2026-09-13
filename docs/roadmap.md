# Remaining roadmap

This page tracks open work. Current APIs belong in the [documentation index](README.md);
measured improvements and their limits belong in [performance](performance/optimizations.md).
The presence of an API or passing semantic tests does not establish visual,
screen-reader, or cross-platform correctness.

## Application testing and platform validation

- Provide a public application test harness for input, focus, window lifecycle,
  a controlled clock and asynchronous delivery. Internal engine tests already
  exercise these contracts, but applications still assemble their own harness.
- Extend native and browser scenarios for independent windows, hide/show,
  close/reopen, cancellation and cleanup during failures. Keep per-platform
  evidence distinct from source-level ownership tests.
- Audit real screen readers and IMEs, including bidi editing, modal focus,
  virtualized controls and protected text, on Linux, Windows, macOS and Web.
- Expand deterministic rendering comparisons for transforms, masks, gradients,
  shadows, refraction and custom effects. Follow the [private-display procedure](contributing/linux-testing.md).

## APIs to extend when required by applications

- Internal and system drag-and-drop with typed payloads, allowed operations,
  cancellation and explicit platform capabilities.
- Native application menus connected to the existing action system; configurable
  shortcut sequences and conflict diagnostics.
- Before/after geometry transitions for insertion, removal and reordering.
  Explicit animated layout properties already exist in the [animation API](ui/animation.md).
- A structured rich-document editor, if needed. Current rich-text display and
  controlled text editing do not supply that document model.
- Broader DevTools property editors and accessibility validation. Expand/collapse,
  primitive numeric editing and a detached native tools window already exist;
  see [DevTools](contributing/devtools.md) for their current scope.

## Performance

Keep comparisons reproducible and tied to a real workload. The retained tree,
layout caches, dirty propagation and explicit layout boundaries are implemented.
Some passes still assemble global output; do not claim that every update costs
only the changed subtree. Prioritize remaining work using the
[CPU and memory evidence](performance/optimizations.md), including gallery components.
Compare equivalent traces on native backends and WebGPU before changing
rendering quality or pass composition defaults.

## Distribution

Prepare root license files, a compatibility policy, a release changelog,
publishable internal dependencies and a CI matrix for supported platforms and
feature combinations. Validate a consumer in a separate repository using only
its required features. The workspace remains experimental.

An optional DSL comes after the Rust contracts stabilize. Its parser and tooling
must lower into the same public elements, styles, effects and animations without
adding a dependency from the runtime or renderer back to the DSL.

Every implementation must preserve idle behavior and pass the
[contribution gate](contributing/code-quality.md).
