#!/usr/bin/env bash
# Run graphical checks in a private compositor, never on the user's desktop.
set -euo pipefail

if [[ $# -eq 0 ]]; then
    echo "Usage: $0 COMMAND [ARGUMENT...]" >&2
    exit 2
fi
command -v mutter >/dev/null
command -v dbus-run-session >/dev/null

test_runtime="$(mktemp -d "${TMPDIR:-/tmp}/argui-display.XXXXXX")"
cleanup() {
    local result=$?
    trap - EXIT
    # A portal may unmount itself between mountpoint and fusermount during teardown.
    for mount in "$test_runtime/doc" "$test_runtime/gvfs"; do
        if mountpoint -q "$mount"; then
            if ! fusermount3 -uz "$mount" && mountpoint -q "$mount"; then
                result=1
            fi
        fi
    done
    if ! rm -rf -- "$test_runtime"; then result=1; fi
    exit "$result"
}
trap cleanup EXIT
chmod 700 "$test_runtime"

backend="${ARGUI_TEST_BACKEND:-wayland}"
case "$backend" in
    wayland) test_command=("$@") ;;
    x11) test_command=("$(dirname "$0")/linux-hidden-x11.sh" "$@") ;;
    *) echo "Unknown ARGUI_TEST_BACKEND: $backend" >&2; exit 2 ;;
esac

env -u DISPLAY -u WAYLAND_DISPLAY -u WAYLAND_SOCKET -u DBUS_SESSION_BUS_ADDRESS \
    XDG_RUNTIME_DIR="$test_runtime" GDK_BACKEND=wayland QT_QPA_PLATFORM=wayland GIO_USE_VFS=local \
    dbus-run-session -- mutter --headless --wayland --no-x11 \
    --virtual-monitor="${ARGUI_TEST_MONITOR:-1600x1200@60}" \
    --wayland-display=argui-test -- \
    env WAYLAND_DISPLAY=argui-test ARGUI_HIDDEN_DISPLAY=1 "${test_command[@]}"
