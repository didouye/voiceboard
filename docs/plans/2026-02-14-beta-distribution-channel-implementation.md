# Beta Distribution Channel - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a beta distribution channel so users can opt-in to test upcoming features via an in-app toggle.

**Architecture:** Single unified release workflow triggered by `main` (stable) and `develop` (beta). A backend Django endpoint proxies GitHub Releases to serve Tauri update manifests per channel. The Tauri updater is customized at runtime to point to the correct channel endpoint.

**Tech Stack:** Django REST Framework, GitHub Releases API, Tauri v2 updater plugin (custom builder), Angular signals, GitHub Actions

---

### Task 1: Backend — Create `updates` Django app skeleton

**Files:**
- Create: `backend/apps/updates/__init__.py`
- Create: `backend/apps/updates/apps.py`
- Create: `backend/apps/updates/urls.py`
- Create: `backend/apps/updates/views.py`
- Create: `backend/apps/updates/tests/__init__.py`
- Create: `backend/apps/updates/tests/test_views.py`
- Modify: `backend/config/settings/base.py:19-42` (add to INSTALLED_APPS)
- Modify: `backend/config/urls.py:7-12` (add URL route)

**Step 1: Create app skeleton files**

Create `backend/apps/updates/__init__.py` (empty).

Create `backend/apps/updates/apps.py`:
```python
"""Updates app configuration."""

from django.apps import AppConfig


class UpdatesConfig(AppConfig):
    default_auto_field = "django.db.models.BigAutoField"
    name = "apps.updates"
    verbose_name = "Updates"
```

Create `backend/apps/updates/urls.py`:
```python
"""Updates URL patterns."""

from django.urls import path

from .views import LatestUpdateView

urlpatterns = [
    path("latest", LatestUpdateView.as_view(), name="updates-latest"),
]
```

Create `backend/apps/updates/views.py` (empty view for now):
```python
"""Update manifest endpoint that proxies GitHub Releases."""

from rest_framework.views import APIView


class LatestUpdateView(APIView):
    pass
```

Create `backend/apps/updates/tests/__init__.py` (empty).

**Step 2: Register app and URL**

In `backend/config/settings/base.py`, add `"apps.updates"` to `INSTALLED_APPS` after `"apps.teams"` (line 41):
```python
    "apps.teams",
    "apps.updates",
```

In `backend/config/urls.py`, add after the teams URL (line 11):
```python
    path("api/updates/", include("apps.updates.urls")),
```

**Step 3: Add GitHub config to settings**

In `backend/config/settings/base.py`, add at the end (after line 197):
```python
# GitHub (for update manifest proxy)
GITHUB_REPO = os.environ.get("GITHUB_REPO", "didouye/voiceboard")
```

**Step 4: Commit**

```bash
git add backend/apps/updates/ backend/config/settings/base.py backend/config/urls.py
git commit -m "feat(updates): create updates Django app skeleton"
```

---

### Task 2: Backend — Implement update manifest endpoint with tests

**Files:**
- Modify: `backend/apps/updates/views.py`
- Modify: `backend/apps/updates/tests/test_views.py`

**Step 1: Write the failing tests**

Create `backend/apps/updates/tests/test_views.py`:
```python
"""Tests for the updates endpoint."""

from unittest.mock import patch, MagicMock

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
```

**Step 2: Run tests to verify they fail**

Run: `cd backend && python -m pytest apps/updates/tests/test_views.py -v`
Expected: FAIL (view not implemented)

**Step 3: Implement the view**

Replace `backend/apps/updates/views.py`:
```python
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
```

**Step 4: Add Django cache configuration**

In `backend/config/settings/base.py`, add after the Redis URL (line 164):
```python
# Cache (for GitHub API rate limiting)
CACHES = {
    "default": {
        "BACKEND": "django.core.cache.backends.locmem.LocMemCache",
        "LOCATION": "voiceboard-cache",
    }
}
```

**Step 5: Run tests to verify they pass**

Run: `cd backend && python -m pytest apps/updates/tests/test_views.py -v`
Expected: All PASS

**Step 6: Commit**

```bash
git add backend/apps/updates/ backend/config/settings/base.py backend/config/urls.py
git commit -m "feat(updates): add update manifest endpoint proxying GitHub Releases"
```

