//! Package manager detection and cleaning
//!
//! Detects installed package managers and provides their clean commands.

use std::path::PathBuf;
use std::process::Command;

/// Detected package manager on the system
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PackageManager {
    Apt,
    Dnf,
    Pacman,
    Zypper,
    Snap,
    Flatpak,
}

impl PackageManager {
    /// Human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Apt => "APT (Debian/Ubuntu)",
            Self::Dnf => "DNF (Fedora/RHEL)",
            Self::Pacman => "Pacman (Arch)",
            Self::Zypper => "Zypper (openSUSE)",
            Self::Snap => "Snap",
            Self::Flatpak => "Flatpak",
        }
    }

    /// Clean command to run (returns command + args)
    pub fn clean_cmd(&self) -> (&'static str, Vec<String>) {
        match self {
            Self::Apt => ("apt", vec!["clean".to_string()]),
            Self::Dnf => ("dnf", vec!["clean".to_string(), "all".to_string()]),
            Self::Pacman => ("pacman", vec!["-Sc".to_string()]),
            Self::Zypper => ("zypper", vec!["clean".to_string()]),
            Self::Snap => ("snap", vec!["refresh".to_string()]),
            Self::Flatpak => ("flatpak", vec!["--unused".to_string(), "--delete".to_string()]),
        }
    }
}

/// Detect which package managers are available on this system
pub fn detect_package_managers() -> Vec<PackageManager> {
    let mut detected = Vec::new();

    for pm in ALL_PACKAGE_MANAGERS.iter() {
        if is_installed(pm) {
            detected.push(*pm);
        }
    }

    detected
}

fn is_installed(pm: &PackageManager) -> bool {
    let (cmd, _) = pm.clean_cmd();
    PathBuf::from(cmd).exists() || which(cmd).is_some()
}

fn which(cmd: &str) -> Option<PathBuf> {
    std::env::var_os("PATH")
        .and_then(|paths| {
            std::env::split_paths(&paths)
                .map(|p| p.join(cmd))
                .find(|p| p.exists())
        })
}

const ALL_PACKAGE_MANAGERS: &[PackageManager] = &[
    PackageManager::Apt,
    PackageManager::Dnf,
    PackageManager::Pacman,
    PackageManager::Zypper,
    PackageManager::Snap,
    PackageManager::Flatpak,
];

/// Run a package manager's clean command
pub async fn run_clean(pm: PackageManager) -> Result<(), std::io::Error> {
    let (cmd, args) = pm.clean_cmd();
    Command::new(cmd).args(&args).spawn()?.wait()?;
    Ok(())
}
