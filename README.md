# Voxly Desktop

The Windows desktop client for [Voxly](https://voxly-vc.vercel.app), built with **Tauri 2 and Rust**.

## Current foundation

- Loads the existing Voxly web application in a native Windows WebView2 window.
- Reuses the existing Supabase authentication, data, calls, and web backend. There is no second account system or duplicated API.
- Keeps the Voxly window available from the Windows notification area (system tray).
- Closing the main window hides it to the tray; it does not terminate the app.
- The tray menu includes **Open Voxly**, **Open Messages**, **Open Calls**, **Open Groups**, **Reload Voxly**, **Send Test Notification**, **Check for Updates**, and **Quit Voxly**.
- The tray menu and tooltip show the current unread-message count, refreshed by the authenticated Voxly web client.
- Native Windows notifications cover incoming direct messages, group messages, one-to-one calls, and group calls while the desktop app is running in the tray.
- A tray click restores and focuses the main window.
- Enforces a single running instance.
- Uses Tauri's native notification plugin with an explicit permission request; notification settings include a test alert.
- Checks for signed updates in the background at startup; update checks can also be started from the tray menu.
- Installs updates only when they are signed by the updater public key embedded in the app.
- Builds a per-user Windows NSIS installer.

## Version 0.1.5

- **Reload Voxly:** reload the desktop web view directly from the system-tray menu.
- **Send Test Notification:** send a native Windows notification from the tray to quickly verify notification delivery.
- Settings test alerts and realtime desktop alerts use the same native Rust notification handler as the tray test.
- Use the native window visibility state when deciding whether background alerts should appear; tray-hidden notifications no longer depend on WebView document visibility.
- Explicitly handle WebView2 microphone and camera permission requests for the official Voxly production origin.

The desktop integration subscribes to Voxly's existing Supabase realtime events while the app is running. Group unread counts are stored locally on this Windows device because the current group-message schema has no server-side read receipts. The first milestone uses the deployed web UI, so an internet connection is required. The desktop shell is native; the current interface and application backend remain served by the existing Voxly deployment.

## Development requirements

- Windows 10 or Windows 11
- Node.js 22 or newer
- Rust stable (1.90 or newer) with the MSVC Windows target
- Microsoft C++ Build Tools (Desktop development with C++)
- Microsoft Edge WebView2 Runtime

## Run locally

Open PowerShell in the repository root:

```powershell
npm install
npm run dev
```

## Build a local installer

This build creates a normal installer without signed updater artifacts:

```powershell
npm run build
```

The NSIS installer is generated under:

```text
src-tauri/target/release/bundle/nsis/
```

The installer uses current-user install mode, so installing Voxly does not require administrator privileges.

## Publishing signed updates

The updater public key is committed in `src-tauri/tauri.conf.json`. The matching private key and its password must never be committed.

Create these repository Actions secrets under **Settings → Secrets and variables → Actions**:

- `TAURI_SIGNING_PRIVATE_KEY`: the complete contents of your local `voxly.key` file.
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: the password used when generating that key.

The release workflow validates that `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` use the same version. Updating the version on `main` triggers the signed Windows build and publishes a matching GitHub Release with a `latest.json` manifest.

The first updater-enabled installer must be installed manually over earlier pre-updater builds. Existing 0.1.0 installations do not contain the public key and cannot securely self-update. Install the first signed release (0.1.3) once; later releases, including 0.1.5, can update automatically.

## CI

The GitHub Actions workflow in `.github/workflows/windows-build.yml` validates the Windows installer on pushes, pull requests, and manual runs. It intentionally omits signed updater artifacts; signed artifacts are created by the desktop release workflow.

## Security model

The application loads only the official Voxly production origin for remote web content. Tauri IPC is scoped to that origin through a remote capability. The capability grants the Tauri core defaults and native notifications; it does not grant filesystem, shell, or process-plugin access.

Do not place Supabase service-role keys, LiveKit secrets, Firebase Admin credentials, or other backend secrets in this repository or in the desktop client. Authentication and sensitive operations remain on the existing Voxly web/backend services.

## Project layout

```text
.
├── .github/workflows/
│   ├── desktop-release.yml
│   └── windows-build.yml
├── package.json
├── rust-toolchain.toml
└── src-tauri
    ├── capabilities/main-capability.json
    ├── icons/
    ├── src/
    │   ├── lib.rs
    │   └── main.rs
    ├── build.rs
    ├── Cargo.toml
    └── tauri.conf.json
```
