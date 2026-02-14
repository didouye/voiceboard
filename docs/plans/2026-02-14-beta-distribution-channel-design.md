# Beta Distribution Channel - Design

> **Date:** 2026-02-14
> **Status:** Ready for implementation

## Overview

Add a beta distribution channel so users can opt-in to test upcoming features before they reach the stable release. Users choose their channel via an in-app toggle in settings. The stable channel remains the default.

**Key decisions:**
- **Opt-in via in-app toggle** (not separate installers)
- **`develop` branch** triggers beta builds, `main` triggers stable builds
- **Single unified CI workflow** with shared `run_number` for consistent versioning
- **Backend endpoint** serves the Tauri update manifest per channel
- **Single backend instance**, backwards-compatible for both channels
- **No downgrade** when switching from beta back to stable

## Versioning

CalVer format `YY.MM.RUN_NUMBER` using a single shared `github.run_number` counter across both channels.

The version in the Tauri binary is always purely numeric (e.g., `26.2.44`). The Git tag appends `-beta` for beta releases (e.g., `v26.2.44-beta`) for human readability only.

**Constraints:** Each version component must be ≤ 65535 (Windows MSI/NSIS 16-bit limit).

**Example timeline:**

| Push | Branch | Run # | Tauri Version | Git Tag | GitHub Release |
|------|--------|-------|---------------|---------|----------------|
| 1 | develop | 42 | 26.2.42 | v26.2.42-beta | Pre-release |
| 2 | develop | 43 | 26.2.43 | v26.2.43-beta | Pre-release |
| 3 | main | 44 | 26.2.44 | v26.2.44 | Release |
| 4 | develop | 45 | 26.2.45 | v26.2.45-beta | Pre-release |

## CI/CD: Unified Workflow

Merge both channels into a single `release.yml` triggered by both branches:

```yaml
on:
  push:
    branches: [main, develop]
```

### Conditional logic by branch

| Aspect | `main` | `develop` |
|--------|--------|-----------|
| GitHub Release type | release | pre-release |
| Git tag format | `v26.2.44` | `v26.2.44-beta` |
| Release name | `Voiceboard 26.2.44` | `Voiceboard 26.2.44 (Beta)` |
| Sentry environment | `production` | `beta` |
| `VOICEBOARD_API_URL` | `https://voiceboard.cloud/api` | `https://voiceboard.cloud/api` (same backend) |

### Release creation step

```yaml
- name: Create GitHub Release
  uses: softprops/action-gh-release@v2
  with:
    tag_name: v${{ steps.version.outputs.app_version }}${{ github.ref_name == 'develop' && '-beta' || '' }}
    prerelease: ${{ github.ref_name == 'develop' }}
    name: Voiceboard ${{ steps.version.outputs.app_version }}${{ github.ref_name == 'develop' && ' (Beta)' || '' }}
```

### Removed from workflow

The `latest.json` asset generation is removed from the workflow. The backend now builds this manifest dynamically.

## Backend: Update Endpoint

New Django app `updates` with a single endpoint.

### Endpoint

```
GET /api/updates/latest?channel=stable&target=darwin-aarch64&current_version=26.2.38
```

**Parameters:**
- `channel` — `stable` (default) or `beta`
- `target` — Tauri platform identifier (e.g., `darwin-aarch64`, `windows-x86_64`, `linux-x86_64`)
- `current_version` — The app's current version

### Logic

1. Call GitHub Releases API: `GET /repos/didouye/voiceboard/releases`
2. Filter by channel:
   - `stable` → latest release where `prerelease == false`
   - `beta` → latest release regardless of pre-release flag (so beta users also receive stable releases when they are newer)
3. Find the asset matching `target` (`.tar.gz` or `.nsis.zip`) and its `.sig` file
4. Compare `current_version` with release version — if not newer, return `204 No Content`
5. Build and return the Tauri update manifest:

