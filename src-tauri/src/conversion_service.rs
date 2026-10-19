use image::{codecs::jpeg::JpegEncoder, imageops, DynamicImage, ImageBuffer, ImageFormat, ImageReader, Rgb};
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

const SUPPORTED_FORMATS: &[&str] = &["png", "jpg", "jpeg", "webp", "bmp", "tif", "tiff"];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageOperationResult {
    pub input_path: String,
    pub output_path: String,
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub bytes_saved: i64,
}

fn jpeg_compatible(image: &DynamicImage) -> DynamicImage {
    let rgba = image.to_rgba8();
    let mut background = ImageBuffer::from_pixel(rgba.width(), rgba.height(), Rgb([255, 255, 255]));
    imageops::overlay(&mut background, &rgba, 0, 0);
    DynamicImage::ImageRgb8(background)
}

fn target_format(extension: &str) -> Result<ImageFormat, String> {
    match extension.trim_start_matches('.').to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
        "png" => Ok(ImageFormat::Png),
        "webp" => Ok(ImageFormat::WebP),
        "bmp" => Ok(ImageFormat::Bmp),
        "tif" | "tiff" => Ok(ImageFormat::Tiff),
        _ => Err(format!(
            "SHIFT's built-in image engine does not support the target format '.{extension}'. Supported: PNG, JPG, WebP, BMP and TIFF."
        )),
    }
}

fn extension_for(path: &Path) -> String {
    path.extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
}

fn decode(path: &Path) -> Result<DynamicImage, String> {
    ImageReader::open(path)
        .map_err(|error| format!("Could not open {}: {error}", path.display()))?
        .with_guessed_format()
        .map_err(|error| format!("Could not identify the image format for {}: {error}", path.display()))?
        .decode()
        .map_err(|error| format!("Could not decode {}: {error}", path.display()))
}

fn unique_sibling(input: &Path, extension: &str, label: &str) -> Result<PathBuf, String> {
    let parent = input
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let stem = input
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("The input filename is not valid Unicode: {}", input.display()))?;
    let clean_ext = extension.trim_start_matches('.').to_ascii_lowercase();
    for index in 0_u32..100_000 {
        let suffix = if index == 0 {
            String::new()
        } else {
            format!(" {index}")
        };
        let path = parent.join(format!("{stem} (SHIFT {label}{suffix}).{clean_ext}"));
        if !path.exists() {
            return Ok(path);
        }
    }
    Err(format!(
        "Could not find an unused output filename beside {}.",
        input.display()
    ))
}

fn finish_output(
    input: &Path,
    output_extension: &str,
    label: &str,
    mut temporary: NamedTempFile,
) -> Result<ImageOperationResult, String> {
    use std::io::Write;
    temporary
        .flush()
        .map_err(|error| format!("Could not flush the converted image: {error}"))?;
    let temporary_path = temporary.path().to_path_buf();
    ImageReader::open(&temporary_path)
        .map_err(|error| format!("Could not reopen the converted image: {error}"))?
        .with_guessed_format()
        .map_err(|error| format!("Could not identify the converted image: {error}"))?
        .decode()
        .map_err(|error| format!("Converted image validation failed: {error}"))?;

    let input_bytes = fs::metadata(input)
        .map_err(|error| format!("Could not measure {}: {error}", input.display()))?
        .len();
    let output_bytes = fs::metadata(&temporary_path)
        .map_err(|error| format!("Could not measure the converted image: {error}"))?
        .len();
    let final_path = unique_sibling(input, output_extension, label)?;
    temporary.persist_noclobber(&final_path).map_err(|error| {
        format!(
            "The output could not be saved safely to {}: {}",
            final_path.display(),
            error.error
        )
    })?;

    Ok(ImageOperationResult {
        input_path: input.to_string_lossy().into_owned(),
        output_path: final_path.to_string_lossy().into_owned(),
        input_bytes,
        output_bytes,
        bytes_saved: (input_bytes as i128 - output_bytes as i128)
            .clamp(i64::MIN as i128, i64::MAX as i128) as i64,
    })
}

fn validate_inputs(inputs: &[String]) -> Result<Vec<PathBuf>, String> {
    if inputs.is_empty() {
        return Err("Drag or choose at least one image.".to_owned());
    }
    if inputs.len() > 500 {
        return Err("Process no more than 500 images in one batch.".to_owned());
    }
    inputs
        .iter()
        .map(|raw| {
            let path = PathBuf::from(raw);
            let link_metadata = fs::symlink_metadata(&path)
                .map_err(|error| format!("Cannot access {}: {error}", path.display()))?;
            if link_metadata.file_type().is_symlink() {
                return Err(format!("Symbolic-link inputs are not supported: {}", path.display()));
            }
            let metadata = fs::metadata(&path)
                .map_err(|error| format!("Cannot access {}: {error}", path.display()))?;
            if !metadata.is_file() {
                return Err(format!("Expected an image file, but got {}.", path.display()));
            }
            let extension = extension_for(&path);
            if !SUPPORTED_FORMATS.contains(&extension.as_str()) {
                return Err(format!(
                    "The source format '.{extension}' is not supported by SHIFT's built-in image engine yet."
                ));
            }
            Ok(path)
        })
        .collect()
}

