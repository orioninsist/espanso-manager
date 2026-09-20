# Espanso Manager

A small native desktop manager for Espanso snippets, built with Tauri, Rust, and TypeScript.

## Scope

Espanso Manager intentionally does only a few things:

- list and search snippets
- add, edit, and delete snippets
- read and write `~/.config/espanso/match/base.yml`
- preserve symlink-based dotfiles setups by resolving the real target before writing
- prevent duplicate triggers
- write changes atomically
- commit and push only the Espanso snippet file when you press **GitHub’a Kaydet**

There is no database, browser tab, localhost service, or background daemon.

## Development

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

## Git behavior

Editing snippets only changes the local YAML file.

The **GitHub’a Kaydet** button performs:

```text
git add <base.yml>
git commit -m "Update Espanso snippets"
git push origin HEAD
```

Only the snippet file is staged and committed.
