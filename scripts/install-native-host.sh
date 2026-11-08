#!/usr/bin/env sh
set -eu
HOST_EXE="${1:?Usage: install-native-host.sh /absolute/path/to/ubs-native-host EXTENSION_ID}"
EXTENSION_ID="${2:?Pass the extension ID shown by the browser}"
[ -x "$HOST_EXE" ] || { echo "Native host executable is not executable: $HOST_EXE" >&2; exit 1; }
case "$EXTENSION_ID" in
  *[!a-p]*|'') echo 'Extension ID must contain exactly 32 characters a-p.' >&2; exit 1;;
esac
[ "${#EXTENSION_ID}" -eq 32 ] || { echo 'Extension ID must contain exactly 32 characters a-p.' >&2; exit 1; }
case "$(uname -s)" in
  Darwin) BASE="$HOME/Library/Application Support" ;;
  *) BASE="${XDG_CONFIG_HOME:-$HOME/.config}" ;;
esac
MANIFEST="$BASE/ubs-native-messaging/com.universal-browser-sync.bridge.json"
mkdir -p "$(dirname "$MANIFEST")"
HOST_ABS="$(cd "$(dirname "$HOST_EXE")" && pwd)/$(basename "$HOST_EXE")"
cat > "$MANIFEST" <<JSON
{
  "name": "com.universal-browser-sync.bridge",
  "description": "Universal Browser Sync metadata-only native messaging bridge",
  "path": "$HOST_ABS",
  "type": "stdio",
  "allowed_origins": ["chrome-extension://$EXTENSION_ID/"]
}
JSON
for dir in \
  "$HOME/.config/google-chrome/NativeMessagingHosts" \
  "$HOME/.config/microsoft-edge/NativeMessagingHosts" \
  "$HOME/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts" \
  "$HOME/.config/opera/NativeMessagingHosts" \
  "$HOME/Library/Application Support/Google/Chrome/NativeMessagingHosts" \
  "$HOME/Library/Application Support/Microsoft Edge/NativeMessagingHosts" \
  "$HOME/Library/Application Support/BraveSoftware/Brave-Browser/NativeMessagingHosts" \
  "$HOME/Library/Application Support/com.operasoftware.Opera/NativeMessagingHosts"; do
  mkdir -p "$dir"
  cp "$MANIFEST" "$dir/com.universal-browser-sync.bridge.json"
done
chmod 600 "$MANIFEST"
echo "Registered UBS native host for Chromium-family browsers. Restart the browser."
echo "Shared manifest: $MANIFEST"