pub fn convert_images(
    inputs: &[String],
    target_extension: &str,
) -> Result<Vec<ImageOperationResult>, String> {
    let target_ext = target_extension.trim_start_matches('.').to_ascii_lowercase();
    let format = target_format(&target_ext)?;
    let paths = validate_inputs(inputs)?;
    // Validate the whole selection before producing any outputs, so a mixed batch
    // cannot leave a partial set behind when one input is already the target type.
    for path in &paths {
        let source_ext = extension_for(path);
        let equivalent = matches!(
            (source_ext.as_str(), target_ext.as_str()),
            ("jpg", "jpeg") | ("jpeg", "jpg") | ("tif", "tiff") | ("tiff", "tif")
        );
        if source_ext == target_ext || equivalent {
            return Err(format!(
                "{} is already in the selected format.",
                path.file_name().unwrap_or_default().to_string_lossy()
            ));
        }
    }

    let mut results = Vec::with_capacity(paths.len());
    for path in paths {
        let decoded = decode(&path)?;
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let mut temporary = NamedTempFile::new_in(parent).map_err(|error| {
            format!(
                "Could not create a temporary output beside {}: {error}",
                path.display()
            )
        })?;
        let encode_result = if format == ImageFormat::Jpeg {
            // JPEG has no alpha channel; composite transparent images onto white.
            jpeg_compatible(&decoded).write_to(&mut temporary, format)
        } else {
            decoded.write_to(&mut temporary, format)
        };
        encode_result.map_err(|error| {
            format!(
                "Could not convert {} to {}: {error}",
                path.display(),
                target_ext
            )
        })?;
        results.push(finish_output(&path, &target_ext, "converted", temporary)?);
    }
    Ok(results)
}

pub fn compress_jpeg_images(
    inputs: &[String],
) -> Result<Vec<ImageOperationResult>, String> {
    let paths = validate_inputs(inputs)?;
    if paths
        .iter()
        .any(|path| !matches!(extension_for(path).as_str(), "jpg" | "jpeg"))
    {
        return Err(
            "Built-in lossy image compression currently supports JPEG files only. For PNG, WebP, BMP and TIFF, choose a different output format instead."
                .to_owned(),
        );
    }

    let mut results = Vec::with_capacity(paths.len());
    for path in paths {
        let decoded = decode(&path)?;
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let mut temporary = NamedTempFile::new_in(parent).map_err(|error| {
            format!(
                "Could not create a temporary output beside {}: {error}",
                path.display()
            )
        })?;
        JpegEncoder::new_with_quality(&mut temporary, 78)
            .encode_image(&decoded)
            .map_err(|error| {
                format!("Could not compress JPEG {}: {error}", path.display())
            })?;
        results.push(finish_output(
            &path,
            &extension_for(&path),
            "compressed",
            temporary,
        )?);
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};
    use tempfile::tempdir;

    #[test]
    fn converts_png_to_webp_without_overwriting_the_input() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("source.png");
        ImageBuffer::from_fn(24, 24, |x, y| Rgb([(x * 5) as u8, (y * 7) as u8, 90]))
            .save(&source)
            .unwrap();
        let results = convert_images(&[source.to_string_lossy().into_owned()], "webp").unwrap();
        assert_eq!(results.len(), 1);
        assert!(source.exists());
        assert!(Path::new(&results[0].output_path).exists());
        assert!(results[0].output_path.ends_with(".webp"));
    }

    #[test]
    fn converts_transparent_png_to_jpeg() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("alpha.png");
        ImageBuffer::from_pixel(8, 8, image::Rgba([255, 0, 0, 50]))
            .save(&source)
            .unwrap();
        let result = convert_images(&[source.to_string_lossy().into_owned()], "jpg").unwrap();
        assert!(source.exists());
        let opened = ImageReader::open(&result[0].output_path)
            .unwrap()
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();
        assert!(!matches!(opened, DynamicImage::ImageRgba8(_)));
        assert!(result[0].output_path.ends_with(".jpg"));
    }

    #[test]
    fn does_not_clobber_an_existing_conversion() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("source.png");
        ImageBuffer::from_pixel(4, 4, Rgb([12, 50, 200]))
            .save(&source)
            .unwrap();
        let first = convert_images(&[source.to_string_lossy().into_owned()], "bmp").unwrap();
        let second = convert_images(&[source.to_string_lossy().into_owned()], "bmp").unwrap();
        assert_ne!(first[0].output_path, second[0].output_path);
    }

    #[test]
    fn jpeg_compression_keeps_source_and_creates_new_valid_image() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("source.jpg");
        ImageBuffer::from_pixel(80, 80, Rgb([100, 120, 140]))
            .save(&source)
            .unwrap();
        let results = compress_jpeg_images(&[source.to_string_lossy().into_owned()]).unwrap();
        assert!(source.exists());
        assert!(Path::new(&results[0].output_path).exists());
        assert!(results[0].output_path.contains("compressed"));
    }
}
