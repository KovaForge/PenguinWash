//! Android SDK and emulator relocation helpers.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::symlink;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelocationItem {
    pub label: String,
    pub source: PathBuf,
    pub destination: PathBuf,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidRelocationPlan {
    pub target_root: PathBuf,
    pub target_available_bytes: Option<u64>,
    pub total_size_bytes: u64,
    pub cross_filesystem: bool,
    pub target_writable: bool,
    pub items: Vec<RelocationItem>,
    pub warnings: Vec<String>,
    pub env_hints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidRelocationSummary {
    pub relocated_items: usize,
    pub freed_home_bytes: u64,
    pub target_root: PathBuf,
}

pub fn plan_android_relocation(target_root: Option<PathBuf>) -> Result<AndroidRelocationPlan> {
    let home = dirs::home_dir().context("Unable to determine the home directory")?;
    let resolved_target = match target_root {
        Some(path) => path,
        None => discover_target_root(&home)?,
    };

    if !resolved_target.exists() {
        bail!(
            "Target root does not exist: {}",
            resolved_target.display()
        );
    }

    let items = discover_android_items(&home, &resolved_target)?;
    if items.is_empty() {
        bail!("No Android SDK or AVD directories were found in the home partition");
    }

    let total_size_bytes = items.iter().map(|item| item.size_bytes).sum();
    let cross_filesystem = fs_device_id(&home)? != fs_device_id(&resolved_target)?;
    let target_available_bytes = available_bytes(&resolved_target);
    let target_writable = is_writable_path(&resolved_target);
    let mut warnings = Vec::new();

    if resolved_target.starts_with(&home) {
        warnings.push(format!(
            "Target {} is still inside the home directory, so this will not reduce /home usage.",
            resolved_target.display()
        ));
    }

    if !cross_filesystem {
        warnings.push(format!(
            "Target {} is on the same filesystem as /home, so this will reorganize paths but not free the home partition.",
            resolved_target.display()
        ));
    }

    if !target_writable {
        warnings.push(format!(
            "Target {} is not writable for the current user.",
            resolved_target.display()
        ));
    }

    if let Some(free) = target_available_bytes {
        if free < total_size_bytes {
            warnings.push(format!(
                "Target only has {} free but Android data needs {}.",
                crate::humanize_bytes(free),
                crate::humanize_bytes(total_size_bytes)
            ));
        }
    }

    Ok(AndroidRelocationPlan {
        target_root: resolved_target,
        target_available_bytes,
        total_size_bytes,
        cross_filesystem,
        target_writable,
        items,
        warnings,
        env_hints: android_env_hints(),
    })
}

pub fn execute_android_relocation(plan: &AndroidRelocationPlan) -> Result<AndroidRelocationSummary> {
    if !plan.target_writable {
        bail!(
            "Target root is not writable for the current user: {}",
            plan.target_root.display()
        );
    }

    if let Some(free) = plan.target_available_bytes {
        if free < plan.total_size_bytes {
            bail!(
                "Target root does not have enough free space: {} available, {} required",
                crate::humanize_bytes(free),
                crate::humanize_bytes(plan.total_size_bytes)
            );
        }
    }

    let mut relocated_items = 0usize;
    let mut freed_home_bytes = 0u64;

    for item in &plan.items {
        relocate_item(item)?;
        relocated_items += 1;
        freed_home_bytes += item.size_bytes;
    }

    Ok(AndroidRelocationSummary {
        relocated_items,
        freed_home_bytes,
        target_root: plan.target_root.clone(),
    })
}

pub fn format_android_relocation_plan(plan: &AndroidRelocationPlan) -> String {
    let mut lines = Vec::new();

    lines.push("Android relocation plan".to_string());
    lines.push(format!("  Target root: {}", plan.target_root.display()));
    lines.push(format!(
        "  Estimated home space to free: {}",
        crate::humanize_bytes(plan.total_size_bytes)
    ));

    if let Some(free) = plan.target_available_bytes {
        lines.push(format!(
            "  Target free space: {}",
            crate::humanize_bytes(free)
        ));
    }

    for item in &plan.items {
        lines.push(format!(
            "  - {}: {} -> {} ({})",
            item.label,
            item.source.display(),
            item.destination.display(),
            crate::humanize_bytes(item.size_bytes)
        ));
    }

    if !plan.warnings.is_empty() {
        lines.push(String::new());
        lines.push("Warnings:".to_string());
        for warning in &plan.warnings {
            lines.push(format!("  - {}", warning));
        }
    }

    lines.push(String::new());
    lines.push("Environment hints:".to_string());
    for hint in &plan.env_hints {
        lines.push(format!("  - {}", hint));
    }

    lines.join("\n")
}

fn discover_android_items(home: &Path, target_root: &Path) -> Result<Vec<RelocationItem>> {
    let managed_root = target_root.join("penguinwash-relocated").join(current_username());
    let candidates = [
        ("Android SDK", home.join("Android").join("Sdk")),
        ("Android AVDs", home.join(".android").join("avd")),
        (
            "Android Studio Flatpak AVDs",
            home.join(".var")
                .join("app")
                .join("com.google.AndroidStudio")
                .join("config")
                .join(".android")
                .join("avd"),
        ),
    ];

    let mut items = Vec::new();
    for (label, source) in candidates {
        if !source.exists() {
            continue;
        }

        let metadata = fs::symlink_metadata(&source)
            .with_context(|| format!("Failed to inspect {}", source.display()))?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if !metadata.is_dir() {
            continue;
        }

        let relative = relative_home_path(home, &source)?;
        let destination = managed_root.join(relative);
        let size_bytes = directory_size(&source)?;
        items.push(RelocationItem {
            label: label.to_string(),
            source,
            destination,
            size_bytes,
        });
    }

    Ok(items)
}

fn relative_home_path(home: &Path, source: &Path) -> Result<PathBuf> {
    source
        .strip_prefix(home)
        .map(PathBuf::from)
        .with_context(|| format!("{} is not inside {}", source.display(), home.display()))
}

fn current_username() -> String {
    std::env::var("USER").unwrap_or_else(|_| "user".to_string())
}

fn discover_target_root(home: &Path) -> Result<PathBuf> {
    let home_dev = fs_device_id(home)?;
    let mut best: Option<(PathBuf, u64)> = None;

    for base in candidate_mount_bases() {
        if !base.exists() {
            continue;
        }

        let entries = fs::read_dir(&base)
            .with_context(|| format!("Failed to read {}", base.display()))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let dev = match fs_device_id(&path) {
                Ok(dev) => dev,
                Err(_) => continue,
            };
            if dev == home_dev {
                continue;
            }
            if !is_writable_path(&path) {
                continue;
            }

            let free = available_bytes(&path).unwrap_or(0);
            match &best {
                Some((_, best_free)) if free <= *best_free => {}
                _ => best = Some((path, free)),
            }
        }
    }

    best.map(|(path, _)| path).with_context(|| {
        "Unable to auto-detect a mounted target outside /home. Pass --target-root explicitly."
            .to_string()
    })
}

