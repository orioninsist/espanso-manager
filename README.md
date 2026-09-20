# Espanso Manager

Native desktop manager for a small, focused Espanso workflow.

## Purpose

Espanso Manager is intentionally narrow:

- list and search snippets
- add, edit, and delete snippets
- edit `~/.config/espanso/match/base.yml`
- preserve symlink-based dotfiles setups
- prevent duplicate triggers
- write YAML changes atomically
- commit and push only the Espanso snippet file with one button

There is no database, browser tab, localhost service, or background daemon in normal use.

## Daily use

The installed release binary lives at:

```text
~/.local/bin/espanso-manager
```

The dotfiles Niri binding launches it with:

```text
Super+F9
```

After reboot, nothing needs to be started manually. Niri launches the native binary when the shortcut is pressed.

Snippet edits are local until **GitHub’a Kaydet** is pressed. That button commits only `.config/espanso/match/base.yml` and pushes it to the configured Git remote.

## Development

```bash
npm install
npm run tauri dev
```

Development mode uses Vite on port 1420. This is not used by the installed release binary.

## Release build

```bash
npm run build
cargo build --release --manifest-path src-tauri/Cargo.toml
install -Dm755 src-tauri/target/release/espanso-manager ~/.local/bin/espanso-manager
```

The release binary embeds the built frontend from `dist/` and does not require npm, Vite, or a localhost server at runtime.

## Git behavior

The save button performs the equivalent of:

```text
git add -- <base.yml>
git commit --only -m "Update Espanso snippets" -- <base.yml>
git push origin HEAD
```

Using `--only` ensures unrelated staged or modified dotfiles are not included in the snippet commit.
