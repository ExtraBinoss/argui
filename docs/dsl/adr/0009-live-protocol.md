# ADR 0009: Versioned live protocol compatibility

## Decision

The host compiler and development runtime communicate through a
transport-independent protocol. Every package declares protocol version, IR
format version, engine compatibility version, root public API hash, and
monotonic generation.

Unknown protocol or IR versions are rejected before decoding payload details.
Engine incompatibility and public ABI changes produce explicit restart-required
responses. Accepted generations are prepared fully and committed atomically;
older/out-of-order generations are ignored.

Desktop sockets/TCP and browser/mobile WebSocket transports carry identical
messages and never embed compiler code in the target application.