---

### Task 3: Rust — Add `update_channel` to AppSettings

**Files:**
- Modify: `src-tauri/src/domain/settings.rs:76-101`
- Modify: `src-tauri/src/application/commands.rs:123-219` (DTOs)

**Step 1: Write the failing tests**

Add to the existing tests in `src-tauri/src/domain/settings.rs`, after the last test (line 288):
```rust
    #[test]
    fn test_update_channel_default_is_stable() {
        let settings = AppSettings::new();
        assert_eq!(settings.update_channel, UpdateChannel::Stable);
    }

    #[test]
    fn test_update_channel_serialization() {
        let mut settings = AppSettings::new();
        settings.update_channel = UpdateChannel::Beta;
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"update_channel\":\"beta\""));
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.update_channel, UpdateChannel::Beta);
    }

    #[test]
    fn test_update_channel_default_on_missing_field() {
        // Old settings JSON without update_channel should default to Stable
        let json = r#"{"audio":{"input_device_id":null,"output_device_id":null,"preview_device_id":null,"master_volume":1.0,"sample_rate":48000,"buffer_size":1024,"mic_monitoring":false},"start_minimized":false,"auto_start_mixing":false}"#;
        let settings: AppSettings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.update_channel, UpdateChannel::Stable);
    }
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml -p voiceboard -- domain::settings`
Expected: FAIL (UpdateChannel not defined)

**Step 3: Implement UpdateChannel and add to AppSettings**

In `src-tauri/src/domain/settings.rs`, add after the default functions (before line 22):
```rust
/// Update distribution channel
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UpdateChannel {
    Stable,
    Beta,
}

impl Default for UpdateChannel {
    fn default() -> Self {
        Self::Stable
    }
}

fn default_update_channel() -> UpdateChannel {
    UpdateChannel::default()
}
```

Add to `AppSettings` struct (after `auto_start_mixing` field):
```rust
    /// Update distribution channel (stable or beta)
    #[serde(default = "default_update_channel")]
    pub update_channel: UpdateChannel,
```

Update `AppSettings::new()` to include:
```rust
            update_channel: UpdateChannel::Stable,
```

**Step 4: Update DTOs in commands.rs**

In `src-tauri/src/application/commands.rs`, add to `AppSettingsDto` (after `auto_start_mixing` field, around line 201):
```rust
    #[serde(default)]
    pub update_channel: String,
```

Update `From<&AppSettings> for AppSettingsDto` to include:
```rust
            update_channel: format!("{:?}", settings.update_channel).to_lowercase(),
```

Update `From<AppSettingsDto> for AppSettings` to include:
```rust
            update_channel: match dto.update_channel.as_str() {
                "beta" => crate::domain::UpdateChannel::Beta,
                _ => crate::domain::UpdateChannel::Stable,
            },
```

**Step 5: Export UpdateChannel from domain module**

In `src-tauri/src/domain/mod.rs`, add `UpdateChannel` to the public exports.

**Step 6: Run tests to verify they pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml -p voiceboard -- domain::settings`
Expected: All PASS

**Step 7: Run clippy and format**

Run: `cargo fmt --manifest-path src-tauri/Cargo.toml && cargo clippy --manifest-path src-tauri/Cargo.toml`

**Step 8: Commit**

```bash
git add src-tauri/src/domain/ src-tauri/src/application/commands.rs
git commit -m "feat(settings): add update_channel field to AppSettings"
```

---

### Task 4: Rust — Implement channel-aware custom updater

**Files:**
- Modify: `src-tauri/src/application/commands.rs:1760-1892` (update commands)
- Modify: `src-tauri/tauri.conf.json:43-50` (updater config)

**Step 1: Update tauri.conf.json updater endpoint**

Replace the updater endpoint in `src-tauri/tauri.conf.json` (lines 44-49) to use the backend URL. Keep the pubkey:
```json
    "updater": {
      "endpoints": [
        "https://voiceboard.cloud/api/updates/latest?channel=stable"
      ],
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEIyODQxMkQ4QjgxM0ExNUEKUldSYW9STzQyQktFc3FPNE1VejhxYWR0UnNRUm11dDVsRUhtUDlON1l3UHFkWVZUeC9DSkxzbW0K"
    }
