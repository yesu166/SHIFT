use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use serde::Serialize;
use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Output},
};
use tempfile::{tempdir_in, NamedTempFile};
use walkdir::WalkDir;

const MAX_INPUTS: usize = 100;
const MAX_ARCHIVE_ENTRIES: usize = 100_000;
const MAX_ARCHIVE_UNPACKED_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_ARCHIVE_PATH_BYTES: usize = 32_000;
const MAX_ARCHIVE_PATH_DEPTH: usize = 64;
const MAX_DIAGNOSTIC: usize = 2_000;
const MEDIA_FORMATS: &[&str] = &[
    "mp3", "m4a", "wav", "flac", "ogg", "opus", "aiff", "wma",
    "mp4", "mov", "mkv", "webm", "avi", "wmv", "gif", "jpg", "jpeg", "png", "webp", "bmp", "tiff", "tif", "heic", "avif",
];
const OFFICE_FORMATS: &[&str] = &[
    "pdf", "docx", "odt", "rtf", "txt", "html", "epub", "pptx", "odp",
    "xlsx", "ods", "csv", "tsv", "jpg", "png",
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineStatus {
    pub ffmpeg: bool,
    pub libre_office: bool,
    pub qpdf: bool,
    pub poppler: bool,
    pub seven_zip: bool,
    pub inkscape: bool,
    pub built_in_images: bool,
    pub built_in_subtitles: bool,
    pub built_in_archives: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineOperationResult {
    pub input_path: String,
    pub output_path: String,
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub bytes_saved: i64,
    pub detail: String,
}


fn run_command(tool: &str, args: &[&str]) -> io::Result<Output> {
    let mut command = Command::new(tool);
    command.args(args);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    command.output()
}

fn run_command_strings(tool: &str, args: &[String]) -> io::Result<Output> {
    let mut command = Command::new(tool);
    command.args(args);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    command.output()
}

pub fn detect_engines() -> EngineStatus {
    EngineStatus {
        ffmpeg: tool_works(&["ffmpeg"], &["-version"]),
        libre_office: tool_works(&["soffice", "libreoffice"], &["--version"]),
        qpdf: tool_works(&["qpdf"], &["--version"]),
        poppler: tool_works(&["pdftoppm"], &["-v"]) && tool_works(&["pdftotext"], &["-v"]),
        seven_zip: tool_works(&["7z", "7zz", "7za"], &["-h"]),
        inkscape: tool_works(&["inkscape"], &["--version"]),
        built_in_images: true,
        built_in_subtitles: true,
        built_in_archives: true,
    }
}

fn tool_works(names: &[&str], args: &[&str]) -> bool {
    names.iter().any(|name| {
        run_command(name, args)
            .map(|output| output.status.success() || !output.stdout.is_empty() || !output.stderr.is_empty())
            .unwrap_or(false)
    })
}

fn find_tool(names: &[&str], args: &[&str]) -> Result<String, String> {
    for name in names {
        match run_command(name, args) {
            Ok(output) if output.status.success() || !output.stdout.is_empty() || !output.stderr.is_empty() => {
                return Ok((*name).to_owned());
            }
            _ => {}
        }
    }
    Err(format!(
        "Required local engine not found ({}). Install it, add it to PATH, and restart SHIFT.",
        names.join(" or ")
    ))
}

fn run_checked(tool: &str, args: &[String]) -> Result<Output, String> {
    let output = run_command_strings(tool, args)
        .map_err(|error| format!("Could not start {tool}: {error}. Check that the engine is installed and available on PATH."))?;
    if !output.status.success() {
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        let trimmed = diagnostic.trim();
        let excerpt: String = trimmed.chars().take(MAX_DIAGNOSTIC).collect();
        return Err(if excerpt.is_empty() {
            format!("{tool} exited with status {}.", output.status)
        } else {
            format!("{tool} failed ({}): {excerpt}", output.status)
        });
    }
    Ok(output)
}

fn ext(path: &Path) -> String {
    path.extension().and_then(|part| part.to_str()).unwrap_or("").to_ascii_lowercase()
}

fn stem(path: &Path) -> Result<String, String> {
    path.file_stem().and_then(|part| part.to_str()).map(str::to_owned)
        .ok_or_else(|| format!("Could not read the filename for {}.", path.display()))
}

fn validate_inputs(inputs: &[String]) -> Result<Vec<PathBuf>, String> {
    if inputs.is_empty() {
        return Err("Choose at least one input file.".into());
    }
    if inputs.len() > MAX_INPUTS {
        return Err(format!("Choose no more than {MAX_INPUTS} files per conversion job."));
    }
    inputs.iter().map(|raw| {
        let path = PathBuf::from(raw);
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("Cannot access {}: {error}", path.display()))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("Symbolic links are not supported as direct inputs: {}", path.display()));
        }
        if !metadata.is_file() {
            return Err(format!("This operation expects files, not folders: {}", path.display()));
        }
        Ok(path)
    }).collect()
}

fn validate_archive_inputs(inputs: &[String]) -> Result<Vec<PathBuf>, String> {
    if inputs.is_empty() { return Err("Choose at least one file or folder to archive.".into()); }
    if inputs.len() > MAX_INPUTS { return Err(format!("Choose no more than {MAX_INPUTS} paths per archive job.")); }
    inputs.iter().map(|raw| {
        let path = PathBuf::from(raw);
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("Cannot access {}: {error}", path.display()))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("Symbolic links are not supported as archive roots: {}", path.display()));
        }
        if !metadata.is_file() && !metadata.is_dir() {
            return Err(format!("Unsupported archive input: {}", path.display()));
        }
        Ok(path)
    }).collect()
}

