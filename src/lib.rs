//! PenguinWash core library

pub mod android;
pub mod categories;
pub mod cleaner;
pub mod config;
pub mod packagemgr;
pub mod scanner;
#[cfg(feature = "gui")]
pub mod gui;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

/// Represents a single file or directory to be cleaned
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanableItem {
    pub path: PathBuf,
    pub size: u64,
    pub category: String,
    pub description: String,
}

/// A scan result for one category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryScan {
    pub name: String,
    pub key: String,
    pub description: String,
    pub items: Vec<CleanableItem>,
    pub total_size: u64,
    pub item_count: usize,
    pub auto_cleanable: bool,
}

impl CategoryScan {
    pub fn total_size_formatted(&self) -> String {
        humanize_bytes(self.total_size)
    }
}

/// Full scan result across all categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub categories: Vec<CategoryScan>,
    pub grand_total_size: u64,
    pub scan_duration_ms: u64,
}

impl ScanResult {
    pub fn grand_total_size_formatted(&self) -> String {
        humanize_bytes(self.grand_total_size)
    }
}

/// Configuration for PenguinWash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub large_file_threshold_mb: u64,
    pub old_file_threshold_days: u64,
    pub journal_vacuum_days: u64,
    pub log_vacuum_days: u64,
    pub snap_retain_revisions: u32,
    pub dry_run: bool,
    pub auto_clean: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            large_file_threshold_mb: 100,
            old_file_threshold_days: 365,
            journal_vacuum_days: 90,
            log_vacuum_days: 90,
            snap_retain_revisions: 2,
            dry_run: true,
            auto_clean: false,
        }
    }
}

/// A large file entry for remote diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargeFileEntry {
    pub path: String,
    pub size_bytes: u64,
    pub modified: Option<SystemTime>,
    pub accessed: Option<SystemTime>,
}

impl LargeFileEntry {
    pub fn size_formatted(&self) -> String {
        humanize_bytes(self.size_bytes)
    }
}

/// Human-readable byte size
pub fn humanize_bytes(bytes: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < units.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.1} {}", size, units[unit_idx])
}

/// Run a full scan across all categories
pub async fn run_scan(config: &Config) -> Result<ScanResult> {
    use std::time::Instant;
    let start = Instant::now();

    let mut category_scans = Vec::new();

    for cat_def in categories::all_category_defs() {
        let scan = scanner::scan_category(cat_def, config).await?;
        category_scans.push(scan);
    }

    let grand_total_size: u64 = category_scans.iter().map(|c| c.total_size).sum();
    let elapsed = start.elapsed().as_millis() as u64;

    Ok(ScanResult {
        categories: category_scans,
        grand_total_size,
        scan_duration_ms: elapsed,
    })
}

/// Scan for large files on a remote machine
pub async fn run_large_file_scan(paths: Vec<PathBuf>, threshold_mb: u64) -> Result<Vec<LargeFileEntry>> {
    let mut all_files = Vec::new();

    for root in paths {
        if !root.exists() {
            tracing::warn!("Path does not exist, skipping: {}", root.display());
            continue;
        }
        let files = scanner::scan_large_files(&root, threshold_mb).await?;
        all_files.extend(files);
    }

    // Sort all by size descending
    all_files.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

    Ok(all_files)
}
