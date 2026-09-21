# ADR 0010: Typed public UI ABI and restart boundary

## Decision

Exported root components generate a deterministic ABI from public properties,
callbacks, models, structs, enums, slots, directions, and types. Rust bindings
are statically typed and contain no `DslValue` or string-based lookup.

Private component changes may reload when state migration is compatible.
Changing the public ABI connected to Rust never performs a best-effort remap;
the runtime preserves the current generation and reports restart required.
Callback identity derives from semantic `CallbackId`, not local render slot
order.
