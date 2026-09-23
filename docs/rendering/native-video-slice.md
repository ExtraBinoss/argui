# Native video: first implementation slice

This is the implementation plan for a real Linux `<video>` primitive shared by
the Solid and React native hosts. It is not a description of a shipped feature.
The current gallery decodes SVG and still images before the first frame; it has
no moving-image decoder, playback clock, or dynamic GPU texture update.

## Contract and ownership

1. Add a distinct video asset handle and `Video` schema with `source`, `fit`,
   `playing`, `loop`, and `muted` properties. Reject a raster/SVG handle as a
   video source and vice versa. Generate the Solid and React JSX contracts from
   the same Rust schema. A native `ElementKind::Video` carries the asset handle
   and playback properties in both adapters' `UiTree`.
2. Import an MP4 file through the existing gallery asset generator. Embed its
   bytes, keep the stable JavaScript-safe ID, and validate the handle before
   committing the host transaction. The encoded file is not a still-image
   `ImageAsset`; decoding starts only while a video node is mounted. Key each
   player by retained node identity so two nodes using the same source can have
   independent playback state.
3. Put the GStreamer backend behind a Linux target-specific dependency. Use
   `playbin` with `appsrc://` and an `appsrc` random-access reader over the
   embedded bytes; `need-data` and `seek-data` serve the MP4 without extracting
   a temporary file. Route audio to a normal GStreamer audio sink and video to
   `appsink` with RGBA caps. The pipeline clock and buffer presentation times
   govern playback. The native worker owns GStreamer objects and never calls
   JavaScript or paints on its callback thread.
4. Limit the video sink to at most two decoded buffers, dropping stale frames
   when the UI thread is late. Send the newest frame, dimensions, stride,
   timestamp, and player identity through a bounded native channel. Present on
   the UI thread only when that frame is due; pause both decoder and audio when
   `playing` is false, the node is removed, or the window is suspended.
5. Add a dynamic image texture path that reuses the existing image shader and
   bind groups. Allocate a texture when the decoded size changes; otherwise
   update the existing texture with `queue.write_texture`. Do not call
   `register_image` for every frame: that API recreates the texture and
   invalidates the entire retained surface. Mark the video node's transformed,
   clipped bounds as damaged even though its display-list command is unchanged.
   A frame outside the viewport should not force a redraw.
6. Build one real Media-page player with native play/pause and mute controls,
   a visible current-frame change, and an accessible label. Both gallery
   adapters use the same imported asset and native primitive. No JavaScript
   interval advances the video. Keep non-Linux runs free of a fake player:
   reject an attempted video mount with an explicit unavailable-backend error
   until that target has a decoder and packaging.

## Verification before merging

- Unit tests: media handle validation; wrong-kind rejection; node mount,
  property change, removal, and suspension; frame queue keeps only the newest
  due frame; texture dimensions and stride validation; local damage excludes
  the rest of the viewport.
- Linux integration under `./scripts/linux-hidden-display.sh`: decode an
  embedded short H.264/AAC MP4, capture two distinct video frames, verify
  native play/pause and page removal, and compare Solid/React host trees. An
  unsupported codec must return an error, not a blank video.
- Measure frame time, CPU, RSS and decoded queue size while playing and after
  two minutes idle. Scroll Animation Lab while a video runs elsewhere only if
  this does not disturb the separate scroll performance fix.

## Mobile boundary

The Android Gradle package currently installs only the Rust `libmain.so` and
the iOS Xcode project does not link a media framework. GStreamer publishes an
[Android SDK](https://gstreamer.freedesktop.org/documentation/installing/for-android-development.html)
and an [iOS xcframework](https://gstreamer.freedesktop.org/documentation/installing/for-ios-development.html),
but those must be packaged, linked, and tested on devices before either target
can claim video support. The Linux RGBA upload path is a correctness slice;
mobile throughput, copies, codec availability, app size, and audio lifecycle
need measurements before selecting a hardware-texture path. GStreamer's
[appsink queue limits](https://gstreamer.freedesktop.org/documentation/app/appsink.html)
and [seekable appsrc](https://gstreamer.freedesktop.org/documentation/app/appsrc.html)
provide the intended bounded, embedded-file data flow.