fn unique_path(directory: &Path, base: &str, label: &str, extension: &str) -> Result<PathBuf, String> {
    fs::create_dir_all(directory)
        .map_err(|error| format!("Could not create output directory {}: {error}", directory.display()))?;
    let clean_ext = extension.trim_start_matches('.').to_ascii_lowercase();
    for index in 0_u32..100_000 {
        let suffix = if index == 0 { String::new() } else { format!(" {index}") };
        let candidate = directory.join(format!("{base} (SHIFT {label}{suffix}).{clean_ext}"));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err("Could not reserve a unique output filename.".into())
}

fn measured_result(input: &Path, output: &Path, detail: &str) -> Result<EngineOperationResult, String> {
    let input_bytes = fs::metadata(input)
        .map_err(|error| format!("Could not measure {}: {error}", input.display()))?.len();
    let output_bytes = fs::metadata(output)
        .map_err(|error| format!("Could not measure {}: {error}", output.display()))?.len();
    Ok(EngineOperationResult {
        input_path: input.to_string_lossy().into_owned(),
        output_path: output.to_string_lossy().into_owned(),
        input_bytes,
        output_bytes,
        bytes_saved: (input_bytes as i128 - output_bytes as i128)
            .clamp(i64::MIN as i128, i64::MAX as i128) as i64,
        detail: detail.to_owned(),
    })
}

fn commit_staged_file(
    input: &Path,
    staged: &Path,
    extension: &str,
    label: &str,
    detail: &str,
) -> Result<EngineOperationResult, String> {
    let parent = input.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    if !staged.is_file() {
        return Err(format!("The engine did not create its expected output: {}", staged.display()));
    }
    let staged_size = fs::metadata(staged).map_err(|error| format!("Could not measure staged output: {error}"))?.len();
    if staged_size == 0 {
        return Err("The conversion engine produced an empty output file.".into());
    }
    validate_generated_output(staged, extension)?;
    let final_path = unique_path(parent, &stem(input)?, label, extension)?;
    let mut temporary = NamedTempFile::new_in(parent)
        .map_err(|error| format!("Could not stage the final output beside {}: {error}", input.display()))?;
    let mut source = File::open(staged).map_err(|error| format!("Could not read generated output: {error}"))?;
    io::copy(&mut source, temporary.as_file_mut()).map_err(|error| format!("Could not copy generated output: {error}"))?;
    temporary.flush().map_err(|error| format!("Could not flush generated output: {error}"))?;
    temporary.persist_noclobber(&final_path).map_err(|error| {
        format!("Could not safely save output to {}: {}", final_path.display(), error.error)
    })?;
    measured_result(input, &final_path, detail)
}

fn validate_generated_output(path: &Path, extension: &str) -> Result<(), String> {
    let lower_ext = extension.to_ascii_lowercase();
    if matches!(lower_ext.as_str(), "jpg" | "jpeg" | "png" | "webp" | "bmp" | "tif" | "tiff") {
        image::ImageReader::open(path)
            .map_err(|error| format!("Could not open converted image: {error}"))?
            .with_guessed_format()
            .map_err(|error| format!("Could not identify converted image: {error}"))?
            .decode()
            .map_err(|error| format!("Converted image validation failed: {error}"))?;
    }
    if matches!(lower_ext.as_str(), "docx" | "pptx" | "xlsx" | "odt" | "ods" | "epub") {
        let required_part = match lower_ext.as_str() {
            "docx" => "word/document.xml",
            "pptx" => "ppt/presentation.xml",
            "xlsx" => "xl/workbook.xml",
            "epub" => "META-INF/container.xml",
            _ => "content.xml",
        };
        let file = File::open(path).map_err(|error| format!("Could not open generated office package: {error}"))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|error| format!("Generated office package is not a valid ZIP container: {error}"))?;
        let mut part = archive.by_name(required_part)
            .map_err(|error| format!("Generated {} is missing {required_part}: {error}", lower_ext.to_uppercase()))?;
        let mut sample = [0_u8; 512];
        part.read(&mut sample).map_err(|error| format!("Could not read {required_part}: {error}"))?;
    }
    if matches!(lower_ext.as_str(), "pdf") {
        let mut file = File::open(path).map_err(|error| format!("Could not open generated PDF: {error}"))?;
        let mut header = [0_u8; 1024];
        let count = file.read(&mut header).map_err(|error| format!("Could not validate generated PDF: {error}"))?;
        if !header[..count].windows(5).any(|window| window == b"%PDF-") {
            return Err("The converter output did not contain a valid PDF header.".into());
        }
    }
    Ok(())
}

fn stage_dir(input: &Path) -> Result<tempfile::TempDir, String> {
    let parent = input.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    tempdir_in(parent).map_err(|error| format!("Could not create temporary work directory: {error}"))
}

fn ensure_target(target: &str, formats: &[&str]) -> Result<String, String> {
    let clean = target.trim_start_matches('.').to_ascii_lowercase();
    if !formats.contains(&clean.as_str()) {
        return Err(format!("Unsupported target format '{target}'."));
    }
    Ok(clean)
}

fn run_ffmpeg(input: &Path, operation: &str, target: &str) -> Result<Vec<EngineOperationResult>, String> {
    let target = ensure_target(target, MEDIA_FORMATS)?;
    if operation != "media-compress" && (ext(input) == target || (target == "jpg" && ext(input) == "jpeg") || (target == "tif" && ext(input) == "tiff")) {
        return Err("The input is already in that format.".into());
    }
    let tool = find_tool(&["ffmpeg"], &["-version"])?;
    let work = stage_dir(input)?;
    let staged = work.path().join(format!("result.{target}"));
    let mut args = vec![
        "-nostdin".to_owned(), "-hide_banner".to_owned(), "-loglevel".to_owned(), "error".to_owned(),
        "-y".to_owned(), "-i".to_owned(), input.to_string_lossy().into_owned(),
    ];
    if operation == "extract-audio" {
        args.push("-vn".to_owned());
        if target == "mp3" {
            args.extend(["-codec:a".to_owned(), "libmp3lame".to_owned(), "-q:a".to_owned(), "2".to_owned()]);
        }
    } else if target == "gif" {
        args.extend(["-vf".to_owned(), "fps=12,scale=960:-1:flags=lanczos".to_owned()]);
    } else if operation == "media-compress" {
        match target.as_str() {
            "mp4" | "mov" | "mkv" => args.extend([
                "-c:v".to_owned(), "libx264".to_owned(), "-crf".to_owned(), "28".to_owned(),
                "-preset".to_owned(), "medium".to_owned(), "-c:a".to_owned(), "aac".to_owned(),
                "-b:a".to_owned(), "128k".to_owned(),
            ]),
            "webm" => args.extend([
                "-c:v".to_owned(), "libvpx-vp9".to_owned(), "-crf".to_owned(), "34".to_owned(),
                "-b:v".to_owned(), "0".to_owned(), "-c:a".to_owned(), "libopus".to_owned(),
                "-b:a".to_owned(), "96k".to_owned(),
            ]),
            "avi" => args.extend([
                "-c:v".to_owned(), "mpeg4".to_owned(), "-q:v".to_owned(), "5".to_owned(),
                "-c:a".to_owned(), "libmp3lame".to_owned(), "-b:a".to_owned(), "128k".to_owned(),
            ]),
            "wmv" => args.extend([
                "-c:v".to_owned(), "wmv2".to_owned(), "-b:v".to_owned(), "1200k".to_owned(),
                "-c:a".to_owned(), "wmav2".to_owned(), "-b:a".to_owned(), "128k".to_owned(),
            ]),
            "mp3" => args.extend(["-c:a".to_owned(), "libmp3lame".to_owned(), "-b:a".to_owned(), "128k".to_owned()]),
            "m4a" => args.extend(["-c:a".to_owned(), "aac".to_owned(), "-b:a".to_owned(), "128k".to_owned()]),
            "flac" => args.extend(["-c:a".to_owned(), "flac".to_owned(), "-compression_level".to_owned(), "12".to_owned()]),
            "ogg" => args.extend(["-c:a".to_owned(), "libvorbis".to_owned(), "-q:a".to_owned(), "4".to_owned()]),
            "opus" => args.extend(["-c:a".to_owned(), "libopus".to_owned(), "-b:a".to_owned(), "96k".to_owned()]),
            "wma" => args.extend(["-c:a".to_owned(), "wmav2".to_owned(), "-b:a".to_owned(), "128k".to_owned()]),
            _ => return Err(format!("Compression settings are not defined for {target}.")),
        }
    } else if matches!(target.as_str(), "mp3" | "m4a" | "wav" | "flac" | "ogg" | "opus" | "aiff" | "wma")
        && matches!(ext(input).as_str(), "mp4" | "mov" | "mkv" | "webm" | "avi" | "wmv")
    {
        args.push("-vn".to_owned());
    } else if matches!(target.as_str(), "jpg" | "jpeg" | "png" | "webp" | "bmp" | "tiff" | "tif")
        && matches!(ext(input).as_str(), "mp4" | "mov" | "mkv" | "webm" | "avi" | "wmv" | "gif")
    {
        args.extend(["-frames:v".to_owned(), "1".to_owned()]);
    }
    args.push(staged.to_string_lossy().into_owned());
    run_checked(&tool, &args)?;
    // Decode a bounded sample of the new file before publishing it. This catches
    // unsupported encoder outputs and broken headers without reading it all into RAM.
    run_checked(&tool, &[
        "-nostdin".to_owned(), "-hide_banner".to_owned(), "-loglevel".to_owned(), "error".to_owned(),
        "-i".to_owned(), staged.to_string_lossy().into_owned(),
        "-t".to_owned(), "1".to_owned(), "-f".to_owned(), "null".to_owned(), "-".to_owned(),
    ])?;
    let is_compression = operation == "media-compress";
    let label = if is_compression { "compressed" } else { "converted" };
    let detail = if is_compression {
        format!("FFmpeg · size-focused re-encode to {target.to_uppercase()} (lossy settings may reduce quality)")
    } else {
        format!("FFmpeg · {target.to_uppercase()}")
    };
    Ok(vec![commit_staged_file(input, &staged, &target, label, &detail)?])
}

