# ZenSync

Sync your **Zen Browser** configuration across multiple machines using a Git repository as the backbone.

Written in Rust. Linux only for now.

> **Status: early MVP (v0.0.1).** It works, but it's manual — you have to run `zensync push` and `zensync pull` yourself.

---

## What problem this solves

Firefox Sync (which Zen uses under the hood) only covers part of the browser state:

- Bookmarks
- Passwords
- History
- Installed extensions
- Open tabs

**It doesn't cover the customizations that matter most on Zen:**

- Workspaces and containers
- Arc-style CSS (`userChrome.css`, `userContent.css`)
- Zen mods (`zen-themes.json`)
- `about:config` settings (`user.js`, `prefs.js`)
- Custom keyboard shortcuts
- Toolbar layout (`xulstore.json`)
- Essential tabs and pinned tabs

ZenSync covers all of that. You can use it alongside Firefox Sync — or instead of it, if you'd rather sync passwords/bookmarks through Git too (see [What gets synced](#what-gets-synced)).

---

## What it does today

- 3 manual commands: `init`, `push`, `pull`
- Hardcoded allowlist — syncs essentially the entire profile (~40 MB typical)
- Automatic detection of the active profile (Flatpak and native install)
- Automatic backup before every `pull`
- Defensive lock: `pull` aborts if Zen is running
- Absolute path sanitization (`/home/user/` → `{{HOME}}`)

---

## What gets synced

The full allowlist lives in [`src/files.rs`](src/files.rs). Summary:

**User preferences:**
- `user.js`, `prefs.js`, `xulstore.json`, `compatibility.ini`
- `chrome/` recursive (CSS + mods)
- `zen-themes.json`, `zen-keyboard-shortcuts.json`, `zen-sessions.jsonlz4`, `zen-live-folders.jsonlz4`
- `containers.json`, `handlers.json`, `search.json.mozlz4`
- `extensions.json`, `extension-preferences.json`, `extension-settings.json`, `extension-store/`

**Bookmarks, history, favicons:**
- `places.sqlite`, `favicons.sqlite`, `bookmarkbackups/`

**Passwords, cookies, forms, permissions:**
- `logins.json`, `key4.db`, `key3.db`, `cert9.db`
- `cookies.sqlite`, `formhistory.sqlite`, `permissions.sqlite`, `content-prefs.sqlite`

**Session state:**
- `sessionstore.jsonlz4`, `sessionstore-backups/`, `sessionCheckpoints.json`, `zen-sessions-backup/`

**Network/security state:**
- `AlternateServices.bin`, `SiteSecurityServiceState.bin`, `security_state/`, `pkcs11.txt`

**Other:**
- `storage/` (IndexedDB — web apps keep their login state and data here)
- `settings/`, `broadcast-listeners.json`, `serviceworker.txt`, `shield-preference-experiments.json`

## What does NOT get synced

The denylist in [`src/files.rs`](src/files.rs) blocks **transient state** that either regenerates automatically or corrupts the target if copied:

- Lock files: `.parentlock`, `parent.lock`, `lock`, any `*.lock` / `*.lck`
- SQLite write-ahead logs: any `*-wal`, `*-shm`, `*-journal`

These are also **deliberately excluded** (not on the allowlist):

- `cache2/`, `startupCache/`, `thumbnails/`, `shader-cache/` — caches (regenerate)
- `weave/` — Firefox Sync internal state (syncing this causes conflicts)
- `datareporting/`, `crashes/`, `minidumps/`, `logs/` — telemetry / diagnostics
- `gmp-gmpopenh264/`, `gmp-widevinecdm/` — DRM codecs (installed on demand, ~20 MB)

---

## Important warnings

### Passwords in Git

By default the MVP syncs `logins.json` and `key4.db` — meaning **your passwords go into the Git repo**. Even if the repo is private, this means that if your GitHub account or SSH key is compromised, your passwords leak.

If you'd rather leave passwords to Firefox Sync (safer), remove those entries from the allowlist in `src/files.rs` and rebuild.

### SQLite + Zen open

Files like `places.sqlite`, `cookies.sqlite`, etc. use SQLite in WAL mode. If you run `zensync push` while Zen is open, the snapshot can be inconsistent (captured mid-write).

