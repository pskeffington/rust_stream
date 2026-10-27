"""Offline contract tests for New England HLS manifest diagnostics."""
import importlib.util
import pathlib
import unittest
from unittest.mock import patch
import io

PATH = pathlib.Path(__file__).resolve().parents[1] / "scripts" / "probe_new_england_hls.py"
spec = importlib.util.spec_from_file_location("ne_probe", PATH)
probe_module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe_module)


class FakeResponse:
    status = 200
    def __init__(self, body):
        self.body = io.BytesIO(body)
    def __enter__(self):
        return self
    def __exit__(self, *args):
        return False
    def read(self, n):
        return self.body.read(n)


class Tests(unittest.TestCase):
    def setUp(self):
        self.item = {"id": "ma-cbsboston", "endpoint": "https://example.org/master.m3u8"}

    def test_master_playlist(self):
        with patch("urllib.request.OpenerDirector.open", return_value=FakeResponse(b"#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1000\nvariant.m3u8\n")):
            result = probe_module.probe(self.item, 2)
        self.assertEqual(result["manifest_status"], "manifest_reachable")
        self.assertEqual(result["manifest_kind"], "master")
        self.assertFalse(result["auto_connect"])

    def test_html_does_not_pass(self):
        with patch("urllib.request.OpenerDirector.open", return_value=FakeResponse(b"<html>not a stream</html>")):
            result = probe_module.probe(self.item, 2)
        self.assertEqual(result["manifest_status"], "not_hls")

    def test_oversize_does_not_pass(self):
        with patch("urllib.request.OpenerDirector.open", return_value=FakeResponse(b"#EXTM3U\n" + b"x" * (probe_module.MAX_BYTES + 1))):
            result = probe_module.probe(self.item, 2)
        self.assertEqual(result["manifest_status"], "size_limit")

    def test_non_https_rejected_before_request(self):
        self.item["endpoint"] = "http://example.org/master.m3u8"
        with patch("urllib.request.OpenerDirector.open") as opened:
            result = probe_module.probe(self.item, 2)
        opened.assert_not_called()
        self.assertEqual(result["manifest_status"], "invalid_url")


if __name__ == "__main__":
    unittest.main()