fn run_inkscape(input: &Path, target: &str) -> Result<Vec<EngineOperationResult>, String> {
    if ext(input) != "svg" { return Err("Inkscape conversion expects an SVG input.".into()); }
    let target = ensure_target(target, &["png", "pdf"])?;
    let tool = find_tool(&["inkscape"], &["--version"])?;
    let work = stage_dir(input)?;
    let staged = work.path().join(format!("result.{target}"));
    let args = vec![
        format!("--export-filename={}", staged.to_string_lossy()),
        input.to_string_lossy().into_owned(),
    ];
    run_checked(&tool, &args)?;
    Ok(vec![commit_staged_file(input, &staged, &target, "converted", &format!("Inkscape · SVG to {target}",))?])
}

fn run_office(input: &Path, target: &str) -> Result<Vec<EngineOperationResult>, String> {
    let target = ensure_target(target, OFFICE_FORMATS)?;
    if ext(input) == target {
        return Err("The input is already in that format.".into());
    }
    let tool = find_tool(&["soffice", "libreoffice"], &["--version"])?;
    let work = stage_dir(input)?;
    let args = vec![
        "--headless".to_owned(), "--nologo".to_owned(), "--nodefault".to_owned(),
        "--nolockcheck".to_owned(), "--convert-to".to_owned(), target.clone(),
        "--outdir".to_owned(), work.path().to_string_lossy().into_owned(),
        input.to_string_lossy().into_owned(),
    ];
    run_checked(&tool, &args)?;
    if matches!(target.as_str(), "jpg" | "png") && matches!(ext(input).as_str(), "ppt" | "pptx" | "odp") {
        // Use the stable presentation -> PDF path, then render every page with Poppler.
        let poppler = find_tool(&["pdftoppm"], &["-v"])?;
        let pdf_args = vec![
            "--headless".to_owned(), "--nologo".to_owned(), "--nodefault".to_owned(),
            "--nolockcheck".to_owned(), "--convert-to".to_owned(), "pdf".to_owned(),
            "--outdir".to_owned(), work.path().to_string_lossy().into_owned(),
            input.to_string_lossy().into_owned(),
        ];
        run_checked(&tool, &pdf_args)?;
        let staged_pdf = work.path().join(format!("{}.pdf", stem(input)?));
        validate_generated_output(&staged_pdf, "pdf")?;
        let prefix = work.path().join("slide");
        let switch = if target == "jpg" { "-jpeg" } else { "-png" };
        run_checked(&poppler, &[
            switch.to_owned(), "-r".to_owned(), "300".to_owned(),
            staged_pdf.to_string_lossy().into_owned(), prefix.to_string_lossy().into_owned(),
        ])?;
        let mut staged_images: Vec<PathBuf> = fs::read_dir(work.path())
            .map_err(|error| format!("Could not inspect rendered slides: {error}"))?
            .filter_map(Result::ok).map(|entry| entry.path())
            .filter(|path| ext(path) == target && path.file_name().and_then(|name| name.to_str()).unwrap_or("").starts_with("slide-"))
            .collect();
        staged_images.sort();
        if staged_images.is_empty() {
            return Err(format!("Poppler did not render any {target} slides. Try exporting to PDF instead."));
        }
        return staged_images.iter().enumerate().map(|(index, staged)| {
            commit_staged_file(input, staged, &target, &format!("slide-{}", index + 1),
                &format!("LibreOffice + Poppler · slide {} of {}", index + 1, staged_images.len()))
        }).collect();
    }
    let staged = work.path().join(format!("{}.{}", stem(input)?, target));
    Ok(vec![commit_staged_file(input, &staged, &target, "converted", &format!("LibreOffice · {target.to_uppercase()}"))?])
}

fn pdf_optimize(input: &Path) -> Result<Vec<EngineOperationResult>, String> {
    if ext(input) != "pdf" { return Err("PDF optimization requires a PDF input.".into()); }
    let tool = find_tool(&["qpdf"], &["--version"])?;
    let work = stage_dir(input)?;
    let staged = work.path().join("optimized.pdf");
    let args = vec![
        "--object-streams=generate".to_owned(), "--recompress-flate".to_owned(),
        "--compression-level=9".to_owned(), input.to_string_lossy().into_owned(),
        staged.to_string_lossy().into_owned(),
    ];
    run_checked(&tool, &args)?;
    Ok(vec![commit_staged_file(input, &staged, "pdf", "optimized", "qpdf · PDF optimization")?])
}

