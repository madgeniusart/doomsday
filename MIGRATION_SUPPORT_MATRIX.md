# Migration support and security boundaries

Universal Browser Sync uses browser-owned profile formats and browser extension APIs. It must not decrypt protected browser databases, bypass operating-system credential protection, or silently copy bearer sessions.

## Implemented in this branch

| Data | Chromium destination (Chrome, Edge, Brave, Opera) | Firefox destination | Safari destination |
|---|---|---|---|
| Bookmarks | Native JSON profile merge; URL dedupe; backup; browser must be closed | Netscape HTML export for browser import | Netscape HTML export for browser import |
| History | Native SQLite visit merge; backup; browser must be closed | JSON reference export | JSON reference export |
| Cookies | Metadata report only; no values | Metadata report only; no values | Adapter/platform limitations apply |
| Extensions | Inventory export; no silent installation | Inventory export; no silent installation | Inventory export; no silent installation |
| Preferences | Reference export only; settings are not blindly applied | Reference export only | Reference export only |
| Passwords | Not transferred | Not transferred | Not transferred |
| Authenticated sessions | Not transferred | Not transferred | Not transferred |

## Why credentials and sessions are not silently migrated

Saved passwords are protected by browser and operating-system facilities (for example, OS credential stores and browser encryption keys). A profile-file copy is not a portable password export. Password migration should use the browser vendor's own export/import flow, with the user initiating it and deleting any temporary plaintext export immediately after import. This app does not read, store, or relay plaintext passwords.

Authentication cookies and tokens are bearer credentials. A session may be bound to the browser profile, device, IP address, TLS state, or server-side risk checks. Copying cookie values does not guarantee a working session and may invalidate the source session. The current helper deliberately handles cookie metadata only; it never reads cookie values or web storage. Sign in again on the destination browser when a site requires it.

## Extension installation and native messaging

Browser extension installation is governed by each browser's distribution and signing policy. Chromium-family browsers generally require a user-approved installation flow for unpacked extensions unless distributed through an approved store or managed enterprise policy. Firefox signing and Safari's app-extension packaging impose separate requirements. The current project provides an extension package and inventory export, not unattended installation. A native-messaging host must be installed and registered separately per OS and browser; it is not enabled merely by shipping a manifest in the extension archive.

## Operational requirements

- Close the destination browser before native profile database writes.
- Keep the automatically-created timestamped backup until the destination browser has been checked.
- Treat export files as sensitive browsing data and store them only in a user-controlled folder.
- A successful file/database write does not guarantee that every website or browser version will accept imported data.


## Additional credential and companion support (this implementation pass)

| Capability | Current behavior | Boundary / next step |
|---|---|---|
| Password migration | Imports browser-exported CSV, validates URL/username/password columns, encrypts the staged vault with the OS-keychain-backed AES-256-GCM key, and exports a user-selected CSV for the destination browser's built-in importer. | The final browser import is performed by the browser's own password manager. The exported CSV is plaintext and must be deleted securely after import. |
| Native messaging | Adds a native host for Chrome, Edge, Brave, and Opera. Extension sends a health ping; desktop app can drain queued connection events. | This is a one-way metadata bridge, not an arbitrary command channel. Firefox and Safari packaging are not enabled. |
| Extension installation | Provides platform registration scripts for the native host and instructions to load the extension. | Browsers require user approval or enterprise policy to install an extension; UBS does not silently install it. |
| Authenticated sessions | Not transferred. Cookie values and browser session tokens are not read by the extension or native host. | Users sign in normally in the destination browser. Sites may bind sessions to a device/browser and reject session reuse. |
