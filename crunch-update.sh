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

#!/bin/bash
#
# > make a file executable
# chmod +x ./crunch-update.sh

DIRNAME="~/crunch-bot"
FILENAME="$DIRNAME/crunch"

read -p "Enter the Crunch version that you would like to download (e.g.: 0.21.0): " INPUT_VERSION

if [ "$INPUT_VERSION" = "" ]; then
        INPUT_VERSION="0.21.0"
fi

read -p "Enter a specific Ubuntu version the release was built on, or leave empty for latest. Available options: [ubuntu-22.04, ubuntu-20.04, linux-musl]: " TARGET_VERSION

if [ "$TARGET_VERSION" != "" ]; then
        TARGET_VERSION=".${TARGET_VERSION//.}"
fi

URI="https://github.com/turboflakes/crunch/releases/download/v$INPUT_VERSION/crunch$TARGET_VERSION"
URI_SHA256="https://github.com/turboflakes/crunch/releases/download/v$INPUT_VERSION/crunch.sha256$TARGET_VERSION"
wget $URI && wget $URI_SHA256

if [ "$TARGET_VERSION" != "" ]; then
        mv "crunch$TARGET_VERSION" crunch
fi

if sha256sum -c "crunch.sha256$TARGET_VERSION" 2>&1 | grep -q 'OK'
then
        if [ ! -d "$DIRNAME" ]
        then
                mkdir $DIRNAME
        fi
        if [[ -f "$FILENAME" ]]
        then
                mv "$FILENAME" "$FILENAME.backup"
        fi
        rm "crunch.sha256$TARGET_VERSION"
        chmod +x crunch
        mv crunch "$FILENAME"
        echo "** crunch v$INPUT_VERSION successfully downloaded and verified $FILENAME **"
else
        echo "Error: SHA256 doesn't match!"
        rm "$FILENAME*"
fi
