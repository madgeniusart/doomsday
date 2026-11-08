# Native messaging bridge setup

This bridge connects the companion extension to the desktop app through the browser's Native Messaging protocol. It carries only a health ping and aggregate bookmark/history/extension counts. It explicitly rejects password, cookie-value, URL, and authentication-session transfer.

## Build the native host

From the project root:

```sh
cd native-host
cargo build --release
```

The executable is `native-host/target/release/ubs-native-host` (`ubs-native-host.exe` on Windows). Keep it at a stable path; do not move it after registration.

## Load the extension

1. Open `chrome://extensions` (or the equivalent extensions page in Edge, Brave, or Opera).
2. Enable Developer mode.
3. Choose **Load unpacked** and select the project's `browser-extension` directory.
4. Copy the extension ID shown on that page.

Browsers require explicit user action to load an unpacked extension. UBS does not silently install or sideload extensions.

## Register the native host

Windows PowerShell:

```powershell
.\scripts\install-native-host.ps1 -HostExecutable "C:\path\to\ubs-native-host.exe" -ExtensionId "YOUR_32_CHARACTER_EXTENSION_ID"
```

macOS / Linux:

```sh
./scripts/install-native-host.sh /absolute/path/to/ubs-native-host YOUR_32_CHARACTER_EXTENSION_ID
```

Restart the browser. Open the UBS Helper popup; the service worker attempts a `PING`. In the desktop app, open Settings and select **Check companion connection**. A `bridge_connected` event confirms the native host reached the app's local event queue.

## Troubleshooting

- Confirm the extension ID exactly matches the installed unpacked extension.
- Confirm the host executable exists at the registered absolute path and is executable.
- Restart the browser after changing the manifest or registry registration.
- On Windows, inspect `HKCU\Software\Google\Chrome\NativeMessagingHosts\com.universal-browser-sync.bridge` (and the equivalent Edge/Brave/Opera keys).
- The native host event queue is stored in the UBS application data directory as `extension-bridge-events.jsonl`; it contains only metadata events.

The installer registers the native host, not the browser extension itself. Browser vendors intentionally require user approval or enterprise policy for extension installation.
