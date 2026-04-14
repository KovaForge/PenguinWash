# PenguinWash 🐧

> Free, open-source Linux system cleaner. No telemetry. No subscriptions. Just a clean Linux.

PenguinWash scans and safely removes system junk — caches, logs, old package manager files, snap/flatpak orphans, Docker images — freeing disk space without the bloat of paid cleaner apps. Includes a remote diagnosis tool for scanning any Linux machine and sharing results.

**Inspired by:** [PureMac](https://github.com/momenbasel/PureMac) (macOS), [BleachBit](https://bleachbit.org/) (Linux)

---

## Features

- **Smart Scan** — Scans all categories for cleanable junk
- **Remote Diagnosis** — Scan any Linux machine, share results as JSON
- **User Junk** — Thumbnails, browser caches, pip/npm/conda/cargo/go caches
- **System Junk** — `/var/cache`, `/var/log`, journal logs
- **Package Managers** — apt, dnf, pacman, zypper caches
- **Snap/Flatpak** — Old revisions and unused runtimes
- **Docker** — Unused images and volumes
- **Trash** — Empty the Trash bin
- **GUI Mode** — Optional egui-based graphical interface (pure Rust, no GTK needed)
- **Scheduled Cleaning** — Configurable via `~/.config/penguinwash.toml`

---

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

### GUI Build

```bash
cargo build --features gui --release
./target/release/penguinwash --gui
```

---

## Usage

### Scan (all categories)

```bash
./penguinwash scan
./penguinwash scan --verbose    # detailed logging
./penguinwash scan --json      # machine-readable output
```

### Large File Scan (remote diagnosis)

Scan for large files on any Linux machine. Output is sorted by size (largest first).

```bash
# Find files >100MB in specific paths
./penguinwash large-files --min-mb 100 --paths /home /var/cache /tmp

# Scan entire filesystem for files >500MB
./penguinwash large-files --min-mb 500 --paths /

# JSON output — share this file for remote analysis
./penguinwash large-files --min-mb 100 --paths / --json > scan-results.json
```

**Typical output:**
```
      Size  Path
--------------------------------------------------------------------------------
      1.2 GB  /home/user/.cache/qmd/models/model.gguf
    839.0 MB  /tmp/git-objects.pack
    609.5 MB  /var/cache/flatpak/...
    ...
13 files found
```

### Configuration

```bash
./penguinwash show-config     # print current config
./penguinwash reset-config     # reset to defaults
```

### GUI

```bash
./penguinwash --gui            # launch graphical interface
```

---

## Configuration

Config file: `~/.config/penguinwash.toml` (auto-created on first run)

```toml
large_file_threshold_mb = 100   # minimum size for large-file scan
old_file_threshold_days = 365  # age threshold for "old" files
journal_vacuum_days = 90        # journal log retention
log_vacuum_days = 90            # system log retention
snap_retain_revisions = 2       # snap revisions to keep
dry_run = true                  # always safe by default
auto_clean = false              # never auto-delete without --force
```

---

## Categories

| Category | Path | Auto-Clean |
|---|---|---|
| Snap Revisions | `/var/lib/snapd/snaps` | ✅ |
| Flatpak Orphans | `/var/lib/flatpak`, `~/.local/share/flatpak` | ✅ |
| Thumbnails | `~/.cache/thumbnails` | ✅ |
| User Cache | `~/.cache/*` | ✅ |
| APT Cache | `/var/cache/apt` | ✅ |
| DNF/YUM Cache | `/var/cache/dnf` | ✅ |
| Pacman Cache | `/var/cache/pacman` | ✅ |
| Journal Logs | `/var/log/journal` | ✅ |
| System Logs | `/var/log` | ❌ (manual) |
| Trash | `~/.local/share/Trash` | ✅ |
| Docker Images | `/var/lib/docker` | ✅ |
| Old Kernels | `/boot` | ❌ (manual) |
| Mail Attachments | `~/.local/share/mail-downloads` | ✅ |

---

## Remote Diagnosis Workflow

1. Copy `penguinwash` binary to the problematic machine
2. Run: `./penguinwash large-files --min-mb 100 --paths / --json > scan-results.json`
3. Share `scan-results.json` — it contains file paths, sizes, and access/modify times
4. Analyze: large log files, orphaned package caches, old kernel images, unused Docker images, etc.

---

## Safety

- Never deletes system-critical files
- Hardcoded exclusions: `/etc/`, `/.ssh/`, `/var/lib/systemd/`
- Large & old files are **never auto-selected**
- All deletion is dry-run by default — use `--force` to actually delete
- All deletion operations are non-destructive to the OS

---

## Architecture

- **Pure Rust** — no external runtime dependencies
- **Async** — tokio-powered parallel scanning
- **No GTK required** — egui GUI works on any Linux with OpenGL
- **Zero telemetry** — no network calls, no data collection

---

## License

MIT
