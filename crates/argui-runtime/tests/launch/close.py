"""Close the model-free X11 test window through its normal WM_DELETE_WINDOW path."""
import os
import time
from Xlib import X, display, protocol

assert os.environ.get("ARGUI_HIDDEN_DISPLAY") == "1"
assert not os.environ.get("WAYLAND_DISPLAY")
connection = display.Display()
root = connection.screen().root


def descendants(window):
    for child in window.query_tree().children:
        yield child
        yield from descendants(child)


deadline = time.monotonic() + 15
while time.monotonic() < deadline:
    window = next(
        (candidate for candidate in descendants(root)
         if candidate.get_wm_name() == "Argui model-free launch test"),
        None,
    )
    if window is not None:
        delete = connection.intern_atom("WM_DELETE_WINDOW")
        message = protocol.event.ClientMessage(
            window=window,
            client_type=connection.intern_atom("WM_PROTOCOLS"),
            data=(32, [delete, X.CurrentTime, 0, 0, 0]),
        )
        window.send_event(message, event_mask=X.NoEventMask)
        connection.flush()
        break
    time.sleep(0.05)
else:
    raise AssertionError("model-free test window did not appear")
