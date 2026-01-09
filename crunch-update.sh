#!/bin/bash
# The MIT License (MIT)
# Copyright © 2021 Aukbit Ltd.
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.

set -e

DIRNAME="$HOME/crunch-bot"
FILENAME="$DIRNAME/crunch"
TEMPDIR=$(mktemp -d)

cleanup() {
    rm -rf "$TEMPDIR"
}
trap cleanup EXIT

read -p "Enter the Crunch version that you would like to download (e.g.: 0.21.0): " INPUT_VERSION
if [ -z "$INPUT_VERSION" ]; then
    INPUT_VERSION="0.21.0"
fi

read -p "Enter a specific Ubuntu version the release was built on, or leave empty for latest. Available options: [ubuntu-22.04, ubuntu-20.04, linux-musl]: " TARGET_VERSION

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
