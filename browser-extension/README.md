# Universal Browser Sync Helper

Manifest V3 companion extension for user-approved cookie metadata inspection and the metadata-only native-messaging bridge.

## Install for local testing

1. Open the browser extensions page (`chrome://extensions` in Chrome; equivalent page in Edge, Brave, or Opera).
2. Enable Developer mode.
3. Choose **Load unpacked** and select this `browser-extension` directory.
4. Copy the extension ID shown by the browser.
5. Build and register the native host using [`native-host/README.md`](native-host/README.md).
6. Restart the browser and open the UBS Helper popup.

Browsers require explicit user action to load an unpacked extension. UBS does not silently sideload extensions.

## Privacy and scope

- Site cookie metadata is inspected only after the user grants optional host permission. Cookie values are never read.
- The extension does not read `localStorage`, `sessionStorage`, IndexedDB, passwords, or page contents.
- Native messaging carries only a connection ping and optional aggregate profile counts.
- Authentication cookies, session tokens, and passwords are never sent to the desktop app.
- The native host queues metadata events in the UBS app-data directory for the desktop app to read.
- Revoke site access in the extension popup or browser extension settings.

## Supported browsers

Chrome, Edge, Brave, and Opera (Chromium Native Messaging). Firefox and Safari require separate manifest/install packaging and are not enabled by this companion build yet.
