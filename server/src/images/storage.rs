use std::path::{Path, PathBuf};

use crate::AppError;

/// Returns the relative path for an original image: originals/{ulid}/scan.jpg
pub fn original_rel_path(doc_id: &str) -> String {
    format!("originals/{doc_id}/scan.jpg")
}

/// Returns the relative path for a thumbnail: thumbnails/{ulid}/thumb.webp
pub fn thumbnail_rel_path(doc_id: &str) -> String {
    format!("thumbnails/{doc_id}/thumb.webp")
}

/// Returns the absolute path for an original image.
pub fn original_abs_path(data_dir: &Path, doc_id: &str) -> PathBuf {
    data_dir.join(original_rel_path(doc_id))
}

/// Returns the absolute path for a thumbnail.
pub fn thumbnail_abs_path(data_dir: &Path, doc_id: &str) -> PathBuf {
    data_dir.join(thumbnail_rel_path(doc_id))
}

/// Store the original image bytes to disk.
pub async fn store_original(data_dir: &Path, doc_id: &str, bytes: &[u8]) -> Result<String, AppError> {
    let rel = original_rel_path(doc_id);
    let abs = data_dir.join(&rel);

    if let Some(parent) = abs.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    tokio::fs::write(&abs, bytes).await?;
    Ok(rel)
}

/// Delete all images (original + thumbnail) for a document.
pub async fn delete_images(data_dir: &Path, doc_id: &str) -> Result<(), AppError> {
    let orig_dir = data_dir.join("originals").join(doc_id);
    let thumb_dir = data_dir.join("thumbnails").join(doc_id);

    if orig_dir.exists() {
        tokio::fs::remove_dir_all(&orig_dir).await.ok();
    }
    if thumb_dir.exists() {
        tokio::fs::remove_dir_all(&thumb_dir).await.ok();
    }

    Ok(())
}