fn pdf_merge(inputs: &[PathBuf]) -> Result<Vec<EngineOperationResult>, String> {
    if inputs.len() < 2 || inputs.iter().any(|path| ext(path) != "pdf") {
        return Err("Merge needs at least two PDF files.".into());
    }
    let tool = find_tool(&["qpdf"], &["--version"])?;
    let parent = inputs[0].parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    let work = tempdir_in(parent).map_err(|error| format!("Could not stage PDF merge: {error}"))?;
    let staged = work.path().join("merged.pdf");
    let mut args = vec!["--empty".to_owned(), "--pages".to_owned()];
    for input in inputs { args.push(input.to_string_lossy().into_owned()); }
    args.extend(["--".to_owned(), staged.to_string_lossy().into_owned()]);
    run_checked(&tool, &args)?;
    let input_bytes = inputs.iter().map(|path| fs::metadata(path).map(|m| m.len()).unwrap_or(0)).sum();
    validate_generated_output(&staged, "pdf")?;
    let final_path = unique_path(parent, "Merged PDF", "merged", "pdf")?;
    let mut temporary = NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    io::copy(&mut File::open(&staged).map_err(|error| error.to_string())?, temporary.as_file_mut())
        .map_err(|error| format!("Could not stage merged PDF: {error}"))?;
    temporary.flush().map_err(|error| error.to_string())?;
    temporary.persist_noclobber(&final_path).map_err(|error| error.error.to_string())?;
    let output_bytes = fs::metadata(&final_path).map_err(|error| error.to_string())?.len();
    Ok(vec![EngineOperationResult {
        input_path: inputs.iter().map(|p| p.to_string_lossy().into_owned()).collect::<Vec<_>>().join("; "),
        output_path: final_path.to_string_lossy().into_owned(),
        input_bytes, output_bytes,
        bytes_saved: (input_bytes as i128 - output_bytes as i128).clamp(i64::MIN as i128, i64::MAX as i128) as i64,
        detail: "qpdf · PDF merge".into(),
    }])
}

fn pdf_to_text(input: &Path, target: &str) -> Result<Vec<EngineOperationResult>, String> {
    let target = ensure_target(target, &["txt", "md"])?;
    let tool = find_tool(&["pdftotext"], &["-v"])?;
    let work = stage_dir(input)?;
    let staged = work.path().join(format!("text.{target}"));
    run_checked(&tool, &[
        "-layout".to_owned(), input.to_string_lossy().into_owned(),
        staged.to_string_lossy().into_owned(),
    ])?;
    Ok(vec![commit_staged_file(input, &staged, &target, "text", "Poppler · text extraction")?])
}

fn pdf_to_images(input: &Path, target: &str) -> Result<Vec<EngineOperationResult>, String> {
    let target = ensure_target(target, &["jpg", "png"])?;
    let tool = find_tool(&["pdftoppm"], &["-v"])?;
    let work = stage_dir(input)?;
    let prefix = work.path().join("page");
    let switch = if target == "jpg" { "-jpeg" } else { "-png" };
    run_checked(&tool, &[
        switch.to_owned(), "-r".to_owned(), "300".to_owned(),
        input.to_string_lossy().into_owned(), prefix.to_string_lossy().into_owned(),
    ])?;
    let mut pages: Vec<PathBuf> = fs::read_dir(work.path()).map_err(|error| error.to_string())?
        .filter_map(Result::ok).map(|entry| entry.path())
        .filter(|path| ext(path) == target && path.file_name().and_then(|n| n.to_str()).unwrap_or("").starts_with("page-"))
        .collect();
    pages.sort();
    if pages.is_empty() { return Err("Poppler did not produce any page images.".into()); }
    pages.iter().enumerate().map(|(index, page)| {
        let base = format!("{} page {}", stem(input)?, index + 1);
        let final_path = unique_path(input.parent().unwrap_or_else(|| Path::new(".")), &base, "converted", &target)?;
        let mut temporary = NamedTempFile::new_in(final_path.parent().unwrap_or_else(|| Path::new("."))).map_err(|e| e.to_string())?;
        io::copy(&mut File::open(page).map_err(|e| e.to_string())?, temporary.as_file_mut()).map_err(|e| e.to_string())?;
        temporary.flush().map_err(|e| e.to_string())?;
        temporary.persist_noclobber(&final_path).map_err(|e| e.error.to_string())?;
        measured_result(input, &final_path, &format!("Poppler · page {}", index + 1))
    }).collect()
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn text_to_html(input: &Path) -> Result<Vec<EngineOperationResult>, String> {
    if !matches!(ext(input).as_str(), "txt" | "md" | "markdown") {
        return Err("Text-to-HTML accepts TXT or Markdown input.".into());
    }
    let source = fs::read_to_string(input)
        .map_err(|error| format!("Could not read text input as UTF-8: {error}"))?;
    let title = escape_html(&stem(input)?);
    let mut html = format!(
        "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{title}</title><style>body{{max-width:900px;margin:3rem auto;padding:0 1.25rem;font:16px/1.65 system-ui,sans-serif;color:#222}}pre{{white-space:pre-wrap;background:#f4f4f4;padding:1rem;border-radius:.5rem}}li{{margin:.25rem 0}}</style></head><body>\n"
    );
    let mut in_code = false;
    let mut in_list = false;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            if in_list { html.push_str("</ul>\n"); in_list = false; }
            if in_code { html.push_str("</code></pre>\n"); }
            else { html.push_str("<pre><code>"); }
            in_code = !in_code;
            continue;
        }
        if in_code {
            html.push_str(&escape_html(line));
            html.push('\n');
            continue;
        }
        if trimmed.is_empty() {
            if in_list { html.push_str("</ul>\n"); in_list = false; }
            continue;
        }
        let heading_level = trimmed.chars().take_while(|ch| *ch == '#').count();
        if heading_level > 0 && heading_level <= 6 && trimmed.chars().nth(heading_level) == Some(' ') {
            if in_list { html.push_str("</ul>\n"); in_list = false; }
            let content = escape_html(trimmed[heading_level + 1..].trim());
            html.push_str(&format!("<h{heading_level}>{content}</h{heading_level}>\n"));
        } else if let Some(item) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
            if !in_list { html.push_str("<ul>\n"); in_list = true; }
            html.push_str(&format!("<li>{}</li>\n", escape_html(item)));
        } else {
            if in_list { html.push_str("</ul>\n"); in_list = false; }
            html.push_str(&format!("<p>{}</p>\n", escape_html(trimmed)));
        }
    }
    if in_code { html.push_str("</code></pre>\n"); }
    if in_list { html.push_str("</ul>\n"); }
    html.push_str("</body></html>\n");

    let parent = input.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    let final_path = unique_path(parent, &stem(input)?, "converted", "html")?;
    let mut temporary = NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    temporary.write_all(html.as_bytes()).map_err(|error| error.to_string())?;
    temporary.flush().map_err(|error| error.to_string())?;
    temporary.persist_noclobber(&final_path).map_err(|error| error.error.to_string())?;
    Ok(vec![measured_result(input, &final_path, "SHIFT · local text/Markdown to HTML")?])
}

