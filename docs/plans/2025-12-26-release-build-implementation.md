# Multi-Platform Build & Release Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create GitHub Actions workflow that builds Voiceboard for all platforms on merge to main and creates an automatic release.

**Architecture:** Single release.yml workflow with version generation job, parallel build matrix for 4 targets, and release job that uploads all artifacts with checksums.

**Tech Stack:** GitHub Actions, Tauri, Rust cross-compilation, Node.js

---

## Task 1: Create Release Workflow File

**Files:**
- Create: `.github/workflows/release.yml`

**Step 1: Create the workflow file**

Create `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  # Generate CalVer version
  version:
    runs-on: ubuntu-latest
    outputs:
      version: ${{ steps.version.outputs.version }}
    steps:
      - name: Generate CalVer version
        id: version
        run: |
          VERSION=$(date -u '+%Y%m%d.%H%M')
          echo "version=$VERSION" >> $GITHUB_OUTPUT
          echo "Generated version: $VERSION"

  # Build for all platforms
  build:
    needs: version
    strategy:
      fail-fast: false
      matrix:
        include:
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            arch: x64
            platform: windows

          - os: macos-latest
            target: aarch64-apple-darwin
            arch: arm64
            platform: macos

          - os: macos-13
            target: x86_64-apple-darwin
            arch: x64
            platform: macos

          - os: ubuntu-22.04
            target: x86_64-unknown-linux-gnu
            arch: x64
            platform: linux

    runs-on: ${{ matrix.os }}
    env:
      VERSION: ${{ needs.version.outputs.version }}

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Cache Rust
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri -> target

      - name: Install Linux dependencies
        if: matrix.platform == 'linux'
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libwebkit2gtk-4.1-dev \
            libappindicator3-dev \
            librsvg2-dev \
            patchelf \
            libssl-dev \
            libasound2-dev

      - name: Install npm dependencies
        run: npm ci

      - name: Update version in tauri.conf.json
        shell: bash
        run: |
          sed -i.bak 's/"version": "[^"]*"/"version": "${{ env.VERSION }}"/' src-tauri/tauri.conf.json
          cat src-tauri/tauri.conf.json | grep version

      - name: Build Tauri app
        run: npm run tauri build -- --target ${{ matrix.target }}

      - name: Rename artifacts (Windows)
        if: matrix.platform == 'windows'
        shell: pwsh
        run: |
          $version = "${{ env.VERSION }}"
          $arch = "${{ matrix.arch }}"

          # MSI
          $msi = Get-ChildItem -Path "src-tauri/target/${{ matrix.target }}/release/bundle/msi/*.msi" | Select-Object -First 1
          $newMsi = "voiceboard-$version-windows-$arch.msi"
          Copy-Item $msi.FullName -Destination $newMsi

          # Create zip
          $exe = "src-tauri/target/${{ matrix.target }}/release/Voiceboard.exe"
          $zip = "voiceboard-$version-windows-$arch.zip"
          Compress-Archive -Path $exe -DestinationPath $zip

      - name: Rename artifacts (macOS)
        if: matrix.platform == 'macos'
        run: |
          VERSION="${{ env.VERSION }}"
          ARCH="${{ matrix.arch }}"

          # DMG
          DMG=$(find src-tauri/target/${{ matrix.target }}/release/bundle/dmg -name "*.dmg" | head -1)
          cp "$DMG" "voiceboard-$VERSION-macos-$ARCH.dmg"

          # Create tar.gz from .app
          APP=$(find src-tauri/target/${{ matrix.target }}/release/bundle/macos -name "*.app" -type d | head -1)
          tar -czvf "voiceboard-$VERSION-macos-$ARCH.tar.gz" -C "$(dirname "$APP")" "$(basename "$APP")"

      - name: Rename artifacts (Linux)
        if: matrix.platform == 'linux'
        run: |
          VERSION="${{ env.VERSION }}"
          ARCH="${{ matrix.arch }}"

          # AppImage
          APPIMAGE=$(find src-tauri/target/${{ matrix.target }}/release/bundle/appimage -name "*.AppImage" | head -1)
          cp "$APPIMAGE" "voiceboard-$VERSION-linux-$ARCH.AppImage"

          # Create tar.gz
          BINARY="src-tauri/target/${{ matrix.target }}/release/voiceboard"
          tar -czvf "voiceboard-$VERSION-linux-$ARCH.tar.gz" -C "src-tauri/target/${{ matrix.target }}/release" "voiceboard"

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: voiceboard-${{ matrix.platform }}-${{ matrix.arch }}
          path: |
            voiceboard-*.msi
            voiceboard-*.zip
            voiceboard-*.dmg
            voiceboard-*.tar.gz
            voiceboard-*.AppImage
          if-no-files-found: error

  # Create GitHub Release
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

      - name: List artifacts
        run: ls -la artifacts/

      - name: Generate checksums
        run: |
          cd artifacts
          sha256sum voiceboard-* > SHA256SUMS.txt
          cat SHA256SUMS.txt

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          tag_name: v${{ needs.version.outputs.version }}
          name: Voiceboard ${{ needs.version.outputs.version }}
          draft: false
          prerelease: false
          body: |
            ## Voiceboard ${{ needs.version.outputs.version }}

            Automated release from main branch.

            ### Downloads

            | Platform | Installer | Archive |
            |----------|-----------|---------|
            | Windows x64 | [.msi](https://github.com/${{ github.repository }}/releases/download/v${{ needs.version.outputs.version }}/voiceboard-${{ needs.version.outputs.version }}-windows-x64.msi) | [.zip](https://github.com/${{ github.repository }}/releases/download/v${{ needs.version.outputs.version }}/voiceboard-${{ needs.version.outputs.version }}-windows-x64.zip) |
            | macOS Apple Silicon | [.dmg](https://github.com/${{ github.repository }}/releases/download/v${{ needs.version.outputs.version }}/voiceboard-${{ needs.version.outputs.version }}-macos-arm64.dmg) | [.tar.gz](https://github.com/${{ github.repository }}/releases/download/v${{ needs.version.outputs.version }}/voiceboard-${{ needs.version.outputs.version }}-macos-arm64.tar.gz) |
            | macOS Intel | [.dmg](https://github.com/${{ github.repository }}/releases/download/v${{ needs.version.outputs.version }}/voiceboard-${{ needs.version.outputs.version }}-macos-x64.dmg) | [.tar.gz](https://github.com/${{ github.repository }}/releases/download/v${{ needs.version.outputs.version }}/voiceboard-${{ needs.version.outputs.version }}-macos-x64.tar.gz) |
            | Linux x64 | [.AppImage](https://github.com/${{ github.repository }}/releases/download/v${{ needs.version.outputs.version }}/voiceboard-${{ needs.version.outputs.version }}-linux-x64.AppImage) | [.tar.gz](https://github.com/${{ github.repository }}/releases/download/v${{ needs.version.outputs.version }}/voiceboard-${{ needs.version.outputs.version }}-linux-x64.tar.gz) |

            ### Verification

            Download `SHA256SUMS.txt` and verify:
            ```bash
            sha256sum -c SHA256SUMS.txt
            ```

            ### Notes

            - **Windows**: No code signing - you may see a SmartScreen warning
            - **macOS**: No notarization - right-click and select "Open" to bypass Gatekeeper
            - **Linux**: Make AppImage executable: `chmod +x voiceboard-*.AppImage`
          files: |
            artifacts/voiceboard-*
            artifacts/SHA256SUMS.txt
```

