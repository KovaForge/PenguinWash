//! Scanner module — traverses directories and categorizes cleanable items
//!
//! Uses async parallel traversal for performance.

use crate::categories::{self, Category};
use crate::CleanableItem;
use crate::Config;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio::fs;
use walkdir::WalkDir;

/// Scan a single category and return all cleanable items found.
pub async fn scan_category(cat: &Category, _config: &Config) -> Result<crate::CategoryScan> {
    let mut items = Vec::new();
    let mut total_size: u64 = 0;

    for base_path in cat.resolve_paths() {
        if !base_path.exists() {
            continue;
        }

        let entries = walkdir_entries(&base_path).await?;
        for dir_entry in entries {
            let path = dir_entry.path().to_path_buf();

            // Skip excluded paths
            if categories::is_excluded(&path) {
                continue;
            }

            // Handle snap revisions — only disabled ones
            if cat.key == "snap-revisions" && !categories::is_disabled_snap(&path) {
                continue;
            }

            // Handle old kernels — skip current
            if cat.key == "old-kernels" && categories::is_current_kernel(&path) {
                continue;
            }

            // Handle flatpak orphans
            if cat.key == "flatpak-orphans" && !categories::is_flatpak_orphan(&path) {
                continue;
            }

            // Handle user cache — skip subcategories handled elsewhere
            if cat.key == "user-cache" {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name == "thumbnails" || name == "mozilla" || name == "google-chrome" {
                        continue; // separate categories
                    }
                }
            }

            if let Ok(meta) = fs::metadata(&path).await {
                let size = meta.len();
                total_size += size;
                items.push(CleanableItem {
                    path: path.clone(),
                    size,
                    category: cat.key.to_string(),
                    description: describe_item(&path),
                });
            }
        }
    }

    Ok(crate::CategoryScan {
        name: cat.name.to_string(),
        key: cat.key.to_string(),
        description: cat.description.to_string(),
        item_count: items.len(),
        total_size,
        auto_cleanable: cat.auto_cleanable,
        items,
    })
}

/// Async directory walker using walkdir + tokio
async fn walkdir_entries(path: &Path) -> Result<Vec<walkdir::DirEntry>> {
    let path = path.to_path_buf();
    let entries = tokio::task::spawn_blocking(move || {
        WalkDir::new(&path)
            .follow_links(false)
            .max_depth(10)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .collect::<Vec<_>>()
    })
    .await
    .context("walkdir task panicked")?;
    Ok(entries)
}

/// Human-readable description of an item
fn describe_item(path: &Path) -> String {
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if let Some(parent) = path.parent().and_then(|p| p.file_name().and_then(|n| n.to_str())) {
            format!("{} — {}", parent, name)
        } else {
            name.to_string()
        }
    } else {
        path.to_string_lossy().to_string()
    }
}

/// Check if a file is "old" based on threshold days
pub fn is_old_file(path: &Path, threshold_days: u64) -> Result<bool> {
    let meta = std::fs::metadata(path)?;
    let modified = meta.modified()?;
    let age = SystemTime::now()
        .duration_since(modified)
        .map(|d| d.as_secs() / 86400)
        .unwrap_or(0);
    Ok(age >= threshold_days)
}

/// Check if a file exceeds size threshold (in MB)
pub fn is_large_file(path: &Path, threshold_mb: u64) -> Result<bool> {
    let meta = std::fs::metadata(path)?;
    Ok(meta.len() >= threshold_mb * 1024 * 1024)
}

// ─── Large file scanner (for remote diagnosis) ─────────────────────────────────

use crate::LargeFileEntry;

/// Scan for large files starting from `root`, exceeding `threshold_mb`.
/// Returns sorted by size, largest first.
pub async fn scan_large_files(root: &Path, threshold_mb: u64) -> Result<Vec<LargeFileEntry>> {
    let root = root.to_path_buf();
    let threshold = threshold_mb * 1024 * 1024;

    let entries = tokio::task::spawn_blocking(move || {
        WalkDir::new(&root)
            .follow_links(false)
            .max_depth(20)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| {
                let meta = e.metadata().ok()?;
                if meta.len() < threshold {
                    return None;
                }
                let modified = meta.modified().ok();
                let accessed = meta.accessed().ok();
                Some(LargeFileEntry {
                    path: e.path().display().to_string(),
                    size_bytes: meta.len(),
                    modified,
                    accessed,
                })
            })
            .collect::<Vec<_>>()
    })
    .await
    .context("large file scan panicked")?;

    // Sort by size descending
    let mut entries = entries;
    entries.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

    Ok(entries)
}

/// Default paths to scan for large files
pub fn default_large_file_paths() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/"),
    ]
}