fn subtitle_transform(input: &Path, target: &str) -> Result<Vec<EngineOperationResult>, String> {
    let source = fs::read_to_string(input)
        .map_err(|error| format!("Subtitle file must be valid UTF-8 text: {error}"))?;
    let source = source.trim_start_matches('\u{feff}').replace("\r\n", "\n").replace('\r', "\n");
    let source_ext = ext(input);
    let output_ext = ensure_target(target, &["srt", "vtt", "txt"])?;
    let converted = match (source_ext.as_str(), output_ext.as_str()) {
        ("srt", "vtt") => {
            let lines = source.lines()
                .filter(|line| !line.trim().chars().all(|c| c.is_ascii_digit()) || line.trim().is_empty())
                .map(|line| if line.contains("-->") { line.replace(',', ".") } else { line.to_owned() })
                .collect::<Vec<_>>().join("\n");
            format!("WEBVTT\n\n{}\n", lines.trim())
        }
        ("vtt", "srt") => {
            let mut cues = Vec::new();
            for block in source.split("\n\n") {
                let lines: Vec<&str> = block.lines().map(str::trim_end).collect();
                if lines.is_empty() { continue; }
                let joined = lines.join("\n");
                if joined.trim().is_empty()
                    || joined.trim().eq_ignore_ascii_case("WEBVTT")
                    || joined.starts_with("NOTE")
                    || joined.starts_with("STYLE")
                    || joined.starts_with("REGION")
                {
                    continue;
                }
                let Some(time_index) = lines.iter().position(|line| line.contains("-->")) else {
                    continue;
                };
                let time_parts: Vec<&str> = lines[time_index].split_whitespace().take(3).collect();
                if time_parts.len() < 3 { continue; }
                let time_line = time_parts.join(" ").replace('.', ",");
                let text = lines.iter().skip(time_index + 1).filter(|line| !line.is_empty()).copied().collect::<Vec<_>>();
                if text.is_empty() { continue; }
                cues.push(format!("{}\n{}\n{}", cues.len() + 1, time_line, text.join("\n")));
            }
            if cues.is_empty() {
                return Err("No valid WebVTT cues were found in the input.".into());
            }
            cues.join("\n\n") + "\n"
        }
        ("srt" | "vtt", "txt") => source.lines().filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty()
                && !trimmed.eq_ignore_ascii_case("WEBVTT")
                && !trimmed.chars().all(|c| c.is_ascii_digit())
                && !trimmed.contains("-->")
                && !trimmed.starts_with("NOTE")
                && !trimmed.starts_with("STYLE")
                && !trimmed.starts_with("REGION")
        }).collect::<Vec<_>>().join("\n"),
        _ => return Err("Subtitle conversion supports SRT ↔ VTT and SRT/VTT to TXT.".into()),
    };
    let parent = input.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    let base = stem(input)?;
    let final_path = unique_path(parent, &base, "converted", &output_ext)?;
    let mut temporary = NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    temporary.write_all(converted.as_bytes()).map_err(|error| error.to_string())?;
    temporary.flush().map_err(|error| error.to_string())?;
    temporary.persist_noclobber(&final_path).map_err(|error| error.error.to_string())?;
    Ok(vec![measured_result(input, &final_path, "SHIFT · subtitle conversion")?])
}

fn create_tar(inputs: &[PathBuf], output_dir: &Path, gzipped: bool) -> Result<Vec<EngineOperationResult>, String> {
    if inputs.is_empty() { return Err("Choose files or folders to archive.".into()); }
    fs::create_dir_all(output_dir).map_err(|error| error.to_string())?;
    let canonical_output = fs::canonicalize(output_dir)
        .map_err(|error| format!("Could not resolve archive destination: {error}"))?;
    let mut canonical_roots: Vec<PathBuf> = Vec::new();
    for input in inputs {
        let canonical = fs::canonicalize(input)
            .map_err(|error| format!("Could not resolve {}: {error}", input.display()))?;
        if canonical_output.starts_with(&canonical) {
            return Err("The selected input folder contains SHIFT's output destination. Choose a narrower folder to avoid archiving the archive into itself.".into());
        }
        if canonical_roots.iter().any(|root| canonical.starts_with(root) || root.starts_with(&canonical)) {
            return Err("Selected archive paths overlap. Do not select a folder together with files inside it.".into());
        }
        canonical_roots.push(canonical);
    }
    let archive_name = if gzipped { "SHIFT Archive.tar.gz" } else { "SHIFT Archive.tar" };
    let output_path = unique_path(output_dir, archive_name.trim_end_matches(if gzipped { ".tar.gz" } else { ".tar" }), "archive", if gzipped { "tar.gz" } else { "tar" })?;
    let temporary = NamedTempFile::new_in(output_dir).map_err(|error| error.to_string())?;
    let mut input_bytes = 0_u64;
    for path in inputs {
        input_bytes = input_bytes.saturating_add(
            sum_input_bytes(path).map_err(|error| format!("Could not measure {}: {error}", path.display()))?
        );
    }
    if gzipped {
        let encoder = GzEncoder::new(temporary, Compression::best());
        let mut builder = tar::Builder::new(encoder);
        append_tar_paths(&mut builder, inputs)?;
        let encoder = builder.into_inner().map_err(|error| format!("Could not finish TAR.GZ: {error}"))?;
        let temporary = encoder.finish().map_err(|error| format!("Could not finish GZIP compression: {error}"))?;
        verify_tar_gz(temporary.path())?;
        temporary.persist_noclobber(&output_path).map_err(|error| error.error.to_string())?;
    } else {
        let mut builder = tar::Builder::new(temporary);
        append_tar_paths(&mut builder, inputs)?;
        let temporary = builder.into_inner().map_err(|error| format!("Could not finish TAR archive: {error}"))?;
        verify_tar(temporary.path())?;
        temporary.persist_noclobber(&output_path).map_err(|error| error.error.to_string())?;
    }
    let output_bytes = fs::metadata(&output_path).map_err(|error| error.to_string())?.len();
    Ok(vec![EngineOperationResult {
        input_path: inputs.iter().map(|path| path.to_string_lossy().into_owned()).collect::<Vec<_>>().join("; "),
        output_path: output_path.to_string_lossy().into_owned(), input_bytes, output_bytes,
        bytes_saved: (input_bytes as i128 - output_bytes as i128).clamp(i64::MIN as i128, i64::MAX as i128) as i64,
        detail: if gzipped { "SHIFT · validated TAR.GZ archive".into() } else { "SHIFT · validated TAR archive".into() },
    }])
}

fn append_tar_paths<W: Write>(builder: &mut tar::Builder<W>, inputs: &[PathBuf]) -> Result<(), String> {
    for input in inputs {
        let meta = fs::symlink_metadata(input).map_err(|error| format!("Could not inspect {}: {error}", input.display()))?;
        if meta.file_type().is_symlink() { return Err(format!("Symbolic links are not supported in archives: {}", input.display())); }
        if meta.is_file() {
            let name = input.file_name().ok_or_else(|| "Input has no filename.".to_string())?;
            builder.append_path_with_name(input, name).map_err(|error| format!("Could not add {} to TAR: {error}", input.display()))?;
        } else if meta.is_dir() {
            let root = input.file_name().ok_or_else(|| "Input folder has no name.".to_string())?;
            for entry in WalkDir::new(input).follow_links(false) {
                let entry = entry.map_err(|error| format!("Could not enumerate {}: {error}", input.display()))?;
                if entry.path_is_symlink() { continue; }
                let relative = entry.path().strip_prefix(input).map_err(|error| error.to_string())?;
                let archive_path = Path::new(root).join(relative);
                if entry.file_type().is_dir() {
                    builder.append_dir(&archive_path, entry.path()).map_err(|error| error.to_string())?;
                } else if entry.file_type().is_file() {
                    builder.append_path_with_name(entry.path(), archive_path).map_err(|error| error.to_string())?;
                }
            }
        } else {
            return Err(format!("Unsupported archive input: {}", input.display()));
        }
    }
    Ok(())
}

