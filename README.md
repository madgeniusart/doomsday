# Universal Browser Sync

A cross-browser profile migration and synchronization desktop application built with Tauri 2, Rust, React, TypeScript, and SQLite.

---

## Features

| Feature | Status | Notes |
|---|---|---|
| Browser detection (Chrome, Firefox, Edge, Brave, Opera) | ✅ Full | Platform-native path detection |
| Safari detection | ✅ macOS only | Requires Full Disk Access |
| Bookmark migration | ✅ Full | Netscape HTML export (importable by all browsers) |
| History migration | ✅ Read + export | JSON export; direct DB injection is browser-specific |
| Cookie inspection & metadata | ✅ Full | Auth cookies flagged and excluded by default |
| Extension inventory | ✅ Full | Cross-browser compatibility mapping |
| Preferences export | ✅ Reference | Format differs per browser; manual import required |
| AES-256-GCM backup encryption | ✅ Full | Key in OS keychain |
| Multi-profile support | ✅ Full | Chromium `Local State`, Firefox `profiles.ini` |
| Browser extension (Chrome/Edge/Brave) | ✅ Full | Cookie metadata, storage snapshot |
| Migration history | ✅ Full | SQLite-persisted per-run records |
| Dry run mode | ✅ Full | Analyse without writing files |
| Search engine migration | 🚧 Planned | Format is browser-specific |
| Direct history DB injection | 🚧 Planned | Requires matching browser SQLite schema |
| Safari cookies | ❌ Platform-limited | Requires Keychain entitlement not grantable without Apple sign-off |
| Password migration | ❌ By design | Passwords are not migrated; use a dedicated password manager |

---

## Architecture

```
universal-browser-sync/
├── src/                          # React + TypeScript frontend
│   ├── App.tsx                   # Root layout, view routing
│   ├── components/
│   │   ├── browser/              # BrowserCard, Dashboard
│   │   ├── migration/            # Wizard, CategorySelector, ProgressPanel, Report, History
│   │   ├── settings/             # BackupPanel, SettingsPanel
│   │   └── ui/                   # Sidebar, ConsentModal, BrowserIcon, primitives
│   ├── services/tauri.ts         # Typed invoke() wrappers for all Rust commands
│   ├── store/appStore.ts         # Zustand global state with immer middleware
│   ├── types/index.ts            # TypeScript types mirroring Rust structs
│   └── utils/index.ts            # Formatting helpers
│
├── src-tauri/                    # Rust Tauri backend
│   ├── src/
│   │   ├── lib.rs                # App setup, plugin registration, command routing
│   │   ├── main.rs               # Binary entry point
│   │   ├── adapters/             # Browser adapters (trait + 6 implementations)
│   │   │   ├── mod.rs            # BrowserAdapter trait, shared types
│   │   │   ├── chromium_base.rs  # Shared Chromium logic (profiles, bookmarks, history, cookies, extensions)
│   │   │   ├── chrome.rs         # Chrome-specific paths
│   │   │   ├── firefox.rs        # Firefox profiles.ini, places.sqlite, cookies.sqlite
│   │   │   ├── edge.rs           # Edge paths → chromium_base
│   │   │   ├── brave.rs          # Brave paths → chromium_base
│   │   │   ├── opera.rs          # Opera flat + nested profile detection
│   │   │   ├── safari.rs         # macOS plutil plist conversion, History.db
│   │   │   └── detector.rs       # Cross-platform BrowserDetector (PATH/bundles/registry)
│   │   ├── commands/             # Tauri command handlers
│   │   │   ├── browser.rs        # detect_browsers, get_browser_profiles, verify_profile_access
│   │   │   ├── bookmarks.rs      # read_bookmarks, export_bookmarks (HTML/JSON)
│   │   │   ├── history.rs        # read_history
│   │   │   ├── cookies.rs        # read_cookies, export_cookies
│   │   │   ├── extensions.rs     # read_extensions, get_compatible_extensions
│   │   │   ├── migration.rs      # start_migration (async), status, cancel, history, report
│   │   │   ├── backup.rs         # create_backup (tar+AES/gzip), restore, list, delete
│   │   │   └── settings.rs       # get_settings, save_settings
│   │   ├── database/mod.rs       # AppDatabase, WAL mode, typed CRUD, migration runner
│   │   ├── encryption/mod.rs     # AES-256-GCM + OS keyring + PBKDF2
│   │   └── migration/mod.rs      # CompatibilityReport analyser
│   ├── migrations/               # SQL migration files (included at compile time)
│   │   ├── 001_initial.sql
│   │   ├── 002_migration_history.sql
│   │   └── 003_backups.sql
│   ├── Cargo.toml
│   ├── build.rs
│   └── tauri.conf.json
│
├── browser-extension/            # WebExtension (MV3) companion
│   ├── manifest.json
│   └── src/
│       ├── background.js         # Service worker: cookie metadata, storage inspection
│       ├── popup.html/.js        # Per-tab cookie summary UI
│       └── content.js            # Content script for localStorage snapshots
│
└── tests/
    ├── unit/
    │   ├── utils.test.ts
    │   └── store.test.ts
    └── integration/
        └── components.test.tsx
```

