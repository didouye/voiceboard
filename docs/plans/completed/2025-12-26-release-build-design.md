# Multi-Platform Build & Release Design

## Overview

Automated CI workflow that builds Voiceboard for Windows, macOS (Intel + ARM), and Linux on every merge to main, then creates a GitHub Release with all artifacts.

## Requirements

- Build on merge to main
- CalVer versioning: `YYYYMMDD.HHMM` (UTC)
- Artifacts: installers + archives + checksums
- Naming: `voiceboard-VERSION-OS-ARCH.ext`
- No code signing (for now)

## Workflow Structure

```
.github/workflows/
├── ci.yml          # Existing - lint, test (all branches)
└── release.yml     # New - build + release (main only)
```

## Jobs Pipeline

```
┌─────────────────────────────────────────────────────┐
│                    release.yml                       │
├─────────────────────────────────────────────────────┤
│  version:     Generate CalVer YYYYMMDD.HHMM         │
│      ↓                                               │
│  build:       Matrix [windows, macos, linux]        │
│      ↓         × [x64, arm64 for mac]               │
│  release:     Create GitHub Release + upload        │
└─────────────────────────────────────────────────────┘
```

## Version Generation

```yaml
jobs:
  version:
    runs-on: ubuntu-latest
    outputs:
      version: ${{ steps.version.outputs.version }}
    steps:
      - name: Generate CalVer
        id: version
        run: |
          VERSION=$(date -u '+%Y%m%d.%H%M')
          echo "version=$VERSION" >> $GITHUB_OUTPUT
```

Example: merge at 01:15 UTC on Dec 26, 2025 → `20251226.0115`

## Build Matrix

| OS | Runner | Target | Arch | Installer | Archive |
|----|--------|--------|------|-----------|---------|
| Windows | windows-latest | x86_64-pc-windows-msvc | x64 | .msi | .zip |
| macOS | macos-latest | aarch64-apple-darwin | arm64 | .dmg | .tar.gz |
| macOS | macos-13 | x86_64-apple-darwin | x64 | .dmg | .tar.gz |
| Linux | ubuntu-22.04 | x86_64-unknown-linux-gnu | x64 | .AppImage | .tar.gz |

```yaml
build:
  needs: version
  strategy:
    matrix:
      include:
        - os: windows-latest
          target: x86_64-pc-windows-msvc
          arch: x64
          ext: msi
          archive: zip

        - os: macos-latest
          target: aarch64-apple-darwin
          arch: arm64
          ext: dmg
          archive: tar.gz

        - os: macos-13
          target: x86_64-apple-darwin
          arch: x64
          ext: dmg
          archive: tar.gz

        - os: ubuntu-22.04
          target: x86_64-unknown-linux-gnu
          arch: x64
          ext: AppImage
          archive: tar.gz

  runs-on: ${{ matrix.os }}
```

## Build Steps

1. Checkout repository
2. Setup Node.js 20 + cache
3. Setup Rust toolchain + target
4. Install dependencies (`npm ci`)
5. Build Tauri (`npm run tauri build`)
6. Rename artifacts: `voiceboard-VERSION-OS-ARCH.ext`
7. Upload artifacts

## Release Job

```yaml
release:
  needs: [version, build]
  runs-on: ubuntu-latest
  permissions:
    contents: write
  steps:
    - name: Download all artifacts
      uses: actions/download-artifact@v4
      with:
        path: artifacts
        merge-multiple: true

    - name: Generate checksums
      run: |
        cd artifacts
        sha256sum * > SHA256SUMS.txt

    - name: Create GitHub Release
      uses: softprops/action-gh-release@v2
      with:
        tag_name: v${{ needs.version.outputs.version }}
        name: Voiceboard ${{ needs.version.outputs.version }}
        body: |
          ## Voiceboard ${{ needs.version.outputs.version }}

          Automated release from main branch.

          ### Downloads
          - **Windows**: `voiceboard-...-windows-x64.msi`
          - **macOS (Apple Silicon)**: `voiceboard-...-macos-arm64.dmg`
          - **macOS (Intel)**: `voiceboard-...-macos-x64.dmg`
          - **Linux**: `voiceboard-...-linux-x64.AppImage`

          ### Checksums
          Verify with: `sha256sum -c SHA256SUMS.txt`
        files: artifacts/*
```

## Artifact Naming

Format: `voiceboard-{VERSION}-{OS}-{ARCH}.{EXT}`

Examples for version `20251226.0115`:
- `voiceboard-20251226.0115-windows-x64.msi`
- `voiceboard-20251226.0115-windows-x64.zip`
- `voiceboard-20251226.0115-macos-arm64.dmg`
- `voiceboard-20251226.0115-macos-arm64.tar.gz`
- `voiceboard-20251226.0115-macos-x64.dmg`
- `voiceboard-20251226.0115-macos-x64.tar.gz`
- `voiceboard-20251226.0115-linux-x64.AppImage`
- `voiceboard-20251226.0115-linux-x64.tar.gz`
- `SHA256SUMS.txt`

## Future Improvements

- [ ] Code signing for Windows (requires certificate)
- [ ] Code signing + notarization for macOS (requires Apple Developer account)
- [ ] ARM64 Windows build
- [ ] ARM64 Linux build
- [ ] Changelog generation from commits
