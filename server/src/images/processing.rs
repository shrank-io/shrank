use std::path::Path;

use image::imageops::FilterType;
use image::ImageReader;

use crate::config::ImageConfig;
use crate::AppError;

/// Generate a thumbnail from the original image.
/// Runs image processing on a blocking thread to avoid starving the async runtime.
pub async fn generate_thumbnail(
    input_path: &Path,
    output_path: &Path,
    config: &ImageConfig,
) -> Result<(), AppError> {
    let input = input_path.to_path_buf();
    let output = output_path.to_path_buf();
    let width = config.thumbnail_width;
    let quality = config.thumbnail_quality;

    tokio::task::spawn_blocking(move || -> Result<(), AppError> {
        let img = ImageReader::open(&input)?.with_guessed_format()?.decode()?;

        // Compute target dimensions maintaining aspect ratio
        let (orig_w, orig_h) = (img.width(), img.height());
        let height = (width as f64 / orig_w as f64 * orig_h as f64) as u32;

        let thumbnail = img.resize(width, height, FilterType::Lanczos3);

        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Encode as WebP
        let file = std::fs::File::create(&output)?;
        let writer = std::io::BufWriter::new(file);
        let encoder = image::codecs::webp::WebPEncoder::new_lossless(writer);

        // Use lossy encoding with quality setting
        // Note: image crate's WebP encoder quality is set at construction.
        // For lossy, we'd need the webp crate directly. For now use lossless
        // which produces good results at reasonable file sizes for thumbnails.
        thumbnail.write_with_encoder(encoder)?;

        let _ = quality; // TODO: use lossy quality when image crate supports it better

        Ok(())
    })
    .await
    .map_err(|e| AppError::Internal(format!("thumbnail task panicked: {e}")))?
}
