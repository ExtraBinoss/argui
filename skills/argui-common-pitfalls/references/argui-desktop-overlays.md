# Desktop overlay pitfalls

- Keep physical desktop coordinates, native logical window size, and Argui UI
  coordinates separate. A monitor position is physical; the input rectangle and
  painted hole are both UI logical. Convert to physical input pixels with the
  window's native DPI multiplied by UI zoom. Reapply an input shape on resize,
  DPI change, and zoom change.
- Use the same retained rectangle for paint and input. During a pointer drag,
  temporarily restore full-window input so crossing the old hole does not
  interrupt the drag. Restore the excluded rectangle when the selection ends.
- X11 Shape controls which window receives a click; transparent paint alone
  does not. A full-size transparent X11 window needs an active compositor.
  Without one, the surface can appear black even when the alpha plane contains
  a valid transparent hole. Check the compositor selection before opening a
  screen overlay and report an unsupported capability if it is absent.
- Keep an ordinary full-surface application window opaque. The quad shader
  antialiases rectangle edges, including a root rectangle that exactly meets
  the client boundary. On a transparent native surface, the outer pixel rows
  can retain partial alpha and expose the desktop as thin bands. Reserve native
  transparency for surfaces that intentionally expose the desktop, and inspect
  the first and last pixel rows in a private display capture.
- Wayland supports surface-local input regions, but the ordinary desktop
  toplevel contract does not provide portable absolute placement or an
  always-on-top guarantee. Reject a screen-sized spotlight explicitly there.
- In the QuickJS host loop, copy `dispatch.borrow().generation` into a local
  before calling `gallery.deliver_service`. Holding the `RefCell` borrow through
  that call panics if its callback commits a new UI batch.
- For private X11 checks, inspect the overlay's RGBA pixels and X Shape input
  rectangles as well as a saved screen capture. A root capture from a test
  server without a compositor cannot prove visual alpha composition. Inject a
  pointer into the clear and dim regions and verify the X11 target window.

Backend boundary: X11 can place a screen-sized overlay and apply X Shape input
regions. Ordinary Wayland toplevels have surface-local input regions but no
portable absolute placement or always-on-top guarantee. WebAssembly has no
native desktop window. Keep the overlay behind an explicit capability check.
