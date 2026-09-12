#!/usr/bin/env bash
# Internal helper: the rootful X server lives entirely in our private Wayland compositor.
set -euo pipefail
[[ "${ARGUI_HIDDEN_DISPLAY:-}" == 1 ]]
[[ "${XDG_RUNTIME_DIR##*/}" == argui-display.* ]]
[[ -S "$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY" ]]
command -v Xwayland >/dev/null
x11_number="$XDG_RUNTIME_DIR/x11-display"
x11_log="$XDG_RUNTIME_DIR/x11.log"
x11_geometry="${ARGUI_TEST_MONITOR:-1600x1200}"
exec 3>"$x11_number"
Xwayland -displayfd 3 -nolisten tcp -noreset \
    -geometry "${x11_geometry%%@*}" >"$x11_log" 2>&1 &
x11_pid=$!
exec 3>&-
cleanup_x11() {
    kill "$x11_pid" 2>/dev/null || true
    wait "$x11_pid" 2>/dev/null || true
}
trap cleanup_x11 EXIT
for ((attempt = 0; attempt < 100; attempt++)); do
    if [[ -s "$x11_number" ]]; then break; fi
    if ! kill -0 "$x11_pid" 2>/dev/null; then cat "$x11_log" >&2; exit 1; fi
    sleep 0.05
done
[[ -s "$x11_number" ]] || { cat "$x11_log" >&2; exit 1; }
env -u WAYLAND_DISPLAY -u WAYLAND_SOCKET \
    DISPLAY=":$(cat "$x11_number")" ARGUI_HIDDEN_X11_PID="$x11_pid" \
    GDK_BACKEND=x11 QT_QPA_PLATFORM=xcb "$@"