fn verify_tar(path: &Path) -> Result<(), String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    let mut archive = tar::Archive::new(file);
    for entry in archive.entries().map_err(|error| format!("Invalid TAR output: {error}"))? {
        entry.map_err(|error| format!("TAR validation failed: {error}"))?;
    }
    Ok(())
}

fn verify_tar_gz(path: &Path) -> Result<(), String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    verify_tar_reader(GzDecoder::new(file))
}

fn verify_tar_reader<R: Read>(reader: R) -> Result<(), String> {
    let mut archive = tar::Archive::new(reader);
    for entry in archive.entries().map_err(|error| format!("Invalid TAR.GZ output: {error}"))? {
        entry.map_err(|error| format!("TAR.GZ validation failed: {error}"))?;
    }
    Ok(())
}

fn sum_input_bytes(path: &Path) -> io::Result<u64> {
    let metadata = fs::metadata(path)?;
    if metadata.is_file() { return Ok(metadata.len()); }
    let mut total = 0_u64;
    for entry in WalkDir::new(path).follow_links(false) {
        let entry = entry.map_err(io::Error::other)?;
        if entry.file_type().is_file() { total = total.saturating_add(entry.metadata()?.len()); }
    }
    Ok(total)
}

fn create_gzip(input: &Path, output_dir: &Path) -> Result<Vec<EngineOperationResult>, String> {
    if !input.is_file() { return Err("GZIP creation requires one file.".into()); }
    fs::create_dir_all(output_dir).map_err(|error| error.to_string())?;
    let parent = output_dir;
    let target = unique_path(parent, &stem(input)?, "compressed", "gz")?;
    let temporary = NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    let mut encoder = GzEncoder::new(temporary, Compression::best());
    let input_bytes = fs::metadata(input).map_err(|error| error.to_string())?.len();
    io::copy(&mut File::open(input).map_err(|error| error.to_string())?, &mut encoder)
        .map_err(|error| format!("Could not compress input with GZIP: {error}"))?;
    let temporary = encoder.finish().map_err(|error| format!("Could not finish GZIP output: {error}"))?;
    let mut decoder = GzDecoder::new(File::open(temporary.path()).map_err(|error| error.to_string())?);
    let mut sink = io::sink();
    io::copy(&mut decoder, &mut sink).map_err(|error| format!("GZIP output validation failed: {error}"))?;
    temporary.persist_noclobber(&target).map_err(|error| error.error.to_string())?;
    let output_bytes = fs::metadata(&target).map_err(|error| error.to_string())?.len();
    Ok(vec![EngineOperationResult {
        input_path: input.to_string_lossy().into_owned(), output_path: target.to_string_lossy().into_owned(),
        input_bytes, output_bytes,
        bytes_saved: (input_bytes as i128 - output_bytes as i128).clamp(i64::MIN as i128, i64::MAX as i128) as i64,
        detail: "SHIFT · validated GZIP file".into(),
    }])
}

fn unpack_tar_to_directory<R: Read>(reader: R, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|error| format!("Could not create extraction folder: {error}"))?;
    let mut archive = tar::Archive::new(reader);
    let entries = archive.entries().map_err(|error| format!("Invalid TAR stream: {error}"))?;
    let root = fs::canonicalize(destination).map_err(|error| error.to_string())?;
    let mut count = 0_usize;
    let mut expanded = 0_u64;
    let mut names = std::collections::HashSet::new();

    for item in entries {
        let mut entry = item.map_err(|error| format!("Could not read TAR entry: {error}"))?;
        count += 1;
        if count > MAX_ARCHIVE_ENTRIES {
            return Err(format!("TAR contains more than {MAX_ARCHIVE_ENTRIES} entries; extraction refused."));
        }
        let raw_path = entry.path().map_err(|error| format!("TAR entry has an invalid path: {error}"))?.into_owned();
        let raw_text = raw_path.to_string_lossy().replace('\\', "/");
        if raw_text.len() > MAX_ARCHIVE_PATH_BYTES || raw_text.contains(':') {
            return Err("TAR contains a path that is too long or unsafe.".into());
        }
        let mut relative = PathBuf::new();
        for component in raw_path.components() {
            match component {
                std::path::Component::Normal(part) => relative.push(part),
                std::path::Component::CurDir => {},
                _ => return Err("TAR contains an absolute or parent-relative path; extraction refused.".into()),
            }
        }
        if relative.as_os_str().is_empty() { continue; }
        if relative.components().count() > MAX_ARCHIVE_PATH_DEPTH {
            return Err("TAR contains an excessively deep path; extraction refused.".into());
        }
        let normalized = relative.to_string_lossy().replace('\\', "/");
        if !names.insert(normalized) {
            return Err("TAR contains duplicate output paths; extraction refused.".into());
        }
        let entry_type = entry.header().entry_type();
        if entry_type.is_symlink() || entry_type.is_hard_link() {
            return Err("TAR contains a link entry; extraction refused.".into());
        }
        if !entry_type.is_file() && !entry_type.is_dir() {
            return Err("TAR contains a special file type; extraction refused.".into());
        }
        let size = entry.header().size()
            .map_err(|error| format!("Could not read TAR entry size: {error}"))?;
        if entry_type.is_dir() {
            if size != 0 {
                return Err("TAR directory entry has unexpected content.".into());
            }
        } else {
            expanded = expanded.saturating_add(size);
            if expanded > MAX_ARCHIVE_UNPACKED_BYTES {
                return Err("TAR contents exceed SHIFT's 2 GiB safe extraction limit.".into());
            }
        }
        let output = destination.join(&relative);
        if !output.starts_with(&root) {
            return Err("TAR output path escaped its extraction folder; extraction refused.".into());
        }
        if entry_type.is_dir() {
            fs::create_dir_all(&output).map_err(|error| format!("Could not create TAR folder: {error}"))?;
        } else {
            let parent = output.parent().ok_or_else(|| "TAR file has no parent folder.".to_string())?;
            fs::create_dir_all(parent).map_err(|error| format!("Could not create TAR parent folder: {error}"))?;
            let mut target = fs::OpenOptions::new().write(true).create_new(true).open(&output)
                .map_err(|error| format!("Could not create extracted file {} safely: {error}", output.display()))?;
            io::copy(&mut entry, &mut target)
                .map_err(|error| format!("Could not extract {}: {error}", output.display()))?;
            target.flush().map_err(|error| format!("Could not flush extracted file: {error}"))?;
        }
    }
    Ok(())
}