**Recommended: close Zen before your first `push`.** Subsequent pushes with Zen open rarely cause problems, but there's no guarantee.

### Pull conflicts

`zensync pull` uses `git pull --ff-only`. If you edited configs on two machines without syncing between them, the pull will fail. For now, resolve manually:

```bash
cd ~/.local/share/zensync/repo
git pull --rebase    # or your preferred strategy
```

---

## Requirements

- **Linux** (Fedora, Ubuntu, Arch — any distro; tested on Fedora 43)
- **Rust** (any recent stable; tested with 1.95)
- **Git** available in `PATH`
- **Zen Browser** installed (native or Flatpak — both autodetected)
- A **GitHub account** (or any Git host) with an empty private repository to store your configs
- An **SSH key** registered on GitHub (or HTTPS with a PAT)

---

## Installation

```bash
# Install Rust if you don't have it
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
source ~/.cargo/env

# Clone the project
git clone https://github.com/DevPorfirio/ZenSync.git
cd ZenSync

# Build and install the binary to ~/.cargo/bin/zensync
cargo install --path .

# Confirm
zensync --help
```

---

## Initial setup

### 1. Create the config repository

Create an **empty private** repository on GitHub (or GitLab, Codeberg, etc.):

- Go to `github.com/new`
- Name: `zen-configs` (or whatever you prefer)
- Visibility: **Private**
- **Do not** check "Add README", "Add .gitignore", or a license — it must be 100% empty

Note the SSH URL, something like: `git@github.com:YOUR_USER/zen-configs.git`.

### 2. Make sure SSH works

```bash
ssh -T git@github.com
# Expected: "Hi USER! You've successfully authenticated..."
```

If you get `Permission denied`, you need to add an SSH key to your GitHub account. Generate one if you don't have any:

```bash
ssh-keygen -t ed25519 -C "you@example.com"
cat ~/.ssh/id_ed25519.pub
```

Paste the output into `github.com/settings/ssh/new`.

### 3. Initialize ZenSync

```bash
zensync init git@github.com:YOUR_USER/zen-configs.git
```

This clones the (empty) repo into `~/.local/share/zensync/repo`.

### 4. First push (with Zen closed)

```bash
# Close Zen first (important because of the SQLite WAL)
pgrep -af zen

zensync push
```

Done — your configs are on GitHub.

---

## Usage

```bash
# Send local changes to Git
zensync push

# Pull changes from Git and apply (Zen must be closed)
zensync pull

# Show help
zensync --help
```

### On a second machine

Repeat [Installation](#installation) and [Initial setup](#initial-setup), but in step 4 use `pull` instead of `push`:

```bash
zensync init git@github.com:YOUR_USER/zen-configs.git
# close Zen
zensync pull
```

`pull` creates a full backup of the current profile at `~/.local/share/zensync/backups/<timestamp>/` before overwriting anything, so you can revert manually if something goes wrong.

### Day-to-day flow

- Changed something on machine A → `zensync push`
- Before opening Zen on machine B → `zensync pull` (with Zen closed) → open Zen

---

## Project structure

```
ZenSync/
├── Cargo.toml
├── plano.md           # planning notes (Portuguese)
├── README.md          # you are here
└── src/
    ├── main.rs        # entry point, subcommand routing
    ├── cli.rs         # clap definitions (subcommands and flags)
    ├── files.rs       # allowlist and denylist
    ├── state.rs       # paths for the state dir (~/.local/share/zensync)
    ├── profile.rs     # detects the active profile via profiles.ini
    ├── lifecycle.rs   # checks whether Zen is running (sysinfo)
    ├── sanitize.rs    # $HOME ↔ {{HOME}} transformation
    ├── git.rs         # shell-out to git
    ├── sync.rs        # copy engine (text/binary/recursive)
    ├── init.rs        # init command
    ├── push.rs        # push command
    └── pull.rs        # pull command
```

---

## Where things live on disk

| Path | Purpose |
|---|---|
| `~/.local/share/zensync/repo/` | Local clone of the configs repo |
| `~/.local/share/zensync/backups/<timestamp>/` | Automatic backups before every `pull` |
| `~/.zen/` or `~/.var/app/app.zen_browser.zen/.zen/` | Zen profile (we don't touch it directly — we resolve it via `profiles.ini`) |

---

## License

MIT.