```json
{
  "version": "26.2.44",
  "notes": "Voiceboard 26.2.44 (Beta)",
  "pub_date": "2026-02-15T10:30:00Z",
  "platforms": {
    "darwin-aarch64": {
      "url": "https://github.com/didouye/voiceboard/releases/download/v26.2.44-beta/voiceboard-...-macos-arm64.tar.gz",
      "signature": "dW50cnVzdGVkI..."
    }
  }
}
```

### Caching

Cache the GitHub API response in Redis for 5 minutes to avoid rate limiting.

### Platform-to-asset mapping

| Target | Asset pattern |
|--------|--------------|
| `darwin-aarch64` | `*-macos-arm64.tar.gz` |
| `darwin-x86_64` | `*-macos-x64.tar.gz` |
| `windows-x86_64` | `*-windows-x64-nsis.zip` |
| `linux-x86_64` | `*-linux-x64.AppImage.tar.gz` |

## Desktop App: Updater Configuration

### Tauri updater (Rust)

Replace the static endpoint in `tauri.conf.json` with a custom updater implementation. Tauri v2 does not support custom template variables in endpoint URLs, so the update check is done manually in Rust code:

1. Read `update_channel` from Tauri Store (`settings.json`), default to `"stable"`
2. Build the request URL: `https://voiceboard.cloud/api/updates/latest?channel={channel}&target={target}&current_version={current_version}`
3. Make the HTTP request
4. If response is `204`, no update available
5. If response is `200`, pass the manifest to Tauri's updater API to download and install

### Angular UI: Settings toggle

Add a toggle in the settings view:

```
Update channel
  ○ Stable — Tested and validated releases
  ● Beta — Test upcoming features (may contain bugs)
```

**Behavior on change:**
1. Persist choice to Tauri Store (`settings.json` → `update_channel: "stable" | "beta"`)
2. Immediately trigger an update check on the new channel
3. If an update is available, show the standard update notification

### No downgrade

When switching from beta to stable, if the current version is newer than the latest stable, the backend returns `204`. The user stays on their current version until a new stable release surpasses it. This is the standard behavior used by VS Code, Chrome, and other apps with beta channels.

## End-to-End Flow

### 1. Developer pushes to `develop`

```
git push origin develop
  → release.yml triggers (run_number: 44)
  → Multi-platform build, version 26.2.44
  → GitHub Pre-release "Voiceboard 26.2.44 (Beta)", tag v26.2.44-beta
  → Sentry release in "beta" environment
```

### 2. Beta user opens the app

```
App reads update_channel = "beta" from store
  → GET https://voiceboard.cloud/api/updates/latest?channel=beta&target=darwin-aarch64&current_version=26.2.42
  → Backend finds pre-release v26.2.44-beta, version > 26.2.42
  → Returns Tauri manifest with URL + signature
  → App downloads and installs the update
```

### 3. Feature validated, developer merges develop → main

```
git push origin main
  → release.yml triggers (run_number: 45)
  → Build version 26.2.45
  → GitHub Release "Voiceboard 26.2.45", tag v26.2.45
  → Sentry release in "production" environment
```

### 4. All users receive the stable release

Stable users receive 26.2.45 as usual. Beta users also receive 26.2.45 because the backend serves the most recent release regardless of pre-release flag for the beta channel.

## Files Changed

| File | Change |
|------|--------|
| `.github/workflows/release.yml` | Add `develop` trigger, conditional pre-release logic, remove `latest.json` generation |
| `backend/updates/` | New Django app with update endpoint |
| `backend/config/urls.py` | Register updates app URLs |
| `backend/config/settings/base.py` | Add `updates` to `INSTALLED_APPS`, GitHub API config |
| `src-tauri/tauri.conf.json` | Remove static updater endpoint |
| `src-tauri/src/application/updater.rs` | New module: custom update check logic |
| `src-tauri/src/application/commands.rs` | New commands: `set_update_channel`, `check_for_update` |
| `src/app/settings/` | Add update channel toggle to settings UI |
