import importlib.util
from pathlib import Path
import sys
import unittest

sys.dont_write_bytecode = True
path = Path(__file__).parents[3] / "src/browser/relay/server.py"
spec = importlib.util.spec_from_file_location("relay", path)
relay = importlib.util.module_from_spec(spec)
spec.loader.exec_module(relay)


class ConfigurationTests(unittest.TestCase):
    def test_normalization(self):
        self.assertEqual(relay.origin("https://EXAMPLE.com:443/"), "https://example.com")
        self.assertEqual(relay.origin("http://[::1]:8081"), "http://[::1]:8081")

    def test_reject_invalid_origins(self):
        for value in ["*", "https://*.example.com", "file:///x", "https://user@site.test",
                      "https://site.test/path", "https://site.test/?x=1", "https://site.test/#x",
                      "https://site.test:99999", "https://site.test\n"]:
            with self.subTest(value=value), self.assertRaises(ValueError):
                relay.origin(value)

    def test_protect_app_and_relay_including_scheme_port_variants(self):
        for targets in [[], ["https://app.test"], ["http://app.test:8080"], ["https://relay.test"]]:
            with self.subTest(targets=targets), self.assertRaises(ValueError):
                relay.configuration("https://app.test", "https://relay.test", targets)
        with self.assertRaises(ValueError):
            relay.configuration("https://app.test", "https://app.test", ["https://site.test"])

    def test_policy_is_server_enforced_and_allowlisted(self):
        config = relay.configuration("https://app.test", "https://relay.test", ["https://site.test", "https://site.test/"])
        self.assertEqual(config["allowedOrigins"], ["https://site.test"])
        self.assertIn("frame-src https://site.test; frame-ancestors https://app.test", relay.policy(config))
        self.assertIn("default-src 'none'", relay.policy(config))


if __name__ == "__main__":
    unittest.main()