```

Note: This is the fallback endpoint used if the custom builder fails. The actual channel is injected at runtime.

**Step 2: Modify check_for_update to use channel-aware endpoint**

Replace the `check_for_update` command (lines 1769-1820) with:
```rust
/// Check if an update is available (uses channel from settings)
#[tauri::command]
pub async fn check_for_update(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<UpdateInfo, String> {
    let channel = {
        let settings = state.settings.read().await;
        format!("{:?}", settings.update_channel).to_lowercase()
    };

    tracing::info!(channel = %channel, "Starting update check");

    let api_url = match option_env!("VOICEBOARD_API_URL") {
        Some(url) => url,
        None => "http://localhost:8000/api",
    };
    let endpoint = format!(
        "{}/updates/latest?channel={}",
        api_url, channel
    );

    let updater = app
        .updater_builder()
        .endpoints(vec![endpoint.parse().map_err(|e: url::ParseError| e.to_string())?])
        .build()
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to create updater instance");
            e.to_string()
        })?;

    match updater.check().await {
        Ok(Some(update)) => {
            tracing::info!(
                version = %update.version,
                current_version = env!("CARGO_PKG_VERSION"),
                channel = %channel,
                "Update available"
            );
            Ok(UpdateInfo {
                available: true,
                version: Some(update.version.clone()),
                body: update.body.clone(),
            })
        }
        Ok(None) => {
            tracing::info!(
                current_version = env!("CARGO_PKG_VERSION"),
                channel = %channel,
                "No update available"
            );
            Ok(UpdateInfo {
                available: false,
                version: None,
                body: None,
            })
        }
        Err(e) => {
            tracing::error!(error = %e, channel = %channel, "Update check failed");
            Err(format!("Update check failed: {}", e))
        }
    }
}
```

**Step 3: Modify install_update similarly**

Replace the `install_update` command (lines 1824-1892) with:
```rust
/// Download and install an available update, then restart
#[tauri::command]
pub async fn install_update(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let channel = {
        let settings = state.settings.read().await;
        format!("{:?}", settings.update_channel).to_lowercase()
    };

    tracing::info!(channel = %channel, "Starting update installation");

    let api_url = match option_env!("VOICEBOARD_API_URL") {
        Some(url) => url,
        None => "http://localhost:8000/api",
    };
    let endpoint = format!(
        "{}/updates/latest?channel={}",
        api_url, channel
    );

    let updater = app
        .updater_builder()
        .endpoints(vec![endpoint.parse().map_err(|e: url::ParseError| e.to_string())?])
        .build()
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to create updater instance");
            e.to_string()
        })?;

    let update = match updater.check().await {
        Ok(Some(update)) => {
            tracing::info!(version = %update.version, "Update found, proceeding with download");
            update
        }
        Ok(None) => {
            tracing::warn!("No update available when trying to install");
            return Err("No update available".to_string());
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to check for update during installation");
            return Err(format!("Failed to check for update: {}", e));
        }
    };

    tracing::info!(version = %update.version, "Starting download and installation");

    let download_result = update
        .download_and_install(
            |downloaded, total| {
                if let Some(total) = total {
                    let percent = (downloaded as f64 / total as f64 * 100.0) as u32;
                    if percent.is_multiple_of(25) {
                        tracing::debug!(
                            downloaded_bytes = downloaded,
                            total_bytes = total,
                            percent = percent,
                            "Download progress"
                        );
                    }
                }
            },
            || {
                tracing::info!("Download complete, starting installation");
            },
        )
        .await;

    match download_result {
        Ok(()) => {
            tracing::info!("Update installed successfully, restarting application");
            app.restart();
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to download and install update");
            Err(format!("Failed to install update: {}", e))
        }
    }
}
```

**Step 4: Add `url` crate to dependencies**

In `src-tauri/Cargo.toml`, add:
```toml
url = "2"
```

**Step 5: Add use statement for UpdaterExt**

At the top of `commands.rs`, add:
```rust
use tauri_plugin_updater::UpdaterExt;
```

**Step 6: Run clippy and format**

Run: `cargo fmt --manifest-path src-tauri/Cargo.toml && cargo clippy --manifest-path src-tauri/Cargo.toml`

**Step 7: Commit**

```bash
git add src-tauri/
git commit -m "feat(updater): use channel-aware backend endpoint for update checks"
```

---

### Task 5: Rust — Add set_update_channel command

**Files:**
- Modify: `src-tauri/src/application/commands.rs` (add new command)
- Modify: `src-tauri/src/lib.rs` (register command)

**Step 1: Add the command**

In `src-tauri/src/application/commands.rs`, add after the `install_update` command:
```rust
/// Set the update distribution channel
#[tauri::command]
pub async fn set_update_channel(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    channel: String,
) -> Result<(), String> {
    let update_channel = match channel.as_str() {
        "beta" => crate::domain::UpdateChannel::Beta,
        "stable" => crate::domain::UpdateChannel::Stable,
        _ => return Err(format!("Invalid channel: {}. Must be 'stable' or 'beta'", channel)),
    };

    tracing::info!(channel = %channel, "Setting update channel");

    {
        let mut settings = state.settings.write().await;
        settings.update_channel = update_channel;
    }

    // Persist to store
    let settings = state.settings.read().await;
    let dto = AppSettingsDto::from(&*settings);
    drop(settings);

    let store = app.store(SETTINGS_STORE).map_err(|e| e.to_string())?;
    store.set(
        SETTINGS_KEY,
        serde_json::to_value(&dto).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| e.to_string())?;

    Ok(())
}
```

**Step 2: Register the command in lib.rs**

In `src-tauri/src/lib.rs`, find the `.invoke_handler(tauri::generate_handler![...])` block and add `commands::set_update_channel` to the list.

**Step 3: Run clippy and format**

Run: `cargo fmt --manifest-path src-tauri/Cargo.toml && cargo clippy --manifest-path src-tauri/Cargo.toml`

**Step 4: Commit**

```bash
git add src-tauri/src/application/commands.rs src-tauri/src/lib.rs
git commit -m "feat(updater): add set_update_channel command"
```

---

### Task 6: Angular — Add update channel to models and settings UI

**Files:**
- Modify: `src/app/core/models/audio-device.model.ts:44-48`
- Modify: `src/app/core/services/tauri.service.ts:213-256`
- Modify: `src/app/shared/components/settings-popup/settings-popup.component.ts`

**Step 1: Add updateChannel to TypeScript models**

In `src/app/core/models/audio-device.model.ts`, add to `AppSettings` interface (after `autoStartMixing`):
```typescript
  updateChannel: 'stable' | 'beta';
