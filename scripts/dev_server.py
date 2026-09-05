"""Serve local build output without retaining obsolete JS or WASM in caches."""

import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer


class Handler(SimpleHTTPRequestHandler):
    def __init__(self, *args, entry, **kwargs):
        self.entry = entry
        super().__init__(*args, **kwargs)

    def end_headers(self):
        self.send_header("Cache-Control", "no-store")
        super().end_headers()

    def send_head(self):
        if self.path.split("?", 1)[0] == "/" and self.entry != "/":
            self.send_response(302)
            self.send_header("Location", self.entry)
            self.end_headers()
            return None
        return super().send_head()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("port", type=int)
    parser.add_argument("--directory", required=True)
    parser.add_argument("--entry", default="/")
    args = parser.parse_args()
    handler = partial(Handler, directory=args.directory, entry=args.entry)
    with ThreadingHTTPServer(("127.0.0.1", args.port), handler) as server:
        try:
            server.serve_forever()
        except KeyboardInterrupt:
            pass
