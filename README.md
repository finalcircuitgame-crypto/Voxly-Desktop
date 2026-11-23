# Voxly Desktop

The Windows desktop client for [Voxly](https://voxly-vc.vercel.app), built with **Tauri 2 and Rust**.

## Current foundation

- Loads the existing Voxly web application in a native Windows WebView2 window.
- Reuses the existing Supabase authentication, data, calls, and web backend. There is no second account system or duplicated API.
- Keeps the Voxly window available from the Windows notification area (system tray).
- Closing the main window hides it to the tray; it does not terminate the app.
- The tray menu includes **Open Voxly**, **Check for Updates**, and **Quit Voxly**.
- A tray click restores and focuses the main window.
- Enforces a single running instance.
- Registers Tauri's native notification plugin for desktop notification integration.
- Checks for signed updates in the background at startup; update checks can also be started from the tray menu.
- Installs updates only when they are signed by the updater public key embedded in the app.
- Builds a per-user Windows NSIS installer.

The first milestone uses the deployed web UI, so an internet connection is required. The desktop shell is native; the current interface and application backend remain served by the existing Voxly deployment.

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

Create a version tag matching the version in both `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` (for example, `v0.1.1`). The `Desktop release` workflow builds the signed NSIS updater bundle and publishes it with a `latest.json` manifest to GitHub Releases.

The first updater-enabled installer must be installed manually over earlier pre-updater builds. Existing 0.1.0 installations do not contain the public key and cannot securely self-update. Install the first signed release (0.1.1) once; later releases can update automatically.

## CI

The GitHub Actions workflow in `.github/workflows/windows-build.yml` validates the Windows installer on pushes, pull requests, and manual runs. It intentionally omits signed updater artifacts; signed artifacts are created only by the tagged release workflow.

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
    ├── tauri.ci.conf.json
    └── tauri.conf.json
```