fn tar_or_gzip_to_zip(input: &Path, output_dir: &Path) -> Result<Vec<EngineOperationResult>, String> {
    let extension = ext(input);
    let name_stem = stem(input)?;
    let compressed_tar = extension == "tgz" || (extension == "gz" && name_stem.to_ascii_lowercase().ends_with(".tar"));
    if !matches!(extension.as_str(), "tar" | "tgz" | "gz") {
        return Err("Built-in TAR/GZIP conversion accepts TAR, TGZ, TAR.GZ or GZIP input.".into());
    }
    let work = tempfile::tempdir().map_err(|error| format!("Could not create a private archive work folder: {error}"))?;
    let folder_name = if compressed_tar {
        name_stem.strip_suffix(".tar").or_else(|| name_stem.strip_suffix(".TAR")).unwrap_or(&name_stem).to_owned()
    } else {
        name_stem.clone()
    };
    let extraction_root = work.path().join(if folder_name.is_empty() { "extracted" } else { &folder_name });
    let zip_source: PathBuf;
    if extension == "tar" || compressed_tar {
        let reader: Box<dyn Read> = if extension == "tar" {
            Box::new(File::open(input).map_err(|error| format!("Could not open TAR input: {error}"))?)
        } else {
            Box::new(GzDecoder::new(File::open(input).map_err(|error| format!("Could not open GZIP input: {error}"))?))
        };
        unpack_tar_to_directory(reader, &extraction_root)?;
        zip_source = extraction_root.clone();
    } else {
        // A plain .gz is a compressed single file, not necessarily a TAR stream.
        fs::create_dir_all(&extraction_root).map_err(|error| error.to_string())?;
        let decoded_name = if name_stem.is_empty() { "decompressed" } else { &name_stem };
        let output = extraction_root.join(decoded_name);
        let decoder = GzDecoder::new(File::open(input).map_err(|error| format!("Could not open GZIP input: {error}"))?);
        let mut limited = decoder.take(MAX_ARCHIVE_UNPACKED_BYTES + 1);
        let mut target = fs::OpenOptions::new().write(true).create_new(true).open(&output)
            .map_err(|error| format!("Could not create GZIP output safely: {error}"))?;
        io::copy(&mut limited, &mut target).map_err(|error| format!("GZIP decompression failed: {error}"))?;
        target.flush().map_err(|error| format!("Could not flush GZIP output: {error}"))?;
        if target.metadata().map_err(|error| error.to_string())?.len() > MAX_ARCHIVE_UNPACKED_BYTES {
            return Err("GZIP contents exceed SHIFT's 2 GiB safe extraction limit.".into());
        }
        zip_source = output;
    }

    let zip_result = crate::zip_service::create_zip_in_dir(
        &[zip_source.to_string_lossy().into_owned()],
        output_dir,
        Some(&format!("{} converted", name_stem)),
    )?;
    Ok(vec![EngineOperationResult {
        input_path: input.to_string_lossy().into_owned(),
        output_path: zip_result.output_path,
        input_bytes: fs::metadata(input).map_err(|error| error.to_string())?.len(),
        output_bytes: zip_result.archive_bytes,
        bytes_saved: zip_result.bytes_saved,
        detail: "SHIFT · validated TAR/GZIP to ZIP conversion".into(),
    }])
}

fn rar_to_zip(input: &Path, output_dir: &Path) -> Result<Vec<EngineOperationResult>, String> {
    if !matches!(ext(input).as_str(), "rar" | "zip" | "7z") {
        return Err("Archive conversion accepts RAR, ZIP or 7Z input.".into());
    }
    let tool = find_tool(&["7z", "7zz", "7za"], &["-h"])?;
    let listing = run_checked(&tool, &[
        "l".to_owned(), "-slt".to_owned(), input.to_string_lossy().into_owned(),
    ])?;
    let listing_text = String::from_utf8_lossy(&listing.stdout);
    let mut in_entries = false;
    let mut listed_entries = 0_usize;
    let mut unpacked_bytes = 0_u64;
    for line in listing_text.lines() {
        if line.trim() == "----------" { in_entries = true; continue; }
        if !in_entries { continue; }
        if let Some(value) = line.strip_prefix("Path = ") {
            listed_entries += 1;
            if listed_entries > MAX_ARCHIVE_ENTRIES {
                return Err(format!("The archive contains more than {MAX_ARCHIVE_ENTRIES} entries and was refused."));
            }
            if value.len() > MAX_ARCHIVE_PATH_BYTES {
                return Err("The archive contains an excessively long path and was refused.".into());
            }
            let normalized = value.replace('\\', "/");
            let path = Path::new(&normalized);
            if path.is_absolute() || normalized.starts_with('/') || normalized.contains(':')
                || path.components().any(|component| matches!(component, std::path::Component::ParentDir))
                || path.components().count() > MAX_ARCHIVE_PATH_DEPTH
            {
                return Err("The archive contains an unsafe absolute, parent-relative or excessively deep path, so SHIFT refused to extract it.".into());
            }
        }
        if let Some(value) = line.strip_prefix("Size = ") {
            if let Ok(size) = value.trim().parse::<u64>() {
                unpacked_bytes = unpacked_bytes.saturating_add(size);
                if unpacked_bytes > MAX_ARCHIVE_UNPACKED_BYTES {
                    return Err("Archive contents exceed SHIFT's 2 GiB safe extraction limit.".into());
                }
            }
        }
        if line.starts_with("Symbolic Link = ") || line.starts_with("Hard Link = ") {
            return Err("The archive contains a link entry; SHIFT refuses to extract links from untrusted archives.".into());
        }
    }
    if !in_entries || listed_entries == 0 {
        return Err("SHIFT could not verify the archive's entry listing, so extraction was refused.".into());
    }

    // Extract to an OS temp directory outside the archive destination so the ZIP
    // traversal guard does not exclude the staged tree from the output archive.
    let work = tempfile::tempdir().map_err(|error| format!("Could not create private extraction folder: {error}"))?;
    let extract_root = work.path().join(stem(input)?);
    fs::create_dir_all(&extract_root).map_err(|error| format!("Could not prepare the extraction folder: {error}"))?;
    run_checked(&tool, &[
        "x".to_owned(), "-y".to_owned(), "-bd".to_owned(),
        format!("-o{}", extract_root.to_string_lossy()), input.to_string_lossy().into_owned(),
    ])?;

    for entry in WalkDir::new(&extract_root).follow_links(false) {
        let entry = entry.map_err(|error| format!("Could not inspect extracted content: {error}"))?;
        if entry.path_is_symlink() {
            return Err("Extracted content contains a link; operation stopped.".into());
        }
    }
    let zip_result = crate::zip_service::create_zip_in_dir(
        &[extract_root.to_string_lossy().into_owned()],
        output_dir,
        Some(&format!("{} converted", stem(input)?)),
    )?;
    Ok(vec![EngineOperationResult {
        input_path: input.to_string_lossy().into_owned(),
        output_path: zip_result.output_path,
        input_bytes: fs::metadata(input).map_err(|error| error.to_string())?.len(),
        output_bytes: zip_result.archive_bytes,
        bytes_saved: zip_result.bytes_saved,
        detail: "7-Zip · archive converted to ZIP (validated by ZIP engine)".into(),
    }])
}

