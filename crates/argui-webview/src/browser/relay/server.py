"""Trusted cross-origin relay. Run with --help for deployment configuration."""
import argparse
import functools
import ipaddress
import json
import re
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlsplit


def origin(value):
    parsed = urlsplit(value)
    if (parsed.scheme not in ("http", "https") or not parsed.hostname
            or parsed.username is not None or parsed.password is not None
            or parsed.path not in ("", "/") or parsed.query or parsed.fragment
            or any(c.isspace() for c in value)):
        raise ValueError("expected an absolute HTTP(S) origin")
    host = parsed.hostname.encode("idna").decode("ascii").lower()
    if ":" in host:
        host = f"[{ipaddress.IPv6Address(host)}]"
    elif not re.fullmatch(r"[a-z0-9]+(?:[.-][a-z0-9]+)*", host):
        raise ValueError("invalid origin hostname")
    port = parsed.port
    suffix = f":{port}" if port and port != (443 if parsed.scheme == "https" else 80) else ""
    return f"{parsed.scheme}://{host}{suffix}"


def configuration(app, relay, allowed):
    app, relay = origin(app), origin(relay)
    if app == relay:
        raise ValueError("relay and application must have distinct origins")
    targets = sorted(set(origin(value) for value in allowed))
    if not targets:
        raise ValueError("at least one allowed website origin is required")
    # Exclude protocol/port variants too: CSP HTTP source matching permits upgrades.
    protected = {urlsplit(app).hostname, urlsplit(relay).hostname}
    if any(urlsplit(target).hostname in protected for target in targets):
        raise ValueError("website hosts must differ from application and relay hosts")
    return {"appOrigin": app, "relayOrigin": relay, "allowedOrigins": targets}


def policy(config):
    return ("default-src 'none'; script-src 'self'; style-src 'self'; "
            "base-uri 'none'; form-action 'none'; frame-src "
            + " ".join(config["allowedOrigins"])
            + "; frame-ancestors " + config["appOrigin"])


class Handler(BaseHTTPRequestHandler):
    def __init__(self, *args, config, **kwargs):
        self.config = config
        super().__init__(*args, **kwargs)

    def do_GET(self):
        routes = {"/": ("index.html", "text/html"),
                  "/client.js": ("client.js", "text/javascript"),
                  "/style.css": ("style.css", "text/css")}
        if self.path == "/config.js":
            content = ("export const config = " + json.dumps(self.config) + ";").encode()
            kind = "text/javascript"
        elif self.path in routes:
            filename, kind = routes[self.path]
            content = Path(__file__).with_name(filename).read_bytes()
        else:
            self.send_error(404)
            return
        self.send_response(200)
        self.send_header("Content-Type", kind + "; charset=utf-8")
        self.send_header("Content-Length", str(len(content)))
        self.send_header("Content-Security-Policy", policy(self.config))
        self.send_header("Cache-Control", "no-store")
        self.send_header("X-Content-Type-Options", "nosniff")
        self.send_header("Referrer-Policy", "no-referrer")
        self.end_headers()
        self.wfile.write(content)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app-origin", required=True)
    parser.add_argument("--public-origin", required=True)
    parser.add_argument("--allowed-origin", action="append", required=True)
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8082)
    args = parser.parse_args()
    try:
        config = configuration(args.app_origin, args.public_origin, args.allowed_origin)
    except ValueError as error:
        parser.error(str(error))
    handler = functools.partial(Handler, config=config)
    with ThreadingHTTPServer((args.host, args.port), handler) as server:
        server.serve_forever()


if __name__ == "__main__":
    main()
