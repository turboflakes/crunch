#!/bin/bash
#
# > make a file executable
# chmod +x ./crunch-update.sh

set -e

DIRNAME="$HOME/crunch-bot"
FILENAME="$DIRNAME/crunch"
TEMPDIR=$(mktemp -d)

cleanup() {
    rm -rf "$TEMPDIR"
}
trap cleanup EXIT

read -p "Enter the Crunch version that you would like to download (e.g.: 0.28.1), or leave empty for latest: " INPUT_VERSION
if [ -z "$INPUT_VERSION" ]; then
    INPUT_VERSION="0.28.1"
fi

read -p "Enter a specific version the release was built on, or leave empty for Ubuntu latest. Available options: [ubuntu-22.04, linux-musl]: " TARGET_VERSION

DOWNLOAD_SUFFIX=""
if [ -n "$TARGET_VERSION" ]; then
    DOWNLOAD_SUFFIX=".${TARGET_VERSION//.}"
fi

URI="https://github.com/turboflakes/crunch/releases/download/v$INPUT_VERSION/crunch$DOWNLOAD_SUFFIX"
URI_SHA256="https://github.com/turboflakes/crunch/releases/download/v$INPUT_VERSION/crunch.sha256$DOWNLOAD_SUFFIX"

echo "Downloading crunch v$INPUT_VERSION..."
cd "$TEMPDIR"
wget -q --show-progress "$URI" -O crunch || { echo "Error: Failed to download crunch binary"; exit 1; }
wget -q --show-progress "$URI_SHA256" -O crunch.sha256 || { echo "Error: Failed to download checksum file"; exit 1; }

# Fix checksum file to match local filename
sed -i 's/crunch[^ ]*/crunch/' crunch.sha256

if sha256sum -c crunch.sha256 2>&1 | grep -q 'OK'; then
    echo "Checksum verified."

    # Create directory if it doesn't exist
    mkdir -p "$DIRNAME"

    # Backup existing binary
    if [[ -f "$FILENAME" ]]; then
        mv "$FILENAME" "$FILENAME.backup"
        echo "Existing binary backed up to $FILENAME.backup"
    fi

    # Install new binary
    chmod +x crunch
    mv crunch "$FILENAME"

    echo ""
    echo "** crunch v$INPUT_VERSION successfully installed at $FILENAME **"
    echo ""
    echo "NOTE: If running as a systemd service, restart it to apply the update."
else
    echo "Error: SHA256 checksum verification failed!"
    exit 1
fi
