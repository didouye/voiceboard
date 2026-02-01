#!/bin/bash
# Download yt-dlp and ffmpeg binaries for all platforms
set -e

BINARIES_DIR="src-tauri/binaries"
mkdir -p "$BINARIES_DIR"

# Use RUNNER_TEMP if available (GitHub Actions), otherwise /tmp
TEMP_DIR="${RUNNER_TEMP:-/tmp}"

# Get target from argument or detect
TARGET="${1:-}"

download_ytdlp() {
    local target=$1
    local ext=""
    local ytdlp_name=""

    case "$target" in
        x86_64-pc-windows-msvc)
            ytdlp_name="yt-dlp.exe"
            ext=".exe"
            ;;
        x86_64-apple-darwin)
            ytdlp_name="yt-dlp_macos"
            ;;
        aarch64-apple-darwin)
            ytdlp_name="yt-dlp_macos"
            ;;
        x86_64-unknown-linux-gnu)
            ytdlp_name="yt-dlp_linux"
            ;;
        *)
            echo "Unknown target: $target"
            exit 1
            ;;
    esac

    local output="$BINARIES_DIR/yt-dlp-${target}${ext}"

    if [ ! -f "$output" ]; then
        echo "Downloading yt-dlp for $target..."
        curl -fL "https://github.com/yt-dlp/yt-dlp/releases/latest/download/${ytdlp_name}" -o "$output"
        chmod +x "$output"
    else
        echo "yt-dlp for $target already exists"
    fi
}

download_ffmpeg() {
    local target=$1
    local ext=""

    case "$target" in
        x86_64-pc-windows-msvc)
            ext=".exe"
            echo "Downloading ffmpeg for Windows..."
            curl -fL "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip" -o "$TEMP_DIR/ffmpeg.zip"
            unzip -o "$TEMP_DIR/ffmpeg.zip" -d "$TEMP_DIR/ffmpeg"
            cp "$TEMP_DIR/ffmpeg/ffmpeg-master-latest-win64-gpl/bin/ffmpeg.exe" "$BINARIES_DIR/ffmpeg-${target}.exe"
            rm -rf "$TEMP_DIR/ffmpeg" "$TEMP_DIR/ffmpeg.zip"
            ;;
        x86_64-apple-darwin)
            echo "Downloading ffmpeg for macOS x64..."
            curl -fL "https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip" -o "$TEMP_DIR/ffmpeg.zip"
            unzip -o "$TEMP_DIR/ffmpeg.zip" -d "$TEMP_DIR/ffmpeg"
            cp "$TEMP_DIR/ffmpeg/ffmpeg" "$BINARIES_DIR/ffmpeg-${target}"
            chmod +x "$BINARIES_DIR/ffmpeg-${target}"
            rm -rf "$TEMP_DIR/ffmpeg" "$TEMP_DIR/ffmpeg.zip"
            ;;
        aarch64-apple-darwin)
            echo "Downloading ffmpeg for macOS ARM..."
            curl -fL "https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip" -o "$TEMP_DIR/ffmpeg.zip"
            unzip -o "$TEMP_DIR/ffmpeg.zip" -d "$TEMP_DIR/ffmpeg"
            cp "$TEMP_DIR/ffmpeg/ffmpeg" "$BINARIES_DIR/ffmpeg-${target}"
            chmod +x "$BINARIES_DIR/ffmpeg-${target}"
            rm -rf "$TEMP_DIR/ffmpeg" "$TEMP_DIR/ffmpeg.zip"
            ;;
        x86_64-unknown-linux-gnu)
            echo "Downloading ffmpeg for Linux..."
            curl -fL "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz" -o "$TEMP_DIR/ffmpeg.tar.xz"
            tar -xf "$TEMP_DIR/ffmpeg.tar.xz" -C "$TEMP_DIR"
            cp "$TEMP_DIR"/ffmpeg-*-amd64-static/ffmpeg "$BINARIES_DIR/ffmpeg-${target}"
            chmod +x "$BINARIES_DIR/ffmpeg-${target}"
            rm -rf "$TEMP_DIR"/ffmpeg-* "$TEMP_DIR/ffmpeg.tar.xz"
            ;;
    esac
}

if [ -n "$TARGET" ]; then
    download_ytdlp "$TARGET"
    download_ffmpeg "$TARGET"
else
    echo "Usage: $0 <target>"
    echo "Targets: x86_64-pc-windows-msvc, x86_64-apple-darwin, aarch64-apple-darwin, x86_64-unknown-linux-gnu"
    exit 1
fi

echo "Binaries downloaded to $BINARIES_DIR"
ls -la "$BINARIES_DIR"
