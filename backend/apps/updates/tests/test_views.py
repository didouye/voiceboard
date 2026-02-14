"""Tests for the updates endpoint."""

from unittest.mock import patch, MagicMock

from django.core.cache import cache
from django.test import TestCase, override_settings
from rest_framework.test import APIClient


MOCK_STABLE_RELEASE = {
    "tag_name": "v26.2.44",
    "prerelease": False,
    "published_at": "2026-02-14T10:30:00Z",
    "body": "Voiceboard 26.2.44",
    "assets": [
        {"name": "voiceboard-26.2.44-macos-arm64.tar.gz", "browser_download_url": "https://github.com/didouye/voiceboard/releases/download/v26.2.44/voiceboard-26.2.44-macos-arm64.tar.gz"},
        {"name": "voiceboard-26.2.44-macos-arm64.tar.gz.sig", "browser_download_url": "https://github.com/didouye/voiceboard/releases/download/v26.2.44/voiceboard-26.2.44-macos-arm64.tar.gz.sig"},
        {"name": "voiceboard-26.2.44-macos-x64.tar.gz", "browser_download_url": "https://github.com/didouye/voiceboard/releases/download/v26.2.44/voiceboard-26.2.44-macos-x64.tar.gz"},
        {"name": "voiceboard-26.2.44-macos-x64.tar.gz.sig", "browser_download_url": "https://github.com/didouye/voiceboard/releases/download/v26.2.44/voiceboard-26.2.44-macos-x64.tar.gz.sig"},
        {"name": "voiceboard-26.2.44-windows-x64.nsis.zip", "browser_download_url": "https://github.com/didouye/voiceboard/releases/download/v26.2.44/voiceboard-26.2.44-windows-x64.nsis.zip"},
        {"name": "voiceboard-26.2.44-windows-x64.nsis.zip.sig", "browser_download_url": "https://github.com/didouye/voiceboard/releases/download/v26.2.44/voiceboard-26.2.44-windows-x64.nsis.zip.sig"},
        {"name": "voiceboard-26.2.44-linux-x64.tar.gz", "browser_download_url": "https://github.com/didouye/voiceboard/releases/download/v26.2.44/voiceboard-26.2.44-linux-x64.tar.gz"},
        {"name": "voiceboard-26.2.44-linux-x64.tar.gz.sig", "browser_download_url": "https://github.com/didouye/voiceboard/releases/download/v26.2.44/voiceboard-26.2.44-linux-x64.tar.gz.sig"},
    ],
}

MOCK_BETA_RELEASE = {
    "tag_name": "v26.2.45-beta",
    "prerelease": True,
    "published_at": "2026-02-15T10:30:00Z",
    "body": "Voiceboard 26.2.45 (Beta)",
    "assets": [
        {"name": "voiceboard-26.2.45-macos-arm64.tar.gz", "browser_download_url": "https://github.com/didouye/voiceboard/releases/download/v26.2.45-beta/voiceboard-26.2.45-macos-arm64.tar.gz"},
        {"name": "voiceboard-26.2.45-macos-arm64.tar.gz.sig", "browser_download_url": "https://github.com/didouye/voiceboard/releases/download/v26.2.45-beta/voiceboard-26.2.45-macos-arm64.tar.gz.sig"},
        {"name": "voiceboard-26.2.45-macos-x64.tar.gz", "browser_download_url": "https://github.com/didouye/voiceboard/releases/download/v26.2.45-beta/voiceboard-26.2.45-macos-x64.tar.gz"},
        {"name": "voiceboard-26.2.45-macos-x64.tar.gz.sig", "browser_download_url": "https://github.com/didouye/voiceboard/releases/download/v26.2.45-beta/voiceboard-26.2.45-macos-x64.tar.gz.sig"},
        {"name": "voiceboard-26.2.45-windows-x64.nsis.zip", "browser_download_url": "https://github.com/didouye/voiceboard/releases/download/v26.2.45-beta/voiceboard-26.2.45-windows-x64.nsis.zip"},
        {"name": "voiceboard-26.2.45-windows-x64.nsis.zip.sig", "browser_download_url": "https://github.com/didouye/voiceboard/releases/download/v26.2.45-beta/voiceboard-26.2.45-windows-x64.nsis.zip.sig"},
        {"name": "voiceboard-26.2.45-linux-x64.tar.gz", "browser_download_url": "https://github.com/didouye/voiceboard/releases/download/v26.2.45-beta/voiceboard-26.2.45-linux-x64.tar.gz"},
        {"name": "voiceboard-26.2.45-linux-x64.tar.gz.sig", "browser_download_url": "https://github.com/didouye/voiceboard/releases/download/v26.2.45-beta/voiceboard-26.2.45-linux-x64.tar.gz.sig"},
    ],
}


def _mock_sig_response(url, **kwargs):
    """Return a mock response for signature file downloads."""
    resp = MagicMock()
    resp.status_code = 200
    resp.text = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZw=="
    return resp


