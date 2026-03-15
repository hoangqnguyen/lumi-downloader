# Lumi Downloader — AI Instructions

## Project

Lumi Downloader is a Tauri v2 desktop app (Rust + TypeScript/Vite) that downloads YouTube videos and audio. It bundles yt-dlp, ffmpeg, and ffprobe as sidecar binaries.

- **Repo**: https://github.com/hoangqnguyen/lumi-downloader
- **Landing page**: `docs/index.html` (static, hosted via GitHub Pages)
- **Platforms**: macOS (aarch64 + x86_64), Windows (x86_64), Linux (x86_64)

## Release Process

### 1. Version bump

Update the version in **all four files** (they must stay in sync):

| File | Field |
|------|-------|
| `package.json` | `"version"` |
| `src-tauri/Cargo.toml` | `version` |
| `src-tauri/tauri.conf.json` | `"version"` |

Then regenerate lock files:
```bash
npm install --package-lock-only
cargo generate-lockfile --manifest-path src-tauri/Cargo.toml
```

### 2. Versioning convention

- **Patch** (x.y.Z): bug fixes, CI changes, landing page updates, config changes
- **Minor** (x.Y.0): new features, new platform support
- **Major** (X.0.0): breaking changes, major rewrites

### 3. Commit

- Stage specific files — never use `git add -A` or `git add .`
- Commit message format: `v{version}: short summary`
- Example: `v1.0.3: fix macOS codesigning for yt-dlp, fix Linux deb/rpm ffmpeg conflict`

### 4. Tag and push

```bash
git tag vX.Y.Z
git push origin master --tags
```

### 5. CI workflow

Pushing a `v*` tag triggers `.github/workflows/release.yml`:
- Builds for macOS (aarch64 + x86_64), Linux (x86_64), Windows (x86_64)
- macOS: codesigns bundled binaries, then Tauri signs + notarizes the app (requires `APPLE_*` secrets)
- Linux: uses dummy ffmpeg/ffprobe stubs; system ffmpeg is declared as deb/rpm dependency
- Windows: builds via `vcvarsall.bat`, uploads artifacts manually via `gh release upload`
- Creates a **draft release** on GitHub

### 6. After CI completes

- Go to GitHub Releases → find the draft
- Set title: `Lumi Downloader vX.Y.Z`
- Write concise release notes
- Publish

### 7. Rules

- Never amend a published tag — create a new patch version
- Never push without testing locally if code changes are involved
- Never delete a release — create the next patch version instead
- The landing page (`docs/index.html`) auto-fetches latest release assets via GitHub API — no manual URL updates needed

## Architecture Notes

- `src-tauri/src/ytdlp/runner.rs` — yt-dlp command runner, handles ffmpeg discovery
- `src-tauri/src/commands/` — Tauri IPC commands
- External binaries live in `src-tauri/binaries/` (gitignored, downloaded in CI)
- ffmpeg resolution order: bundled sidecar dir → PATH → well-known locations (`/opt/homebrew/bin`, `/usr/local/bin`, `/usr/bin`)

## GitHub Secrets (for CI)

`APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`
