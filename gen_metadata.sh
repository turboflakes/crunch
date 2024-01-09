#!/bin/bash

# Two step script
#   1. check for a creditcoin node at TARGET_URL
#   2. extract the metadata from the node 

#   Note: both WS and HTTP are served via the same port
TARGET_URL=${1:-http://127.0.0.1:9944}
TARGET_DEST=${2:-metadata/creditcoin_metadata.scale}
TARGET_VERSION=14 # needed for some reason after I upgraded subxt
CURL_PARAMS="-H 'Content-Type: application/json' -d '{\"id\":\"1\", \"jsonrpc\":\"2.0\", \"method\": \"state_getMetadata\", \"params\":[]}' $TARGET_URL"

COUNTER=0
# make sure there is a node running at TARGET_URL
while [[ "$(eval curl -s -o /dev/null -w '%{http_code}' "$CURL_PARAMS")" != "200" && $COUNTER -lt 10 ]]; do
    echo "ATTEMPT: $COUNTER - Not ready yet ....."
    (( COUNTER=COUNTER+1 ))
    sleep 2
done

# fail if we still can't connect after 10 attempts
set -e

# Note: using eval b/c params are specified as string above
eval curl "$CURL_PARAMS" > /dev/null

subxt metadata --url $TARGET_URL --version $TARGET_VERSION -f bytes > $TARGET_DEST

# Check for the target file and sound the alarm if its not found
if [ -e $TARGET_DEST ]
then
    echo "$TARGET_DEST generated successfully"
else
    echo "$TARGET_DEST not found" >&2
    exit 1
fi