---

## Prerequisites

### All platforms
- **Node.js** ≥ 18  
- **npm** ≥ 9  
- **Rust** ≥ 1.77 ([rustup.rs](https://rustup.rs))

### Linux (Ubuntu / Debian)
```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

### macOS
```bash
xcode-select --install
```

### Windows
1. Install [Visual Studio Build Tools 2022](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the "Desktop development with C++" workload  
2. Install [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (bundled with Windows 11; required on Windows 10)

---

## Installation & Build

### 1 — Clone and install dependencies

```bash
git clone https://github.com/your-org/universal-browser-sync
cd universal-browser-sync
npm install
```

### 2 — Development mode

```bash
npm run tauri:dev
```

This starts the Vite dev server on port 1420 and launches the Tauri window with hot-reload for the frontend. Rust changes trigger a Rust rebuild.

### 3 — Production build

```bash
npm run tauri:build
```

Produces installers in `src-tauri/target/release/bundle/`:
- **Linux:** `.deb`, `.AppImage`
- **macOS:** `.dmg`, `.app`
- **Windows:** `.msi`, NSIS installer

### 4 — Run tests

```bash
# Frontend unit + integration tests
npm test

# Watch mode
npm run test:watch

# With UI
npm run test:ui

# Rust unit tests (requires Rust installed)
cd src-tauri && cargo test
```

### 5 — Type checking & linting

```bash
npm run typecheck
npm run lint
```

---

## Browser Extension Installation

### Chrome / Edge / Brave
1. Open `chrome://extensions` (or `edge://extensions`, `brave://extensions`)
2. Enable **Developer mode**
3. Click **Load unpacked** → select the `browser-extension/` directory

### Firefox
Firefox uses Manifest V2 for permanent extensions. For development:
1. Open `about:debugging#/runtime/this-firefox`
2. Click **Load Temporary Add-on** → select `browser-extension/manifest.json`

> The extension is optional. The desktop app reads browser data directly without it. The extension adds enhanced cookie and storage inspection for the active tab.

---

## Platform-Specific Limitations

### macOS — Safari

Safari's data is protected by macOS system entitlements:

| Data Type | Access | Requirement |
|---|---|---|
| Bookmarks | ✅ | Full Disk Access |
| History | ✅ | Full Disk Access |
| Cookies | ❌ | Requires `com.apple.private.tcc.access` — only available to Apple-signed apps |
| Extensions | ❌ | Safari extensions are App Store apps; no file-level access |

**To grant Full Disk Access:**
1. System Settings → Privacy & Security → Full Disk Access
2. Add Universal Browser Sync

### Linux — Chrome/Chromium cookie decryption

On Linux, Chrome encrypts cookie values with a key derived from the user's keyring password (libsecret). This application reads cookie **metadata** (domain, name, flags, expiry) without decrypting values, which requires no special permissions.

### Windows — Chrome cookie values

Chrome on Windows uses DPAPI to encrypt cookie values. Reading cookie values (not done here by design) would require running as the same user who owns the profile. Metadata is always accessible.

### General

- **Browser must be closed** (or its profile not actively locked) for SQLite databases (History, Cookies) to be copied. The app works around this by copying the database to a temp file before reading.
- **Source data is never written to or modified.** The application is read-only with respect to source browser profiles.
- **History direct injection** is not implemented: Chromium's History database has internal page ID references and visit chains that are non-trivial to replay into another profile without causing corruption. History is exported as a JSON file for reference.
- **Passwords are intentionally excluded**: Browser password databases are encrypted with OS-level keys and should be migrated via a dedicated password manager (Bitwarden, 1Password, etc.).

---

## Security Model

1. **Local-first**: No network requests. All processing is on-device.
2. **Explicit consent**: A consent modal must be accepted before any browser data is accessed.
3. **Read-only source**: The source profile directory is never opened for writing.
4. **Auth cookie filtering**: Cookies matching known session/auth name patterns (`session`, `auth`, `token`, `jwt`, `sid`, etc.) are flagged and excluded from migration exports by default.
5. **AES-256-GCM encryption**: Backups use AES-256-GCM with a 96-bit random nonce per backup. The 256-bit key is stored in the OS keychain:
   - macOS: Keychain Services (`keyring` crate → `security-framework`)
   - Windows: DPAPI via Credential Manager
   - Linux: libsecret / GNOME Keyring / KWallet
6. **PBKDF2-HMAC-SHA256**: Available for password-derived key scenarios (100,000 iterations).
7. **Content Security Policy**: The Tauri webview enforces a strict CSP blocking all external connections.

---

## Database Schema

```sql
-- Settings key-value store
CREATE TABLE settings (
    key        TEXT PRIMARY KEY,
    value      TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Per-migration-run record
CREATE TABLE migration_runs (
    id              TEXT PRIMARY KEY,
    source_browser  TEXT NOT NULL,
    source_profile  TEXT NOT NULL,
    dest_browser    TEXT NOT NULL,
    dest_profile    TEXT NOT NULL,
    data_types      TEXT NOT NULL,   -- JSON array
    status          TEXT NOT NULL,   -- pending|running|completed|failed|cancelled
    started_at      TEXT NOT NULL,
    completed_at    TEXT,
    items_migrated  INTEGER NOT NULL DEFAULT 0,
    items_failed    INTEGER NOT NULL DEFAULT 0,
    error_message   TEXT
);

-- Per-item migration result
CREATE TABLE migration_items (
    id          TEXT PRIMARY KEY,
    run_id      TEXT NOT NULL REFERENCES migration_runs(id) ON DELETE CASCADE,
    category    TEXT NOT NULL,
    item_key    TEXT NOT NULL,
    item_label  TEXT NOT NULL,
    status      TEXT NOT NULL,   -- migrated|failed|skipped|incompatible
    message     TEXT,
    migrated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Backup manifest
CREATE TABLE backups (
    id          TEXT PRIMARY KEY,
    browser     TEXT NOT NULL,
    profile     TEXT NOT NULL,
    path        TEXT NOT NULL,
    size_bytes  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    encrypted   INTEGER NOT NULL DEFAULT 1
);
```

> **Note on app icons:** The icons shipped in `src-tauri/icons/` and
> `browser-extension/icons/` are programmatically generated placeholders
> (a simple blue rounded-square mark). They are valid, correctly-formatted
> files — real PNGs, a genuine multi-resolution Windows `.ico`, and a
> properly-encoded Apple `.icns` container — so `tauri build` and browser
> extension loading both work out of the box. Swap them for your own
> artwork at the same paths/sizes before shipping publicly.

> **Note on Tauri 2 capabilities:** Permissions live in
> `src-tauri/capabilities/default.json`, not inline in `tauri.conf.json`
> (that was the Tauri 1 format). The capability file grants the `dialog`,
> `notification`, and `shell:allow-open` permissions the frontend uses,
> plus `fs` read/write scopes for completeness — though in practice this
> app's actual profile reading happens inside Rust commands via plain
> `std::fs`, which isn't gated by the capability system at all. The `fs`
> scope only matters if the frontend ever calls `@tauri-apps/plugin-fs`
> directly (it currently doesn't).

The database lives at:
- **Linux:** `~/.local/share/universal-browser-sync/universal_browser_sync.db`
- **macOS:** `~/Library/Application Support/com.universal-browser-sync.app/universal_browser_sync.db`
- **Windows:** `%APPDATA%\universal-browser-sync\universal_browser_sync.db`

---

## Adding a New Browser Adapter

1. Create `src-tauri/src/adapters/mybrowser.rs`
2. Implement the `BrowserAdapter` trait:
   ```rust
   pub struct MyBrowserAdapter { user_data_dir: PathBuf }
   
   impl BrowserAdapter for MyBrowserAdapter {
       fn kind(&self) -> BrowserKind { BrowserKind::MyBrowser }
       fn enumerate_profiles(&self) -> Result<Vec<BrowserProfile>> { ... }
       fn read_bookmarks(&self, profile_path: &PathBuf) -> Result<Vec<Bookmark>> { ... }
       fn read_history(&self, profile_path: &PathBuf, max_days: u32) -> Result<Vec<HistoryEntry>> { ... }
       fn read_cookies(&self, profile_path: &PathBuf) -> Result<Vec<CookieInfo>> { ... }
       fn read_extensions(&self, profile_path: &PathBuf) -> Result<Vec<ExtensionInfo>> { ... }
       fn supported_categories(&self) -> Vec<DataCategory> { ... }
   }
   ```
3. Add `BrowserKind::MyBrowser` to the enum in `adapters/mod.rs`
4. Register in `adapters/detector.rs`
5. Wire up in the command match arms in `commands/browser.rs`, `commands/bookmarks.rs`, etc.
6. Add `BROWSER_META` entry in `src/types/index.ts`

---

## Technology Stack

| Layer | Technology |
|---|---|
| Desktop framework | Tauri 2 |
| Backend language | Rust 1.77+ |
| Frontend framework | React 18 + TypeScript |
| State management | Zustand + Immer |
| Styling | Tailwind CSS 3 |
| Database | SQLite via rusqlite (bundled) |
| Encryption | AES-256-GCM (aes-gcm crate) |
| Key storage | OS keychain (keyring crate) |
| Async runtime | Tokio |
| Build tool | Vite 5 |
| Test runner | Vitest + Testing Library |
| Archive format | tar + flate2 (gzip) |

---

## License

MIT