```

**Step 2: Update TauriService mapping**

In `src/app/core/services/tauri.service.ts`, update `mapSettings()` to include:
```typescript
      updateChannel: s.update_channel ?? 'stable',
```

Update `unmapSettings()` to include:
```typescript
      update_channel: s.updateChannel,
```

**Step 3: Add toggle to settings popup**

In `src/app/shared/components/settings-popup/settings-popup.component.ts`:

Add a signal for the update channel:
```typescript
protected readonly _updateChannel = signal<'stable' | 'beta'>('stable');
protected readonly updateChannel = this._updateChannel.asReadonly();
```

In the settings loading method, initialize it:
```typescript
this._updateChannel.set(settings.updateChannel);
```

Add a toggle method:
```typescript
  async toggleUpdateChannel(): Promise<void> {
    const newChannel = this._updateChannel() === 'stable' ? 'beta' : 'stable';
    try {
      await invoke('set_update_channel', { channel: newChannel });
      this._updateChannel.set(newChannel);
    } catch (err) {
      console.error('Failed to set update channel:', err);
    }
  }
```

Add the UI in the template (in the settings section, near the debug mode toggle):
```html
<!-- Update Channel -->
<div class="flex items-center justify-between">
  <div>
    <span class="text-sm font-medium">Update channel</span>
    <p class="text-xs text-gray-400">
      @if (updateChannel() === 'beta') {
        Beta — Test upcoming features (may contain bugs)
      } @else {
        Stable — Tested and validated releases
      }
    </p>
  </div>
  <button
    (click)="toggleUpdateChannel()"
    [class]="updateChannel() === 'beta'
      ? 'bg-amber-500 relative inline-flex h-6 w-11 items-center rounded-full'
      : 'bg-gray-600 relative inline-flex h-6 w-11 items-center rounded-full'"
  >
    <span
      [class]="updateChannel() === 'beta'
        ? 'translate-x-6 inline-block h-4 w-4 rounded-full bg-white transition'
        : 'translate-x-1 inline-block h-4 w-4 rounded-full bg-white transition'"
    ></span>
  </button>
