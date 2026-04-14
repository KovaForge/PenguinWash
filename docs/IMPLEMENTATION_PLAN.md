# PenguinWash — Implementation Plan

## 1. Concept & Goal

**PenguinWash** is a free, open-source Linux system cleaner — the PureMac equivalent for Linux, inspired by BleachBit. It scans and safely removes junk to reclaim disk space.

No telemetry. No subscriptions. Runs entirely offline. Native Linux GUI.

---

## 2. Linux-Specific Disk Bloat Areas (Research-Informed)

### Highest-Impact Bloat Categories

| Category | Path(s) | Typical Size | Auto-Cleanable |
|---|---|---|---|
| **Snap revisions** | `/var/lib/snapd/snaps` | 1–10 GB | Yes — disabled/old revisions only |
| **Flatpak orphans** | `/var/lib/flatpak`, `~/.local/share/flatpak` | 500 MB–5 GB | Yes — unused runtimes |
| **Journal logs** | `/var/log/journal/` | 100 MB–15 GB | Partial — older than 90 days |
| **Docker images** | `/var/lib/docker` | 1–20 GB | Yes — unused images + volumes |
| **Old kernels** | `/boot` | 200–800 MB | Partial — old versions only |
| **Thumbnails** | `~/.cache/thumbnails` | 50 MB–2 GB | Yes |
| **User caches** | `~/.cache` (browsers, pip, npm, conda, go) | 100 MB–10 GB | Yes |
| **Package manager caches** | apt `/var/cache/apt`, dnf `/var/cache/dnf`, pacman `/var/cache/pacman` | 500 MB–5 GB | Yes |
| **Trash** | `~/.local/share/Trash` | varies | Yes |
| **System logs** | `/var/log/*.log` (kern.log, ufw.log) | 100 MB–20 GB | Partial — rotated logs older than N days |
| **Flatpak user install** | `~/.local/share/flatpak` | varies | Partial — unused apps |

### Rare but Massive Bloat
- `/var/log/` alone: users report 40GB+ in single log files (kern.log, ufw.log)
- Snap disabled revisions: forum post shows `/var/lib/snapd/snaps` growing from 2.8GB → 6.2GB in 2 years
- Flatpak unused runtimes accumulate after app updates

---

## 3. Tech Stack

- **Language:** Rust
- **GUI:** GTK4 + libadwaita (modern GNOME HIG, flatpak-friendly)
- **Frontend:** `penguinwash` CLI with `--scan`, `--clean`, `--preview`
- **Backend:** async scan with `tokio`, parallel directory traversal
- **Package managers:** apt, dnf, pacman, zypper, snap, flatpak detection + cleaning
- **System integration:** `pkexec` for root-required operations
- **Build:** `cargo build --release`, `.deb` + `.rpm` via GitHub Actions

---

## 4. Categories & Safety Rules

```
CATEGORY              PATH(S)                                   SAFE TO AUTO-CLEAN?
Snap Revisions        /var/lib/snapd/snaps                      YES (disabled only)
Flatpak Orphans       /var/lib/flatpak, ~/.local/share/flatpak  YES (unused runtimes)
Thumbnails            ~/.cache/thumbnails                       YES
User Cache            ~/.cache/* (pip, npm, conda, go, etc.)     YES
Package Cache         /var/cache/apt, /var/cache/dnf,          YES
                       /var/cache/pacman, /var/cache/zypper
Docker Images         /var/lib/docker                           Partial (unused only)
Trash                 ~/.local/share/Trash                      YES
Old Logs              /var/log/*.log (>90 days)                 Partial (configurable)
System Logs           /var/log/ (kern.log, ufw.log, etc.)      Partial (rotated only)
Old Kernels           /boot                                     Partial (old versions only)
Large Files           ~/Downloads, ~/Documents, ~/Desktop       NO (manual select)
Mail Attachments      ~/.local/share/mail-downloads             YES
Browser Data          ~/.cache/<browser>                        YES
```