**Step 2: Verify file created**

Run: `cat .github/workflows/release.yml | head -20`

Expected: First 20 lines of workflow displayed

**Step 3: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: add multi-platform release workflow"
```

---

## Task 2: Add Tauri Build Script

**Files:**
- Modify: `package.json`

**Step 1: Add tauri build script to package.json**

The current scripts don't have a dedicated tauri build. Add it:

In `package.json`, find the `"scripts"` section and add:

```json
"scripts": {
  "ng": "ng",
  "start": "ng serve",
  "build": "ng build",
  "watch": "ng build --watch --configuration development",
  "tauri": "tauri",
  "tauri:build": "tauri build"
}
```

**Step 2: Verify**

Run: `cat package.json | grep tauri`

Expected: Shows both `"tauri"` and `"tauri:build"` scripts

**Step 3: Commit**

```bash
git add package.json
git commit -m "chore: add tauri:build npm script"
```

---

## Task 3: Ensure Tauri Icons Exist

**Files:**
- Check: `src-tauri/icons/`

**Step 1: Verify icons directory**

Run: `ls -la src-tauri/icons/`

Expected: Should see icon files (32x32.png, 128x128.png, icon.icns, icon.ico)

**Step 2: If icons missing, generate them**

If icons are missing, create placeholder icons:

Run: `cd src-tauri && npm run tauri icon` (if you have a source icon)

Or create the directory and add placeholder message:

```bash
mkdir -p src-tauri/icons
echo "TODO: Add app icons" > src-tauri/icons/README.md
```

**Step 3: Commit if changes made**

```bash
git add src-tauri/icons/
git commit -m "chore: ensure icons directory exists"
```

---

## Task 4: Update ROADMAP

**Files:**
- Modify: `ROADMAP.md`

**Step 1: Update Phase 2 with completed items**

In `ROADMAP.md`, update the Phase 2 section:

```markdown
## Phase 2 - Distribution & CI/CD