</div>
```

**Step 4: Commit**

```bash
git add src/app/
git commit -m "feat(ui): add update channel toggle to settings"
```

---

### Task 7: CI/CD — Modify release workflow for dual-channel support

**Files:**
- Modify: `.github/workflows/release.yml`

**Step 1: Add develop branch trigger**

Change the trigger (line 3-5) to:
```yaml
on:
  push:
    branches: [main, develop]
```

**Step 2: Add channel detection to version job**

In the `version` job, after the version generation step (line 31), add channel detection:
```yaml
      - name: Detect channel
        id: channel
        run: |
          if [ "${{ github.ref_name }}" = "develop" ]; then
            echo "is_beta=true" >> $GITHUB_OUTPUT
            echo "tag_suffix=-beta" >> $GITHUB_OUTPUT
            echo "release_name_suffix= (Beta)" >> $GITHUB_OUTPUT
            echo "sentry_env=beta" >> $GITHUB_OUTPUT
          else
            echo "is_beta=false" >> $GITHUB_OUTPUT
            echo "tag_suffix=" >> $GITHUB_OUTPUT
            echo "release_name_suffix=" >> $GITHUB_OUTPUT
            echo "sentry_env=production" >> $GITHUB_OUTPUT
          fi
```

Add new outputs to the version job:
```yaml
      is_beta: ${{ steps.channel.outputs.is_beta }}
      tag_suffix: ${{ steps.channel.outputs.tag_suffix }}
      release_name_suffix: ${{ steps.channel.outputs.release_name_suffix }}
      sentry_env: ${{ steps.channel.outputs.sentry_env }}
```

**Step 3: Update Sentry job**

In the sentry job, use the environment output. After `sentry-cli releases finalize`, add:
```yaml
      - name: Deploy to environment
        run: |
          sentry-cli releases deploys "$APP_VERSION" new -e "${{ needs.version.outputs.sentry_env }}"
```

**Step 4: Update release job**

In the `release` job, modify the `Create GitHub Release` step:

Replace the tag and name lines:
```yaml
          tag_name: v${{ needs.version.outputs.release_tag }}${{ needs.version.outputs.tag_suffix }}
          name: Voiceboard ${{ needs.version.outputs.release_tag }}${{ needs.version.outputs.release_name_suffix }}
          prerelease: ${{ needs.version.outputs.is_beta == 'true' }}
```

Update the body to reflect the channel:
```yaml
          body: |
            ## Voiceboard ${{ needs.version.outputs.release_tag }}${{ needs.version.outputs.release_name_suffix }}

            Automated release from `${{ github.ref_name }}` branch.

            **App Version:** ${{ needs.version.outputs.app_version }}
            **Channel:** ${{ needs.version.outputs.is_beta == 'true' && 'Beta' || 'Stable' }}
```

**Step 5: Remove latest.json generation**

Remove the entire "Generate update manifest" step (lines 288-346) from the release job. The backend now serves this dynamically.

Remove `artifacts/latest.json` from the `files:` list in the "Create GitHub Release" step.

**Step 6: Update tag in artifact URLs**

In the "Generate update manifest" step and release body download URLs, the tag now includes the suffix. Update the download URLs in the release body to use the full tag:
```yaml
v${{ needs.version.outputs.release_tag }}${{ needs.version.outputs.tag_suffix }}
```

**Step 7: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "feat(ci): add develop branch support for beta releases"
```

---

### Task 8: Final verification and cleanup

**Step 1: Run all Rust tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: All PASS

**Step 2: Run Rust linting**

Run: `cargo fmt --manifest-path src-tauri/Cargo.toml && cargo clippy --manifest-path src-tauri/Cargo.toml`
Expected: No warnings

**Step 3: Run backend tests**

Run: `cd backend && python -m pytest apps/updates/ -v`
Expected: All PASS

**Step 4: Build Angular frontend**

Run: `npm run build`
Expected: Build succeeds

**Step 5: Verify all changes**

Run: `git diff --stat main`
Review the list of changed files matches expectations.
