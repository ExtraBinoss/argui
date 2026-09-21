# ADR 0008: Stable asset keys and revisions

## Decision

An asset registry canonicalizes imported paths into stable `AssetKey` values.
Each accepted content update increments `AssetRevision` and records a content
hash while preserving the key and runtime handle.

Image and vector decoders prepare new resources before registry commit. Failed
updates keep the old revision active. Release reachability includes only assets
referenced by reachable IR, while development retains bounded revisions needed
by in-flight frames.
