# Universal Browser Sync — focused reliability patch

This patch modifies the existing Tauri 2 / Rust / React application. It does not replace the project structure.

## Changes

### Backup and restore
- Reject invalid source profile paths.
- Prevent choosing a backup output folder inside the profile being archived (avoids recursive self-inclusion).
- Use unique backup filenames and a temporary-file + rename write so incomplete output is not published as a finished backup.
- Remove the completed backup file if its database record cannot be saved.
- Stop silently ignoring directory traversal, file-read, and archive-write errors.
- Skip cache/lock entries by exact name and prune skipped directories rather than walking their contents.
- Restore archive entries individually through `unpack_in`, rejecting paths that escape the chosen restore folder.
- Add Rust unit tests for backup round-tripping and traversal rejection.

### Migration reporting and validation
- Validate non-empty category selection and existing source/destination profile directories.
- Compare canonical profile paths and reject identical or nested source/destination folders.
- Reconcile Chromium `Local State` metadata with actual `Default` / `Profile *` directories, so a profile folder is still discovered when account/profile metadata is stale or missing; read its display name and available account label from `Preferences`.
- Record category-level failures as failed report items and mark a run `partial` when some categories fail instead of reporting a false full success.
- Add the `partial` status to the UI and persisted migration history.
- Distinguish `exported` files/reports from data actually imported into the destination browser.
- Mark dry-run output as `planned` rather than claiming that it was exported.
- Clarify that the cookie export is metadata-only: cookie values and active login sessions are not copied.
- Add a real Chromium-family bookmark importer for Chrome, Edge, Brave, and Opera destinations: merges new URLs into a dedicated folder, preserves nested source folders, deduplicates URLs, assigns fresh Chromium node IDs/GUIDs, and creates a timestamped destination backup before writing.
- Refuse native bookmark writes while a Chromium process or profile singleton lock marker indicates that the destination may be open.
- Use a temporary file and platform-aware replacement (including Windows rollback) when committing the updated Chromium Bookmarks database.
- Keep the portable Netscape HTML export for Firefox and Safari destinations, where native import is not implemented by this pass.
- Remove browser `--version` subprocess execution from discovery; browser version is left unset rather than risking visible browser windows during detection.
- Count only native imports as `migrated`; exports and skipped items remain visible and cause a `partial` run instead of a false full-success status.

## Important capability boundary
The current adapters read cookie metadata, not cookie values, and the migration engine does not install browser sessions. This patch intentionally keeps that boundary: a user's one-click consent can authorize supported profile migration, but it cannot guarantee that websites will accept a copied session or remove browser/website re-authentication requirements. The UI/report now avoids claiming that cookie metadata export is a completed login migration.

## Verification status
- The source archive was unpacked and these edits were made against the existing 92-file source tree.
- `npm run typecheck`: passed after the migration-engine changes.
- ESLint: 0 errors; 6 existing React hook dependency warnings remain.
- Vitest could not start because the archived `node_modules` is missing the Linux Rolldown native binding (it reports `Cannot find native binding`).
- Vite production build is blocked by the same missing Linux native binding.
- Rust tests/build could not be run because `rustc`, `cargo`, and `rustfmt` are not installed in this environment. The new Rust unit test is included but not executed.
- Run the existing GitHub Actions workflow or a complete local Node/Rust toolchain before releasing this patch.

### Companion extension and export hardening (follow-up pass)
- Reworked the companion extension to request optional host access only after the user selects **Allow for this site**; it no longer requests blanket access to every website at install time.
- Removed the extension's raw `localStorage` / `sessionStorage` collection paths and removed the unused content script and broad cookie-list message handler.
- The extension now displays cookie metadata only. Cookie values are never returned, including for cookies heuristically classified as non-authentication cookies.
- Added a visible per-site permission revocation action and clearer disclosure that authentication sessions are not transferred.
- Preserved bookmark folder hierarchy in portable Netscape HTML exports and filtered non-web/executable URL schemes; added Rust tests for hierarchy, escaping, and URL filtering.
- Made history, cookie metadata, extension inventory, preferences, and portable bookmark exports use unique filenames so a later run does not silently overwrite a user's previous export.
- Reworked history/cookie reporting into concise batch results; dry runs are marked `planned`, empty categories are reported as skipped, and extension inventory is now actually written as a JSON reference export.
- Preferences are copied as a separate reference file only when a supported source file exists; they are not applied to destination browser settings.

### Verification update
- TypeScript typecheck: passed.
- Extension JavaScript syntax checks (`node --check` for background and popup scripts): passed.
- Extension manifest JSON validation: passed.
- ESLint: 0 errors; 6 pre-existing React hook dependency warnings.
- Vitest still cannot start because the archived dependencies are missing the platform-specific Rolldown native binding. Rust compilation/tests remain unverified because the Rust toolchain is not installed here.

## Additional history-import pass

- Added native Chromium-family history import into the destination `History` SQLite database.
- Requires the destination browser to be closed, creates a timestamped database backup, validates supported URL schemes, converts Unix microsecond timestamps to Chromium's 1601 epoch, and skips exact duplicate URL/visit-time pairs.
- Uses a SQLite transaction and recomputes URL visit aggregates after import.
- Added a Rust unit test covering visit insertion and backup creation (not executable in this environment because Cargo/Rust are unavailable).
- History import remains native only for Chromium-family destinations. Firefox and Safari receive a clearly labeled JSON reference export, not a native import.
- Added `MIGRATION_SUPPORT_MATRIX.md` documenting actual support and the security boundaries around password/session transfer, extension installation, and native messaging.

Verification in this environment: TypeScript typecheck passed; ESLint reports 0 errors and 6 existing hook-dependency warnings; extension JavaScript syntax and manifest JSON checks passed. Rust compilation/tests and Vite/Vitest remain unverified because the Rust toolchain is absent and the installed frontend dependency tree lacks its native Rolldown binding.


## Credential staging and native bridge pass

- Added browser-exported password CSV import with URL scheme validation, duplicate filtering, row/size limits, and immediate AES-256-GCM encryption using the existing OS-keychain-backed key.
- Added explicit user-selected CSV export from the encrypted vault so the destination browser's built-in password manager can import it. Exported CSV is plaintext; UI warns users to import it and securely delete it. No direct protected browser password-store extraction or silent password injection is implemented.
- Added a Rust Native Messaging host (`ubs-native-host`) and Chrome/Edge/Brave/Opera registration scripts. The extension can establish a native-messaging connection; the desktop app drains queued metadata events. The bridge rejects credential/session data and is intentionally metadata-only.
- Added Settings UI for password CSV import/export and companion bridge checks.
- Added setup/troubleshooting docs. Browser extension installation remains an explicit user action; silent sideloading is not attempted. Firefox/Safari native host packaging and authenticated-session transfer remain unsupported.
