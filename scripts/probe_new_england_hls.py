#!/usr/bin/env python3
"""Bounded, opt-in manifest probe for New England direct HLS candidates.

Discovery-only diagnostics. Does not decode streams, follow redirects, fetch
segments, assert authorization or automatically approve a feed.
"""
import argparse
import datetime as dt
import json
import pathlib
import ssl
import sys
import urllib.error
import urllib.request
from urllib.parse import urlsplit

ROOT = pathlib.Path(__file__).resolve().parents[1]
CATALOG = ROOT / "data" / "new_england_sources.json"
MAX_BYTES = 256 * 1024


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def probe(item, timeout):
    endpoint = item["endpoint"]
    result = {"id": item["id"], "manifest_status": "not_tested",
              "http_status": None, "manifest_kind": None, "error": None,
              "frame_status": "not_tested", "publisher_permission": "not_reviewed",
              "auto_connect": False}
    parsed = urlsplit(endpoint)
    if parsed.scheme != "https" or not parsed.hostname or not parsed.path.endswith(".m3u8") or parsed.username or parsed.password or parsed.fragment:
        result["manifest_status"] = "invalid_url"
        return result
    opener = urllib.request.build_opener(
        NoRedirect(),
        urllib.request.HTTPSHandler(context=ssl.create_default_context()))
    request = urllib.request.Request(endpoint, headers={
        "User-Agent": "rust-stream-validation/0.1",
        "Accept": "application/vnd.apple.mpegurl,application/x-mpegURL,*/*;q=0.1"})
    try:
        with opener.open(request, timeout=timeout) as response:
            result["http_status"] = response.status
            if response.status != 200:
                result["manifest_status"] = "http_error"
                return result
            payload = response.read(MAX_BYTES + 1)
            if len(payload) > MAX_BYTES:
                result["manifest_status"] = "size_limit"
                return result
            try:
                text = payload.decode("utf-8-sig")
            except UnicodeDecodeError:
                result["manifest_status"] = "non_utf8"
                return result
            lines = [line.strip() for line in text.splitlines() if line.strip()]
            if not lines or lines[0] != "#EXTM3U":
                result["manifest_status"] = "not_hls"
                return result
            result["manifest_status"] = "manifest_reachable"
            result["manifest_kind"] = ("master" if any(x.startswith("#EXT-X-STREAM-INF") for x in lines)
                else "media" if any(x.startswith("#EXTINF") for x in lines) else "unknown")
    except urllib.error.HTTPError as exc:
        result["http_status"] = exc.code
        result["manifest_status"] = "redirect_denied" if 300 <= exc.code < 400 else "http_error"
    except (urllib.error.URLError, TimeoutError, OSError, ValueError) as exc:
        result["manifest_status"] = "network_error"
        result["error"] = type(exc).__name__
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ids", required=True, help="Comma-separated curated source IDs to probe explicitly")
    parser.add_argument("--output", help="Write timestamped JSON report (no URL secrets)")
    parser.add_argument("--timeout", type=float, default=8.0)
    args = parser.parse_args()
    if not 0 < args.timeout <= 20:
        parser.error("--timeout must be within (0, 20] seconds")
    data = json.loads(CATALOG.read_text(encoding="utf-8"))
    eligible = {item["id"]: item for item in data["sources"]
                if item["kind"] == "direct_hls_candidate" and item.get("endpoint")}
    ids = [i.strip() for i in args.ids.split(",") if i.strip()]
    if not ids or len(ids) != len(set(ids)) or any(i not in eligible for i in ids):
        parser.error("IDs must be unique, nonempty, and explicitly present as direct_hls_candidate")
    report = {"checked_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
              "scope": "manifest_only_no_redirects_no_segments_no_decode",
              "results": [probe(eligible[i], args.timeout) for i in ids]}
    output = json.dumps(report, indent=2) + "\n"
    if args.output:
        target = pathlib.Path(args.output)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(output, encoding="utf-8")
    print(output)
    return 0 if all(r["manifest_status"] == "manifest_reachable" for r in report["results"]) else 1


if __name__ == "__main__":
    sys.exit(main())