@override_settings(GITHUB_REPO="didouye/voiceboard")
class LatestUpdateViewTest(TestCase):
    def setUp(self):
        self.client = APIClient()
        cache.clear()

    @patch("apps.updates.views.requests.get")
    def test_stable_channel_returns_latest_non_prerelease(self, mock_get):
        """Stable channel should return the latest non-prerelease."""
        # First call: GitHub releases list. Second+ calls: signature downloads.
        releases_resp = MagicMock()
        releases_resp.status_code = 200
        releases_resp.json.return_value = [MOCK_BETA_RELEASE, MOCK_STABLE_RELEASE]

        mock_get.side_effect = lambda url, **kwargs: (
            releases_resp if "api.github.com" in url else _mock_sig_response(url)
        )

        resp = self.client.get("/api/updates/latest", {"channel": "stable"})
        self.assertEqual(resp.status_code, 200)
        data = resp.json()
        self.assertEqual(data["version"], "26.2.44")
        self.assertIn("darwin-aarch64", data["platforms"])
        self.assertIn("darwin-x86_64", data["platforms"])
        self.assertIn("windows-x86_64", data["platforms"])
        self.assertIn("linux-x86_64", data["platforms"])

    @patch("apps.updates.views.requests.get")
    def test_beta_channel_returns_latest_regardless_of_prerelease(self, mock_get):
        """Beta channel should return the most recent release (pre-release or not)."""
        releases_resp = MagicMock()
        releases_resp.status_code = 200
        releases_resp.json.return_value = [MOCK_BETA_RELEASE, MOCK_STABLE_RELEASE]

        mock_get.side_effect = lambda url, **kwargs: (
            releases_resp if "api.github.com" in url else _mock_sig_response(url)
        )

        resp = self.client.get("/api/updates/latest", {"channel": "beta"})
        self.assertEqual(resp.status_code, 200)
        data = resp.json()
        self.assertEqual(data["version"], "26.2.45")

    @patch("apps.updates.views.requests.get")
    def test_default_channel_is_stable(self, mock_get):
        """No channel param should default to stable."""
        releases_resp = MagicMock()
        releases_resp.status_code = 200
        releases_resp.json.return_value = [MOCK_BETA_RELEASE, MOCK_STABLE_RELEASE]

        mock_get.side_effect = lambda url, **kwargs: (
            releases_resp if "api.github.com" in url else _mock_sig_response(url)
        )

        resp = self.client.get("/api/updates/latest")
        self.assertEqual(resp.status_code, 200)
        data = resp.json()
        self.assertEqual(data["version"], "26.2.44")

    @patch("apps.updates.views.requests.get")
    def test_invalid_channel_returns_400(self, mock_get):
        """Invalid channel should return 400."""
        resp = self.client.get("/api/updates/latest", {"channel": "nightly"})
        self.assertEqual(resp.status_code, 400)

    @patch("apps.updates.views.requests.get")
    def test_github_api_failure_returns_502(self, mock_get):
        """GitHub API failure should return 502."""
        mock_resp = MagicMock()
        mock_resp.status_code = 403
        mock_resp.text = "rate limited"
        mock_get.return_value = mock_resp

        resp = self.client.get("/api/updates/latest", {"channel": "stable"})
        self.assertEqual(resp.status_code, 502)

    @patch("apps.updates.views.requests.get")
    def test_no_matching_release_returns_204(self, mock_get):
        """No release found should return 204."""
        releases_resp = MagicMock()
        releases_resp.status_code = 200
        releases_resp.json.return_value = []
        mock_get.return_value = releases_resp

        resp = self.client.get("/api/updates/latest", {"channel": "stable"})
        self.assertEqual(resp.status_code, 204)

    @patch("apps.updates.views.requests.get")
    def test_manifest_contains_signatures(self, mock_get):
        """Manifest platform entries should contain signatures."""
        releases_resp = MagicMock()
        releases_resp.status_code = 200
        releases_resp.json.return_value = [MOCK_STABLE_RELEASE]

        mock_get.side_effect = lambda url, **kwargs: (
            releases_resp if "api.github.com" in url else _mock_sig_response(url)
        )

        resp = self.client.get("/api/updates/latest", {"channel": "stable"})
        data = resp.json()
        for platform in data["platforms"].values():
            self.assertIn("url", platform)
            self.assertIn("signature", platform)
            self.assertTrue(len(platform["signature"]) > 0)

    @patch("apps.updates.views.requests.get")
    def test_endpoint_is_publicly_accessible(self, mock_get):
        """Endpoint should not require authentication."""
        releases_resp = MagicMock()
        releases_resp.status_code = 200
        releases_resp.json.return_value = [MOCK_STABLE_RELEASE]

        mock_get.side_effect = lambda url, **kwargs: (
            releases_resp if "api.github.com" in url else _mock_sig_response(url)
        )

        # No auth token set on client
        resp = self.client.get("/api/updates/latest", {"channel": "stable"})
        self.assertEqual(resp.status_code, 200)
