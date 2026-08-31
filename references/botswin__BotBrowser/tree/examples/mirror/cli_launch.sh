#!/bin/bash

# Mirror CLI Launch Script
#
# This script launches a controller + 2 clients using command-line flags
# for easy manual testing and visual inspection of synchronization.
#
# Usage:
#   ./cli_launch.sh
#
# Environment variables (optional):
#   BOTBROWSER_EXEC_PATH            Path to BotBrowser executable (default: chromium)
#   BOT_PROFILE_PATH                Path to .enc profile file (required)
#   BOT_URL                         Target URL to load (default: google.com)
#   BOT_MIRROR_CONTROLLER_ENDPOINT  Controller endpoint (default: 127.0.0.1:9990)
#   BOT_MIRROR_CLIENT_ENDPOINT      Client endpoint (default: 127.0.0.1:9990)

set -e

CHROMIUM="${BOTBROWSER_EXEC_PATH:-chromium}"
PROFILE="${BOT_PROFILE_PATH}"
URL="${BOT_URL:-google.com}"
CONTROLLER_ENDPOINT="${BOT_MIRROR_CONTROLLER_ENDPOINT:-127.0.0.1:9990}"
CLIENT_ENDPOINT="${BOT_MIRROR_CLIENT_ENDPOINT:-127.0.0.1:9990}"

if [ -z "$PROFILE" ]; then
  echo "Error: BOT_PROFILE_PATH environment variable is required"
  echo "Usage: BOT_PROFILE_PATH=/path/to/profile.enc ./cli_launch.sh"
  exit 1
fi

if [ ! -f "$PROFILE" ]; then
  echo "Error: Profile not found: $PROFILE"
  exit 1
fi

if ! command -v "$CHROMIUM" &> /dev/null; then
  echo "Error: Chromium not found at: $CHROMIUM"
  exit 1
fi

echo "Launching Mirror setup..."
echo "  Chromium: $CHROMIUM"
echo "  Profile: $PROFILE"
echo "  URL: $URL"
echo "  Controller endpoint: $CONTROLLER_ENDPOINT"
echo "  Client endpoint: $CLIENT_ENDPOINT"
echo ""

# Launch controller
echo "Starting controller on $CONTROLLER_ENDPOINT..."
"$CHROMIUM" \
  --user-data-dir="$(mktemp -d)" \
  --bot-profile="$PROFILE" \
  --bot-mirror-controller-endpoint=$CONTROLLER_ENDPOINT \
  --window-position=10,50 \
  --window-size=400,600 \
  "$URL" &
CONTROLLER_PID=$!

# Launch client 1
echo "Starting client 1..."
"$CHROMIUM" \
  --user-data-dir="$(mktemp -d)" \
  --bot-profile="$PROFILE" \
  --bot-mirror-client-endpoint=$CLIENT_ENDPOINT \
  --window-position=430,50 \
  --window-size=400,600 \
  "$URL" &
CLIENT1_PID=$!

# Launch client 2
echo "Starting client 2..."
"$CHROMIUM" \
  --user-data-dir="$(mktemp -d)" \
  --bot-profile="$PROFILE" \
  --bot-mirror-client-endpoint=$CLIENT_ENDPOINT \
  --window-position=850,50 \
  --window-size=400,600 \
  "$URL" &
CLIENT2_PID=$!

echo ""
echo "Setup complete. Three browser windows should now be visible:"
echo "  Controller (left) and 2 clients (center and right)"
echo ""
echo "Interact with the controller. Watch the clients synchronize."
echo ""
echo "Press Ctrl+C to stop."
echo ""

# Wait for user interrupt
trap "kill $CONTROLLER_PID $CLIENT1_PID $CLIENT2_PID 2>/dev/null; echo 'Stopped.'" EXIT
wait