pub fn execute_operation(
    inputs: &[String],
    operation_id: &str,
    target_format: Option<&str>,
    output_dir: &Path,
) -> Result<Vec<EngineOperationResult>, String> {
    let paths = if operation_id == "create-tar" || operation_id == "create-targz" {
        validate_archive_inputs(inputs)?
    } else {
        validate_inputs(inputs)?
    };
    if operation_id == "text-to-html" {
        let mut results = Vec::with_capacity(paths.len());
        for path in &paths { results.extend(text_to_html(path)?); }
        return Ok(results);
    }
    if operation_id == "pdf-merge" {
        return pdf_merge(&paths);
    }
    if operation_id == "pdf-optimize" {
        let mut results = Vec::with_capacity(paths.len());
        for path in &paths { results.extend(pdf_optimize(path)?); }
        return Ok(results);
    }
    if operation_id == "create-tar" || operation_id == "create-targz" {
        return create_tar(&paths, output_dir, operation_id == "create-targz");
    }
    if operation_id == "create-gzip" {
        if paths.len() != 1 { return Err("GZIP creation takes exactly one input file.".into()); }
        return create_gzip(&paths[0], output_dir);
    }
    if operation_id == "rar-to-zip" {
        if paths.len() != 1 { return Err("Convert one archive at a time.".into()); }
        return rar_to_zip(&paths[0], output_dir);
    }
    if operation_id == "tar-to-zip" {
        if paths.len() != 1 { return Err("Convert one TAR/GZIP archive at a time.".into()); }
        return tar_or_gzip_to_zip(&paths[0], output_dir);
    }
    let target = target_format.ok_or_else(|| "Choose an output format.".to_string())?;
    let mut output = Vec::new();
    for input in &paths {
        match operation_id {
            "media-convert" | "media-compress" | "extract-audio" => {
                output.extend(run_ffmpeg(input, operation_id, target)?);
            }
            "svg-convert" => output.extend(run_inkscape(input, target)?),
            "office-convert" | "image-to-pdf" => output.extend(run_office(input, target)?),
            "pdf-optimize" => output.extend(pdf_optimize(input)?),
            "pdf-to-text" => output.extend(pdf_to_text(input, target)?),
            "pdf-to-image" => output.extend(pdf_to_images(input, target)?),
            "subtitle-convert" => output.extend(subtitle_transform(input, target)?),
            _ => return Err(format!("Unknown operation '{operation_id}'.")),
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn markdown_html_escapes_untrusted_html() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("notes.md");
        fs::write(&source, "# Heading\n\n<script>alert(1)</script>\n\n- item").unwrap();
        let results = text_to_html(&source).unwrap();
        let html = fs::read_to_string(&results[0].output_path).unwrap();
        assert!(html.contains("<h1>Heading</h1>"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(html.contains("<li>item</li>"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn srt_to_vtt_converts_timestamps_and_adds_header() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("captions.srt");
        fs::write(&source, "1\n00:00:01,200 --> 00:00:02,300\nHi!\n").unwrap();
        let result = subtitle_transform(&source, "vtt").unwrap();
        let output = fs::read_to_string(&result[0].output_path).unwrap();
        assert!(output.starts_with("WEBVTT"));
        assert!(output.contains("00:00:01.200 --> 00:00:02.300"));
    }

    #[test]
    fn creates_readable_tar_archive() {
        let dir = tempdir().unwrap();
        let source_dir = dir.path().join("input");
        let output_dir = dir.path().join("out");
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("hello.txt"), "SHIFT archive test").unwrap();
        let inputs = vec![source_dir.clone()];
        let result = create_tar(&inputs, &output_dir, false).unwrap();
        assert!(Path::new(&result[0].output_path).exists());
        verify_tar(Path::new(&result[0].output_path)).unwrap();
    }

    #[test]
    fn creates_readable_gzip_output() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("hello.txt");
        let output_dir = dir.path().join("out");
        fs::write(&source, "SHIFT gzip test").unwrap();
        let result = create_gzip(&source, &output_dir).unwrap();
        let file = File::open(&result[0].output_path).unwrap();
        let mut decoder = GzDecoder::new(file);
        let mut text = String::new();
        decoder.read_to_string(&mut text).unwrap();
        assert_eq!(text, "SHIFT gzip test");
    }

    #[test]
    fn converts_tar_tree_to_zip_and_keeps_nested_paths() {
        let dir = tempdir().unwrap();
        let source_dir = dir.path().join("project");
        let archive_dir = dir.path().join("source-archives");
        let output_dir = dir.path().join("outputs");
        fs::create_dir_all(source_dir.join("src")).unwrap();
        fs::write(source_dir.join("src").join("main.rs"), "fn main() {}").unwrap();
        let tar = create_tar(&[source_dir], &archive_dir, false).unwrap();
        let result = tar_or_gzip_to_zip(Path::new(&tar[0].output_path), &output_dir).unwrap();
        let file = File::open(&result[0].output_path).unwrap();
        let mut zip = zip::ZipArchive::new(file).unwrap();
        let mut found = false;
        for index in 0..zip.len() {
            let entry = zip.by_index(index).unwrap();
            if entry.name().ends_with("project/src/main.rs") {
                found = true;
            }
        }
        assert!(found, "the converted ZIP should retain nested paths");
    }

    #[test]
    fn converts_gzip_single_file_to_zip_and_preserves_bytes() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("message.txt");
        let source_archive_dir = dir.path().join("source-archives");
        let output_dir = dir.path().join("outputs");
        fs::write(&source, "gzip round-trip").unwrap();
        let gzip = create_gzip(&source, &source_archive_dir).unwrap();
        let result = tar_or_gzip_to_zip(Path::new(&gzip[0].output_path), &output_dir).unwrap();
        let file = File::open(&result[0].output_path).unwrap();
        let mut zip = zip::ZipArchive::new(file).unwrap();
        assert_eq!(zip.len(), 1);
        let mut body = String::new();
        zip.by_index(0).unwrap().read_to_string(&mut body).unwrap();
        assert_eq!(body, "gzip round-trip");
    }

    #[test]
    fn vtt_to_srt_removes_header_and_changes_timestamps() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("captions.vtt");
        fs::write(&source, "WEBVTT\n\n00:00:01.200 --> 00:00:02.300\nHello\n").unwrap();
        let result = subtitle_transform(&source, "srt").unwrap();
        let output = fs::read_to_string(&result[0].output_path).unwrap();
        assert!(!output.starts_with("WEBVTT"));
        assert!(output.contains("00:00:01,200 --> 00:00:02,300"));
    }
}