**Hardcoded exclusions (NEVER touch):**
- `/etc/` — never
- `/boot/` — never
- `/home/*/.ssh/` — never
- `/var/lib/systemd/` — never
- `/var/lib/docker/containers/` — running containers
- `/var/lib/dockeroverlay2/` — overlay filesystem

---

## 5. Directory Structure

```
PenguinWash/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI entry, argument parsing
│   ├── gui.rs                # GTK4 app window + event loop
│   ├── scanner.rs            # Recursive scan + categorization
│   ├── cleaner.rs            # Deletion with safety confirmations
│   ├── categories.rs         # Category definitions + Linux paths
│   ├── packagemgr.rs         # Package manager detection + cleaning
│   ├── docker.rs             # Docker image/volume cleaning
│   ├── config.rs             # Config file (~/.config/penguinwash.toml)
│   ├── output.rs             # JSON / human-readable output
│   └── i18n.rs               # Internationalization (future)
├── data/
│   └── io.penguinwash.PenguinWash.desktop.yaml   # XDG desktop entry
├── po/                        # Translation files (future)
├── docs/
│   └── IMPLEMENTATION_PLAN.md
├── README.md
└── LICENSE
```

---

## 6. Feature Phases

### Phase 1 — Core + GUI
- [ ] GTK4 app window: category list, scan button, size per category
- [ ] Category detail view: file list, individual selection
- [ ] Scan engine: parallel traversal with progress
- [ ] Preview → Confirm → Delete flow (like PureMac)
- [ ] `~/.cache/thumbnails`, `~/.cache/<browser>`, `~/.cache/pip`, `~/.cache/npm`
- [ ] Snap revision scan (disabled only — never remove current)
- [ ] Flatpak orphan scan (unused runtimes)

### Phase 2 — System + Package Managers
- [ ] Root-required scanning via `pkexec` for `/var/cache`, `/var/log`
- [ ] apt cache clean (`apt clean`)
- [ ] dnf cache clean (`dnf clean all`)
- [ ] pacman cache clean (`pacman -Sc`)
- [ ] Journal vacuum (`journalctl --vacuum-time=90d`)
- [ ] Docker image + volume cleaning (`docker prune -a`)

### Phase 3 — Large Files + Automation
- [ ] Large file finder (>100MB, configurable)
- [ ] Old file finder (>365 days, configurable)
- [ ] Config file: `~/.config/penguinwash.toml`
- [ ] Scheduled cleaning via systemd timer (more portable than cron on Arch/Fedora)
- [ ] `--watch` daemon for background monitoring

### Phase 4 — Polish + Release
- [ ] i18n (gettext, at least en/FR/DE/ES)
- [ ] `.deb` + `.rpm` package build via GitHub Actions
- [ ] Flatpak manifest for native Linux distribution

---

## 7. Comparison vs BleachBit / Ubuntu Cleaner

| Feature | PenguinWash | BleachBit | Ubuntu Cleaner |
|---|---|---|---|
| GUI | GTK4 (modern) | GTK2 (legacy) | GTK3 |
| Open source | MIT | GPL | GPL |
| Snap cleaning | YES | NO | NO |
| Flatpak orphan | YES | Partial | NO |
| Docker cleaning | YES | NO | NO |
| Large/old file scan | YES | NO | NO |
| No telemetry | YES | YES | YES |
| Scheduled cleaning | YES (systemd timer) | YES (cron) | NO |
| Rust | YES | NO (Python) | NO (Python) |

---

## 8. Build & Release

- **CI:** GitHub Actions — on tag, build `.deb`, `.rpm`, `.AppImage`
- **Install methods:**
  - Binary download from Releases
  - `cargo install penguinwash`
  - Flatpak (future)
  - Homebrew/Linuxbrew (future)

---

## 9. openclaw Integration

- Repo: `KovaForge/PenguinWash` (private)
- Branching: `KFIP####-short-description` per KFIP convention
- Cron: code review + cargo build verify on push to `develop`
