# Voxly Desktop

The Windows desktop client for [Voxly](https://voxly-vc.vercel.app), built with **Tauri 2 and Rust**.

## Current foundation

- Loads the existing Voxly web application in a native Windows WebView2 window.
- Reuses the existing Supabase authentication, data, calls, and web backend. There is no second account system or duplicated API.
- Keeps the Voxly window available from the Windows notification area (system tray).
- Closing the main window hides it to the tray; it does not terminate the app.
- The tray menu includes **Open Voxly** and **Quit Voxly**.
- A tray click restores and focuses the main window.
- Enforces a single running instance.
- Registers Tauri's native notification plugin for the desktop notification integration work.
- Builds a per-user Windows NSIS installer.

The first milestone uses the deployed web UI, so an internet connection is required. The desktop shell is native; the current interface and application backend remain served by the existing Voxly deployment. Later milestones can add desktop-specific UI and integrate native notifications, shortcuts, startup behavior, and updater support without creating another backend.

## Development requirements

- Windows 10 or Windows 11
- Node.js 22 or newer
- Rust stable with the MSVC Windows target
- Microsoft C++ Build Tools (Desktop development with C++)
- Microsoft Edge WebView2 Runtime

## Run locally

Open PowerShell in the repository root:

```powershell
npm install
npm run dev
```

## Build the Windows installer

```powershell
npm run build
```

The NSIS installer is generated under:

```text
src-tauri/target/release/bundle/nsis/
```

The installer uses the current-user install mode, so installing Voxly does not require administrator privileges.

## CI

The GitHub Actions workflow in `.github/workflows/windows-build.yml` builds the NSIS installer on Windows for pushes to `main`, pull requests, and manual runs. The installer is uploaded as a workflow artifact.

## Security model

The application loads only the production Voxly origin for its remote web content. Tauri IPC is scoped to that origin through a remote capability. The capability grants the Tauri core defaults and native notifications; it does not grant filesystem, shell, or process-plugin access.

Do not place Supabase service-role keys, LiveKit secrets, Firebase Admin credentials, or other backend secrets in this repository or in the desktop client. Authentication and sensitive operations remain on the existing Voxly web/backend services.

## Project layout

```text
.
├── .github/workflows/windows-build.yml
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
