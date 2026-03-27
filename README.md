# FluxEnv

![FluxEnv app icon](apps/desktop/src-tauri/icons/source/fluxenv-icon.svg)

FluxEnv is a cross-platform desktop tool for managing and switching project environments.

It focuses on:
- Loading and merging `.env` files by profile.
- Managing sensitive values securely with system keychain integrations.
- Running project commands with the selected environment injected.

## Project Status

This project is in early setup stage.

## Goals (MVP)

- Register local projects.
- Detect and load `.env`, `.env.local`, `.env.{profile}`.
- Support profile switching (dev/staging/prod/custom).
- Inject environment variables when running commands.
- Store sensitive values in OS keychain (instead of plaintext files).

## Directory Structure

```text
apps/
  desktop/            # Tauri desktop application (UI + command bridge)
    src/
    src-tauri/

crates/
  fluxenv-core/       # env parsing, merge rules, validation
  fluxenv-secrets/    # secure storage adapters (keychain)
  fluxenv-runner/     # command runner with env injection
  fluxenv-models/     # shared domain models

packages/
  ui/                 # shared UI components (optional)

docs/
  adr/                # architecture decision records
```

## Suggested Next Steps

1. Add workspace configuration for Rust crates.
2. Scaffold the Tauri desktop app.
3. Define env merge precedence in ADR-001.
4. Implement and test minimal `fluxenv-core` parsing/merge logic.

## API key storage (desktop)

The Tauri app stores provider API keys in the **OS credential store** (macOS Keychain, Windows Credential Manager, Linux Secret Service) via the Rust [`keyring`](https://crates.io/crates/keyring) crate and service name `com.fluxenv.desktop` (same as `identifier` in `tauri.conf.json`). In-memory `ProviderConfig` keeps **empty** values for secrets; plaintext is only read from the keychain when building the effective environment (e.g. preview). Use **Clear secret** in the UI to remove a stored entry.

## Auto-update (GitHub Releases)

The desktop app uses [Tauri Updater](https://v2.tauri.app/plugin/updater/) with artifacts published to **GitHub Releases**.

### One-time setup

1. **Signing keys**  
   Generate a keypair (keep the private key secret; add only the public key string to `apps/desktop/src-tauri/tauri.conf.json` → `plugins.updater.pubkey`):

   ```bash
   cd apps/desktop && npm run tauri signer generate -w ~/.tauri/fluxenv.key -p ""
   ```

   Copy `~/.tauri/fluxenv.key.pub` content into `pubkey` in `tauri.conf.json`.  
   Store the **private** key only in CI: GitHub repository secret `TAURI_SIGNING_PRIVATE_KEY` (file contents). Optional: `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` if you set a password.

2. **Update endpoint**  
   `plugins.updater.endpoints` in `tauri.conf.json` points at this repo’s `latest.json`:

   `https://github.com/MouXiaoJun/FluxEnv/releases/latest/download/latest.json`

   If you fork the project, change the URL to your `owner/repo`.

3. **CI permissions**  
   In the GitHub repo: **Settings → Actions → General → Workflow permissions** → enable **Read and write** for `GITHUB_TOKEN` (needed to create releases).

### Publishing a release

1. Bump `version` in `apps/desktop/src-tauri/tauri.conf.json` (and `package.json` / `Cargo.toml` if you keep them in sync).
2. Commit and push a tag: `git tag v0.1.0 && git push origin v0.1.0`.
3. Workflow **Release desktop** (`.github/workflows/release.yml`) builds on macOS (x64 + arm64), Linux, Windows, signs artifacts, and uploads a **draft** release. Review and publish the release on GitHub.

After publish, the in-app **Check for updates** button will find updates when `latest.json` on the release matches the configured endpoint and version is newer than the running app.
