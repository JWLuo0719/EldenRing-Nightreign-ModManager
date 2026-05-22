# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

夜临 Mod 管理器 — a Tauri v2 + React 19 desktop app for managing mods for Elden Ring: Nightreign. The app wraps **ME3 (Mod Engine 3)** CLI tools to load mods into the game. UI is in Chinese.

## Commands

All commands run from the `app/` directory:

```bash
# Full Tauri dev (starts Vite + compiles Rust + launches window)
cd app && npx tauri dev

# Frontend only (Vite dev server, no Rust)
cd app && npm run dev

# Type-check + production build
cd app && npm run build

# Lint
cd app && npm run lint

# Rust-only check (from app/src-tauri/)
cd app/src-tauri && cargo check

# Production package
cd app && npx tauri build
```

First `tauri dev` run compiles ~400 Rust crates and takes several minutes. Subsequent runs use cached artifacts.

## Architecture

```
app/
  src-tauri/          # Rust backend (Tauri commands + plugins)
    src/
      lib.rs          # Registers plugins + all 13 invoke handlers
      commands/
        mod_manager.rs # Game path, ME3 path, mod scan/toggle/delete/install, launch
        profile.rs    # Profile CRUD, activate/deactivate
  src/                # React frontend
    App.tsx           # Root: orchestrates setup flow ↔ main UI
    components/       # Titlebar, Sidebar, ModList, ModCard, SetupGuide
    types/mod.ts      # Shared TypeScript interfaces
```

### Data flow

1. `App.tsx` calls Rust via `invoke()` from `@tauri-apps/api/core` — no Tauri events, pure request/response.
2. Rust commands read/write the filesystem directly (no database).
3. Config stored at `dirs::config_dir()/nightreign-mod-manager/config.json`.
4. Profiles stored as individual JSON files in `…/profiles/`, active profile tracked in `active_profile.txt`.

### Mod detection

`scan_mods` scans `{game_path}/mods/` for subdirectories, parses `.me3` files (TOML format) inside each to extract metadata. Mod type is `native` (has `[[natives]]`) or `package` (has `[[packages]]`). Toggle is a folder rename to/from `.disabled`.

### ME3 integration

The app invokes `me3-launcher.exe` from `{me3_path}/bin/`. The `.me3` profile format uses `profileVersion`, `[[supports]]` (game), `[[packages]]` (resource folders), and `[[natives]]` (DLL paths with optional `load_early`/`load_before`).

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Framework | Tauri 2.x |
| Frontend | React 19, TypeScript 6, Tailwind CSS v4 (CSS-based config via `@theme` in `index.css`, no tailwind.config file) |
| Build | Vite 8, `@tailwindcss/vite` plugin |
| Backend | Rust (edition 2021), `serde`/`serde_json`, `toml`, `dirs` |
| Window | Custom titlebar (decorations: false), dark theme |
