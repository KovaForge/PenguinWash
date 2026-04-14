# PenguinWash 🐧

> Free, open-source Linux system cleaner. No telemetry. No subscriptions. Just a clean Linux.

PenguinWash scans and safely removes system junk — caches, logs, old package manager files, snap/flatpak orphans, Docker images — freeing disk space without the bloat of paid cleaner apps.

**Inspired by:** [PureMac](https://github.com/momenbasel/PureMac) (macOS), [BleachBit](https://bleachbit.org/) (Linux)

---

## Features

- **Smart Scan** — One-click scan across all categories
- **User Junk** — Thumbnails, browser caches, pip/npm/conda/cargo/go caches
- **System Junk** — `/var/cache`, `/var/log`, journal logs
- **Package Managers** — apt, dnf, pacman, zypper caches
- **Snap/Flatpak** — Old revisions and unused runtimes
- **Docker** — Unused images and volumes
- **Trash** — Empty the Trash bin
- **Scheduled Cleaning** — Configurable via `~/.config/penguinwash.toml`

## Installation

### Binary (Linux x86_64)

Download from [Releases](https://github.com/KovaForge/PenguinWash/releases) and run:
```bash
chmod +x penguinwash
./penguinwash scan
```

### From Source

```bash
git clone https://github.com/KovaForge/PenguinWash.git
cd PenguinWash
cargo build --release
./target/release/penguinwash scan
```

## Usage

```bash
# Scan all categories
./penguinwash scan

# Clean (dry run — always safe)
./penguinwash clean

# Force actual deletion
./penguinwash clean --force

# Verbose output
./penguinwash scan --verbose

# JSON output (for scripting)
./penguinwash scan --json

# Show config
./penguinwash config show
```

## Configuration

Config file: `~/.config/penguinwash.toml`

```toml
large_file_threshold_mb = 100
old_file_threshold_days = 365
journal_vacuum_days = 90
log_vacuum_days = 90
snap_retain_revisions = 2
dry_run = true
auto_clean = false
```

## Categories

| Category | Path | Auto-Clean |
|---|---|---|
| Snap Revisions | `/var/lib/snapd/snaps` | ✅ |
| Flatpak Orphans | `/var/lib/flatpak`, `~/.local/share/flatpak` | ✅ |
| Thumbnails | `~/.cache/thumbnails` | ✅ |
| User Cache | `~/.cache/*` | ✅ |
| APT Cache | `/var/cache/apt` | ✅ |
| DNF Cache | `/var/cache/dnf` | ✅ |
| Pacman Cache | `/var/cache/pacman` | ✅ |
| Journal Logs | `/var/log/journal` | ✅ |
| System Logs | `/var/log` | ❌ (manual) |
| Trash | `~/.local/share/Trash` | ✅ |
| Docker Images | `/var/lib/docker` | ✅ |
| Old Kernels | `/boot` | ❌ (manual) |
| Mail Attachments | `~/.local/share/mail-downloads` | ✅ |

## Safety

- Never deletes system-critical files
- Hardcoded exclusions: `/etc/`, `/.ssh/`, `/var/lib/systemd/`
- Large & old files are **never auto-selected**
- All deletion operations are non-destructive to the OS

## License

MIT