### Done
- [x] **GitHub Actions CI**
  - Automated build and compilation
  - Clippy linting
  - Automated tests

- [x] **Multi-platform Release Build**
  - Windows x64 (.msi, .zip)
  - macOS ARM64 and x64 (.dmg, .tar.gz)
  - Linux x64 (.AppImage, .tar.gz)
  - CalVer versioning (YYYYMMDD.HHMM)
  - Automatic GitHub Release on merge to main
  - SHA256 checksums

### To Do
- [ ] **Windows Installer Improvements**
  - Code signing (requires certificate)
  - Bundled Virtual Audio Driver

- [ ] **macOS Improvements**
  - Code signing + notarization (requires Apple Developer)

- [ ] **Linux Support**
  - Virtual audio device (PulseAudio/PipeWire)
  - Additional packages (.deb, .rpm)

- [ ] **Auto-Update System**
  - Use tauri-plugin-updater
  - Update notifications in app
```

**Step 2: Commit**

```bash
git add ROADMAP.md
git commit -m "docs: update ROADMAP with release build progress"
```

---

## Task 5: Test Workflow Locally (Optional)

**Files:** None

**Step 1: Test Tauri build locally**

Run: `npm run tauri build`

Expected: Build succeeds, creates installer in `src-tauri/target/release/bundle/`

**Step 2: Check generated artifacts**

Run:
- macOS: `ls src-tauri/target/release/bundle/dmg/`
- Windows: `ls src-tauri/target/release/bundle/msi/`
- Linux: `ls src-tauri/target/release/bundle/appimage/`

Expected: Installer files present

---

## Task 6: Push and Verify

**Files:** None

**Step 1: Push all changes**

Run: `git push origin main`

**Step 2: Monitor GitHub Actions**

Open: `https://github.com/<owner>/voiceboard/actions`

Expected:
- `Release` workflow starts
- All 4 build jobs run in parallel
- Release job creates GitHub Release

**Step 3: Verify Release**

Open: `https://github.com/<owner>/voiceboard/releases`

Expected:
- Release `v20251226.XXXX` exists
- All 8 artifacts + SHA256SUMS.txt attached
- Release notes show download table

---

## Summary

| Task | Description | Commit |
|------|-------------|--------|
| 1 | Create release.yml workflow | `ci: add multi-platform release workflow` |
| 2 | Add tauri:build script | `chore: add tauri:build npm script` |
| 3 | Ensure icons exist | `chore: ensure icons directory exists` |
| 4 | Update ROADMAP | `docs: update ROADMAP with release build progress` |
| 5 | Test locally | (optional verification) |
| 6 | Push and verify | (manual verification) |

## Troubleshooting

### Linux build fails with missing dependencies
The workflow installs required libs. If new deps needed, add to the `apt-get install` step.

### macOS build fails on macos-13
Intel Macs use older Xcode. May need to pin Xcode version with `xcode-select`.

### Windows MSI not found
Check `src-tauri/tauri.conf.json` has `bundle.targets: "all"` or includes `"msi"`.

### Version not updated
The `sed` command modifies `tauri.conf.json` in place. On macOS, uses `.bak` backup.
