# Optional media and native TSX viewport

The starting point is remote `codex/dsl-gallery-live` at `518a808`. This plan
replaces the separate static-media crates and exposes an application-owned GPU
viewport to Solid and React. A media decoder, GStreamer pipeline, video file,
and platform video package are outside this change.

## Static media crate

1. Consolidate `argui-assets`, `argui-image`, and `argui-vector` into
   `argui-media`. Keep source identity and revision handling in the crate root;
   put raster decoding in `image`, SVG parsing in `svg`, and authored vector
   paths in `path`. Keep renderer-neutral `ImageAsset` and `VectorAsset` in
   `argui-paint`, and WGPU rasterization in `argui-render`.
2. Expose one application-facing `media` feature, disabled by default. Enabling
   it includes both raster images and SVG. The gallery enables `media`; apps
   that omit it do not compile the file decoders. There are no separate
   application-facing image and SVG switches: choose `media` or no media.
   Authored paths stay available without a file decoder.
3. Replace all imports, manifests, release metadata, generated lockfiles, tests,
   and documentation in the same change. Delete the three replaced crate
   directories after all consumers use `argui-media`. No compatibility shim.
4. Keep the existing typed source handles and failure behavior. Test feature
   combinations, invalid input, stable revisions, wrong-kind updates, and
   decoding of gallery imports. Avoid embedding unsupported formats by default.

## Application-owned native viewport

1. Add a native schema element with a stable canvas registration identifier,
   normal layout properties, an explicit resolution scale, and an accessible
   label. Generate matching Solid and React JSX types from the Rust schema.
   Both adapters send only control values, never pixels or WGPU handles.
2. Give the application a native registration and render callback using Argui's
   existing GPU-canvas device, queue, encoder, and bounded offscreen target.
   Validate unknown IDs and scale limits before committing a TSX tree.
3. The producer owns a bounded latest-frame mailbox. A native frame arrival
   marks only the mounted viewport dirty and wakes the native renderer; a
   paused or unmounted viewport does no work. Coalesce frames and drop stale
   data. Keep player clocks, audio, codecs, and export entirely in the
   application engine.
4. On resize, expose the new physical extent and DPI to the callback. The
   producer can choose a new output resolution; Argui reallocates only when the
   viewport extent changes. Stable input keeps the retained texture. Preserve
   clipping, transforms, rounded corners, opacity, z-order, and accessibility.
5. Unit-test schema translation, ID validation, resize and revision policy,
   mailbox bounds, unmount cleanup, and Solid/React host transactions with a
   synthetic producer. Compile on desktop only. Do not require a real video,
   Android device, mobile SDK, or GStreamer during this change.

## Editor integration after this change

The editor owns its GStreamer/GES timeline. TSX sends play, pause, seek, and
edit commands to the native editor service. Preview frames flow from a bounded
native sink to the viewport renderer. Export uses a separate GES render pipeline
from a stable project snapshot, independent of the window. Start with CPU
decoded frames uploaded into a reusable WGPU texture; profile before adding
platform-specific GPU texture sharing.
