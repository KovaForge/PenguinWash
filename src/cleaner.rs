//! Cleaner module — safely delete files with confirmation guards

use crate::CleanableItem;
use anyhow::{Context, Result};
use std::path::Path;
use tokio::fs;

/// Delete a list of cleanable items. Always confirm first in dry-run mode.
pub async fn delete_items(items: &[CleanableItem], dry_run: bool) -> Result<DeleteSummary> {
    let mut deleted = 0;
    let mut failed = 0;
    let mut freed_bytes: u64 = 0;

    for item in items {
        if dry_run {
            tracing::info!("[DRY RUN] Would delete: {}", item.path.display());
            continue;
        }

        match safe_delete(&item.path).await {
            Ok(size) => {
                deleted += 1;
                freed_bytes += size;
                tracing::info!("Deleted: {} ({} bytes)", item.path.display(), size);
            }
            Err(e) => {
                failed += 1;
                tracing::warn!("Failed to delete {}: {}", item.path.display(), e);
            }
        }
    }

    Ok(DeleteSummary {
        deleted,
        failed,
        freed_bytes,
    })
}

/// Safely delete a single file (never directories in system paths)
async fn safe_delete(path: &Path) -> Result<u64> {
    if crate::categories::is_excluded(&path.to_path_buf()) {
        anyhow::bail!("Path is in exclusion list, refusing to delete: {}", path.display());
    }

    let meta = fs::metadata(path).await?;
    let size = meta.len();

    fs::remove_file(path).await
        .with_context(|| format!("Failed to delete {}", path.display()))?;

    Ok(size)
}

#[derive(Debug, Clone)]
pub struct DeleteSummary {
    pub deleted: usize,
    pub failed: usize,
    pub freed_bytes: u64,
}
