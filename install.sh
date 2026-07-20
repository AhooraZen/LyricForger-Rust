#!/usr/bin/env bash
# LyricForger-Rust Instant Binary Installer & Updater for Termux & Android/Linux
set -e

BOLD="\033[1m"
CYAN="\033[36m"
GREEN="\033[32m"
YELLOW="\033[33m"
RED="\033[31m"
RESET="\033[0m"

echo -e "${CYAN}${BOLD}⚡ LyricForger-Rust Instant Binary Installer & Updater${RESET}"
echo -e "${CYAN}------------------------------------------------------${RESET}"

# 1. Environment & Architecture Detection
IS_TERMUX=false
if [ -n "$PREFIX" ] && [ -d "$PREFIX/bin" ]; then
    IS_TERMUX=true
    echo -e "${GREEN}📱 Termux environment detected.${RESET}"
else
    echo -e "${GREEN}🐧 Standard Linux environment detected.${RESET}"
fi

ARCH=$(uname -m)
echo -e "${CYAN}🔍 System Architecture: ${BOLD}${ARCH}${RESET}"

BINARY_NAME=""
case "$ARCH" in
    aarch64|arm64)
        BINARY_NAME="lyric_forger-aarch64-linux-android"
        ;;
    armv7l|armv7a|arm)
        BINARY_NAME="lyric_forger-armv7-linux-androideabi"
        ;;
    x86_64|amd64)
        BINARY_NAME="lyric_forger-x86_64-linux-android"
        ;;
    *)
        echo -e "${RED}❌ Unsupported architecture: ${ARCH}${RESET}"
        exit 1
        ;;
esac

# 2. Determine Installation Path
DEST_DIR=""
if [ "$IS_TERMUX" = true ]; then
    DEST_DIR="$PREFIX/bin"
else
    DEST_DIR="$HOME/.local/bin"
    mkdir -p "$DEST_DIR"
fi

DEST_BIN="${DEST_DIR}/lyric-forger"

# 3. Download pre-compiled binary from latest GitHub Release
DOWNLOAD_URL="https://github.com/AhooraZen/LyricForger-Rust/releases/latest/download/${BINARY_NAME}"

echo -e "${YELLOW}📥 Fetching latest pre-compiled binary from GitHub Releases...${RESET}"
echo -e "${CYAN}URL: ${DOWNLOAD_URL}${RESET}"

TEMP_BIN=$(mktemp)

if command -v curl &> /dev/null; then
    curl -sSL -f -o "$TEMP_BIN" "$DOWNLOAD_URL" || {
        # Fallback to tagged v1.0.0-android if latest tag release redirect is pending
        DOWNLOAD_URL="https://github.com/AhooraZen/LyricForger-Rust/releases/download/v1.0.0-android/${BINARY_NAME}"
        curl -sSL -f -o "$TEMP_BIN" "$DOWNLOAD_URL"
    }
elif command -v wget &> /dev/null; then
    wget -q -O "$TEMP_BIN" "$DOWNLOAD_URL" || {
        DOWNLOAD_URL="https://github.com/AhooraZen/LyricForger-Rust/releases/download/v1.0.0-android/${BINARY_NAME}"
        wget -q -O "$TEMP_BIN" "$DOWNLOAD_URL"
    }
else
    echo -e "${RED}❌ Neither curl nor wget is installed. Please install curl or wget.${RESET}"
    exit 1
fi

mv "$TEMP_BIN" "$DEST_BIN"
chmod +x "$DEST_BIN"

echo -e "${GREEN}${BOLD}🎉 Instant Installation / Update Complete! (0 Compilation Required)${RESET}"
echo -e "${GREEN}Executable installed to: ${CYAN}${DEST_BIN}${RESET}"
echo -e "${GREEN}Type ${CYAN}lyric-forger${GREEN} anywhere in Termux to launch!${RESET}"
