# PenguinWash — Implementation Plan

## 1. Concept & Goal

**PenguinWash** is a free, open-source Linux system cleaner — the PureMac equivalent for Linux. It scans and safely removes:
- System caches and logs
- User application caches (browser, npm, pip, etc.)
- Package manager caches (apt, dnf, yum, pacman, snap, flatpak)
- Old journal logs
- Trash bins
- Large and old files
- Orphaned flatpak/snap packages

No telemetry. No subscriptions. Runs entirely offline. Native Linux toolchain.

---

## 2. Linux Port Mapping (PureMac → Linux)

| PureMac Category | Linux Equivalent | Paths |
|---|---|---|
| System Junk | System caches/logs | `/var/cache`, `/var/log`, `/tmp`, `~/.cache` |
| User Cache | User caches | `~/.cache` (browsers, npm, pip, yarn, pnpm) |
| Mail Attachments | Mail downloads | `~/.local/share/mail-downloads` (Thunderbird, etc.) |
| Trash | Trash | `~/.local/share/Trash` |
| Large & Old Files | Downloads/Documents | `~/Downloads`, `~/Documents`, `~/Desktop` |
| Xcode Junk | Dev caches | `~/.cache/pip`, `~/.npm`, `~/.cargo`, `~/.texlive` |
| Homebrew Cache | Package manager caches | apt (`/var/cache/apt`), dnf (`/var/cache/dnf`), pacman (`/var/cache/pacman`), snap (`/var/snap`), flatpak (`/var/lib/flatpak`) |
| Purgeable Space | Journal + old logs | `journalctl --vacuum` |
| — | Orphan packages | flatpak, snap orphans |

---

## 3. Tech Stack

- **Language:** Rust (safe, fast, cross-Linux-distro)
- **Frontend:** CLI with `--scan`, `--clean`, `--interactive`
- **Optional TUI:** `ratatui` for a curses-based UI (future)
- **Package managers supported:** apt (Debian/Ubuntu), dnf/yum (Fedora/RHEL), pacman (Arch), zypper (openSUSE), snap, flatpak
- **Build:** `cargo build --release`
- **Target:** x86_64 Linux (glibc-based)

---

## 4. Feature Phases

### Phase 1 — Core CLI
- [ ] Scan categories listed above
- [ ] Display file sizes and counts per category
- [ ] `--safe` mode (default): never auto-select large/old files or system-critical paths
- [ ] `--clean` flag to execute deletion after confirmation
- [ ] `--json` output for scripting

### Phase 2 — System Integration
- [ ] Root-required cleaning: `/var/cache`, `/var/log` (use `pkexec` or sudo)
- [ ] Detect installed package managers automatically
- [ ] Respect `manjaro-pamac`, `octopi` cache locations

### Phase 3 — Automation
- [ ] Config file: `~/.config/penguinwash.toml`
- [ ] Scheduled cleaning via cron/systemd timer
- [ ] `--watch` daemon mode (lightweight, low-frequency scans)

### Phase 4 — GUI (Optional)
- [ ] TUI using `ratatui`
- [ ] GTK/Qt bindings for desktop integration

---

## 5. File Categories & Safety Rules

```
CATEGORY          PATH(S)                          SAFE TO AUTO-CLEAN?
System Junk       /var/cache/*, /var/log/*.log      Partial (old logs only)
User Cache        ~/.cache/*                       Yes (user-owned)
Trash             ~/.local/share/Trash              Yes
Package Caches    /var/cache/apt, /var/cache/dnf   Yes (package cache only)
                  /var/cache/pacman                Yes
                  /var/lib/flatpak                  Partial (orphans only)
                  /var/lib/snap                     Partial (orphans only)
Large Files       ~/Downloads, ~/Documents          NO (manual selection only)
Old Logs          /var/log/*.log older than 90d    Partial (configurable)
```

**Hardcoded exclusions:**
- `/etc/` — never touch
- `/boot/` — never touch
- `/home/*/.ssh/` — never touch
- `/var/lib/systemd/` — never touch

---

## 6. Directory Structure

```
PenguinWash/
├── Cargo.toml
├── src/
│   ├── main.rs           # CLI entry, argument parsing
│   ├── scanner.rs        # Recursive scan + categorization
│   ├── cleaner.rs        # Deletion logic with safety checks
│   ├── categories.rs     # Category definitions + Linux paths
│   ├── packagemgr.rs    # Package manager detection + cleaning
│   ├── config.rs        # Config file loading
│   └── output.rs        # JSON/table output formatting
├── config/
│   └── penguinwash.toml.example
├── docs/
│   └── IMPLEMENTATION_PLAN.md
├── README.md
└── LICENSE
```

---

## 7. Build & Release

- **CI:** GitHub Actions — on tag push, build `.deb`, `.rpm`, `.AppImage`
- **Install methods:**
  - Binary download from Releases
  - Homebrew (Linuxbrew) — future
  - `cargo install penguinwash`

---

## 8. openclaw Integration

- Repo: `KovaForge/PenguinWash` (private by default, confirm with CEO before public)
- Cron: code review + build verify on push to `develop`
- Branching: `KFIP####-short-description` per KFIP convention
