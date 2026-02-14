"""Update manifest endpoint that proxies GitHub Releases."""

import logging
import re

import requests
from django.conf import settings
from django.core.cache import cache
from rest_framework.permissions import AllowAny
from rest_framework.response import Response
from rest_framework.views import APIView

logger = logging.getLogger(__name__)

VALID_CHANNELS = {"stable", "beta"}
CACHE_KEY = "github_releases"
CACHE_TTL = 300  # 5 minutes

# Map Tauri platform identifiers to asset filename patterns
PLATFORM_ASSET_PATTERNS = {
    "darwin-aarch64": "macos-arm64.tar.gz",
    "darwin-x86_64": "macos-x64.tar.gz",
    "windows-x86_64": "windows-x64.nsis.zip",
    "linux-x86_64": "linux-x64.tar.gz",
}


class LatestUpdateView(APIView):
    """Serve Tauri update manifests by proxying GitHub Releases.

    GET /api/updates/latest?channel=stable
    GET /api/updates/latest?channel=beta
    """

    permission_classes = [AllowAny]
    authentication_classes = []

    def get(self, request):
        channel = request.query_params.get("channel", "stable")
        if channel not in VALID_CHANNELS:
            return Response(
                {"error": f"Invalid channel. Must be one of: {', '.join(VALID_CHANNELS)}"},
                status=400,
            )

        releases = self._fetch_releases()
        if releases is None:
            return Response({"error": "Failed to fetch releases from GitHub"}, status=502)

        release = self._pick_release(releases, channel)
        if release is None:
            return Response(status=204)

        manifest = self._build_manifest(release)
        if manifest is None:
            return Response({"error": "Failed to build update manifest"}, status=502)

        return Response(manifest)

    def _fetch_releases(self):
        """Fetch releases from GitHub API with caching."""
        cached = cache.get(CACHE_KEY)
        if cached is not None:
            return cached

        repo = getattr(settings, "GITHUB_REPO", "didouye/voiceboard")
        url = f"https://api.github.com/repos/{repo}/releases"
        try:
            resp = requests.get(url, headers={"Accept": "application/vnd.github.v3+json"}, timeout=10)
        except requests.RequestException:
            logger.exception("GitHub API request failed")
            return None

        if resp.status_code != 200:
            logger.error("GitHub API returned %s: %s", resp.status_code, resp.text[:200])
            return None

        data = resp.json()
        cache.set(CACHE_KEY, data, CACHE_TTL)
        return data

    def _pick_release(self, releases, channel):
        """Pick the appropriate release for the channel.

        - stable: latest non-prerelease
        - beta: latest release regardless of prerelease flag
        """
        if channel == "stable":
            for release in releases:
                if not release.get("prerelease", False):
                    return release
            return None
        else:  # beta
            return releases[0] if releases else None

    def _build_manifest(self, release):
        """Build a Tauri-compatible update manifest from a GitHub release."""
        tag = release["tag_name"]
        # Extract numeric version from tag (e.g., "v26.2.44-beta" -> "26.2.44")
        match = re.match(r"v?(\d+\.\d+\.\d+)", tag)
        if not match:
            logger.error("Could not parse version from tag: %s", tag)
            return None

        version = match.group(1)
        assets = {a["name"]: a["browser_download_url"] for a in release.get("assets", [])}

        platforms = {}
        for platform_id, pattern in PLATFORM_ASSET_PATTERNS.items():
            # Find the matching asset and its signature
            asset_url = None
            sig_url = None
            for name, url in assets.items():
                if name.endswith(pattern) and not name.endswith(f"{pattern}.sig"):
                    asset_url = url
                if name.endswith(f"{pattern}.sig"):
                    sig_url = url

            if asset_url and sig_url:
                signature = self._download_signature(sig_url)
                platforms[platform_id] = {
                    "url": asset_url,
                    "signature": signature or "",
                }

        return {
            "version": version,
            "notes": release.get("body", f"Voiceboard {version}"),
            "pub_date": release.get("published_at", ""),
            "platforms": platforms,
        }

    def _download_signature(self, sig_url):
        """Download a signature file content."""
        try:
            resp = requests.get(sig_url, timeout=10)
            if resp.status_code == 200:
                return resp.text.strip()
        except requests.RequestException:
            logger.exception("Failed to download signature from %s", sig_url)
        return ""