fn candidate_mount_bases() -> Vec<PathBuf> {
    let mut bases = Vec::new();

    if let Ok(user) = std::env::var("USER") {
        bases.push(PathBuf::from("/media").join(&user));
        bases.push(PathBuf::from("/run/media").join(user));
    }
    bases.push(PathBuf::from("/mnt"));

    bases
}

fn fs_device_id(path: &Path) -> Result<u64> {
    let metadata = fs::metadata(path)
        .with_context(|| format!("Failed to inspect filesystem for {}", path.display()))?;
    Ok(metadata.dev())
}

fn available_bytes(path: &Path) -> Option<u64> {
    let output = Command::new("df")
        .args(["-B1", "--output=avail"])
        .arg(path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    stdout
        .lines()
        .skip(1)
        .find_map(|line| line.trim().parse::<u64>().ok())
}

fn is_writable_path(path: &Path) -> bool {
    Command::new("test")
        .arg("-w")
        .arg(path)
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn directory_size(path: &Path) -> Result<u64> {
    let mut total = 0u64;

    for entry in WalkDir::new(path).follow_links(false) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        total += entry.metadata()?.len();
    }

    Ok(total)
}

fn relocate_item(item: &RelocationItem) -> Result<()> {
    if item.destination.exists() {
        bail!(
            "Destination already exists, refusing to overwrite: {}",
            item.destination.display()
        );
    }

    let source_meta = fs::symlink_metadata(&item.source)
        .with_context(|| format!("Failed to inspect {}", item.source.display()))?;
    if source_meta.file_type().is_symlink() {
        bail!("Source is already a symlink: {}", item.source.display());
    }
    let source_uid = source_meta.uid();
    let source_gid = source_meta.gid();

    let destination_parent = item
        .destination
        .parent()
        .context("Destination has no parent directory")?;
    fs::create_dir_all(destination_parent).with_context(|| {
        format!(
            "Failed to create destination directory {}",
            destination_parent.display()
        )
    })?;
    apply_ownership(destination_parent, source_uid, source_gid, true).ok();

    move_directory(&item.source, &item.destination)?;

    let moved_size = directory_size(&item.destination)?;
    if moved_size != item.size_bytes {
        rollback_move(&item.destination, &item.source).ok();
        bail!(
            "Moved data size mismatch for {}: expected {}, found {}",
            item.label,
            crate::humanize_bytes(item.size_bytes),
            crate::humanize_bytes(moved_size)
        );
    }

    if let Err(error) = symlink(&item.destination, &item.source) {
        rollback_move(&item.destination, &item.source).ok();
        return Err(error).with_context(|| {
            format!(
                "Failed to create symlink {} -> {}",
                item.source.display(),
                item.destination.display()
            )
        });
    }
    apply_ownership(&item.destination, source_uid, source_gid, true).ok();
    apply_ownership(&item.source, source_uid, source_gid, false).ok();

    Ok(())
}

fn rollback_move(destination: &Path, source: &Path) -> Result<()> {
    if source.exists() {
        let metadata = fs::symlink_metadata(source)?;
        if metadata.file_type().is_symlink() {
            fs::remove_file(source)?;
        }
    }

    move_directory(destination, source)
}

fn move_directory(source: &Path, destination: &Path) -> Result<()> {
    let status = Command::new("mv")
        .arg(source)
        .arg(destination)
        .status()
        .with_context(|| {
            format!(
                "Failed to launch mv for {} -> {}",
                source.display(),
                destination.display()
            )
        })?;

    if !status.success() {
        bail!(
            "mv failed for {} -> {} with status {}",
            source.display(),
            destination.display(),
            status
        );
    }

    Ok(())
}

fn apply_ownership(path: &Path, uid: u32, gid: u32, recursive: bool) -> Result<()> {
    let mut command = Command::new("chown");
    if recursive {
        command.arg("-R");
    } else {
        command.arg("-h");
    }
    command
        .arg(format!("{}:{}", uid, gid))
        .arg(path);

    let status = command.status().with_context(|| {
        format!("Failed to launch chown for {}", path.display())
    })?;

    if !status.success() {
        bail!(
            "chown failed for {} with status {}",
            path.display(),
            status
        );
    }

    Ok(())
}

fn android_env_hints() -> Vec<String> {
    vec![
        "ANDROID_HOME can point at the relocated SDK if you want a direct path instead of relying on the symlink.".to_string(),
        "ANDROID_USER_HOME can move Android tool metadata out of ~/.android when needed.".to_string(),
        "ANDROID_AVD_HOME is the supported way to place emulator images outside ~/.android/avd.".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_home_path_keeps_expected_suffix() {
        let home = PathBuf::from("/home/tester");
        let sdk = home.join("Android").join("Sdk");
        let avd = home.join(".android").join("avd");

        assert_eq!(relative_home_path(&home, &sdk).unwrap(), PathBuf::from("Android/Sdk"));
        assert_eq!(relative_home_path(&home, &avd).unwrap(), PathBuf::from(".android/avd"));
    }

    #[test]
    fn env_hints_cover_android_sdk_and_avd() {
        let hints = android_env_hints();

        assert!(hints.iter().any(|hint| hint.contains("ANDROID_HOME")));
        assert!(hints.iter().any(|hint| hint.contains("ANDROID_USER_HOME")));
        assert!(hints.iter().any(|hint| hint.contains("ANDROID_AVD_HOME")));
    }

    #[test]
    fn relative_home_path_supports_flatpak_android_studio_avds() {
        let home = PathBuf::from("/home/tester");
        let flatpak_avd = home
            .join(".var")
            .join("app")
            .join("com.google.AndroidStudio")
            .join("config")
            .join(".android")
            .join("avd");

        assert_eq!(
            relative_home_path(&home, &flatpak_avd).unwrap(),
            PathBuf::from(".var/app/com.google.AndroidStudio/config/.android/avd")
        );
    }
}
