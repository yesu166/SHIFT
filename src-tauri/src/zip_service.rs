use serde::Serialize;
use std::{
    collections::HashSet,
    fs::{self, File},
    io::{self, copy},
    path::{Component, Path, PathBuf},
};
use tempfile::NamedTempFile;
use walkdir::{DirEntry, WalkDir};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

const MAX_INPUT_ROOTS: usize = 1_000;
const MAX_ARCHIVE_ENTRIES: usize = 100_000;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZipResult {
    pub output_path: String,
    pub input_bytes: u64,
    pub archive_bytes: u64,
    pub bytes_saved: i64,
    pub files_archived: usize,
}

#[derive(Debug)]
struct Entry {
    source: PathBuf,
    archive_name: String,
    is_directory: bool,
}

pub fn create_zip_in_dir(
    inputs: &[String],
    output_dir: &Path,
    archive_name: Option<&str>,
) -> Result<ZipResult, String> {
    if inputs.is_empty() {
        return Err("Choose at least one file or folder.".to_owned());
    }
    if inputs.len() > MAX_INPUT_ROOTS {
        return Err(format!("Choose no more than {MAX_INPUT_ROOTS} input paths at once."));
    }

    fs::create_dir_all(output_dir)
        .map_err(|error| format!("Could not create the SHIFT Archives folder: {error}"))?;
    let canonical_output = fs::canonicalize(output_dir)
        .map_err(|error| format!("Could not resolve the archive destination: {error}"))?;

    let (entries, input_bytes) = gather_entries(inputs, &canonical_output)?;
    if entries.is_empty() {
        return Err("No regular files or folders were found in the selection.".to_owned());
    }
    if entries.len() > MAX_ARCHIVE_ENTRIES {
        return Err(format!("The selection contains more than {MAX_ARCHIVE_ENTRIES} archive entries. Split it into smaller jobs."));
    }

    let base_name = sanitize_archive_name(archive_name)?;
    let temporary = NamedTempFile::new_in(&canonical_output)
        .map_err(|error| format!("Could not create a temporary archive in the destination: {error}"))?;
    let temporary_path = temporary.path().to_path_buf();

    {
        let mut writer = ZipWriter::new(
            temporary.reopen()
                .map_err(|error| format!("Could not open the temporary archive: {error}"))?,
        );
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .compression_level(Some(6));

        for entry in &entries {
            if entry.is_directory {
                writer.add_directory(&entry.archive_name, options)
                    .map_err(|error| format!("Could not write a folder entry: {error}"))?;
            } else {
                writer.start_file(&entry.archive_name, options)
                    .map_err(|error| format!("Could not write {} into the archive: {error}", entry.archive_name))?;
                let mut source = File::open(&entry.source)
                    .map_err(|error| format!("Could not read {}: {error}", entry.source.display()))?;
                copy(&mut source, &mut writer)
                    .map_err(|error| format!("Could not compress {}: {error}", entry.source.display()))?;
            }
        }

        writer.finish()
            .map_err(|error| format!("Could not finalize the ZIP structure: {error}"))?;
    }

    validate_archive(&temporary_path, entries.len())?;
    let archive_bytes = fs::metadata(&temporary_path)
        .map_err(|error| format!("Could not measure the completed archive: {error}"))?
        .len();
    let final_path = choose_unique_path(&canonical_output, &base_name);

    temporary.persist_noclobber(&final_path).map_err(|error| {
        format!(
            "The archive was created but could not be safely saved to {}. The target may already exist: {}",
            final_path.display(),
            error.error
        )
    })?;

    let bytes_saved = (input_bytes as i128 - archive_bytes as i128)
        .clamp(i64::MIN as i128, i64::MAX as i128) as i64;

    Ok(ZipResult {
        output_path: final_path.to_string_lossy().into_owned(),
        input_bytes,
        archive_bytes,
        bytes_saved,
        files_archived: entries.len(),
    })
}

fn sanitize_archive_name(input: Option<&str>) -> Result<String, String> {
    let raw = input.unwrap_or("SHIFT Archive").trim();
    if raw.is_empty() {
        return Ok("SHIFT Archive".to_owned());
    }
    let stem = if raw.to_ascii_lowercase().ends_with(".zip") { &raw[..raw.len() - 4] } else { raw };
    let stem = stem.trim();
    if stem.is_empty() || stem == "." || stem == ".." {
        return Err("Enter a valid archive name.".to_owned());
    }
    if stem.chars().any(|ch| {
        ch.is_control() || matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')
    }) || stem.ends_with('.') || stem.ends_with(' ') {
        return Err("Use a simple filename without slashes, reserved characters, or a trailing dot/space.".to_owned());
    }

    let reserved = stem.split('.').next().unwrap_or(stem).to_ascii_uppercase();
    let reserved_device = matches!(
        reserved.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" |
        "COM1" | "COM2" | "COM3" | "COM4" | "COM5" | "COM6" | "COM7" | "COM8" | "COM9" |
        "LPT1" | "LPT2" | "LPT3" | "LPT4" | "LPT5" | "LPT6" | "LPT7" | "LPT8" | "LPT9"
    );
    if reserved_device {
        return Err("That name is reserved by Windows. Choose another archive name.".to_owned());
    }
    Ok(stem.to_owned())
}

fn choose_unique_path(output_dir: &Path, base_name: &str) -> PathBuf {
    let first = output_dir.join(format!("{base_name}.zip"));
    if !first.exists() {
        return first;
    }
    for index in 1_u32..=99_999 {
        let candidate = output_dir.join(format!("{base_name} ({index}).zip"));
        if !candidate.exists() {
            return candidate;
        }
    }
    output_dir.join(format!("{base_name}-{}.zip", std::process::id()))
}

fn gather_entries(inputs: &[String], output_dir: &Path) -> Result<(Vec<Entry>, u64), String> {
    let mut entries = Vec::new();
    let mut archived_names = HashSet::new();
    let mut canonical_roots = HashSet::new();
    let mut input_bytes: u64 = 0;

    for raw in inputs {
        let provided = PathBuf::from(raw);
        let provided_metadata = fs::symlink_metadata(&provided)
            .map_err(|error| format!("Cannot access selected path {}: {error}", provided.display()))?;
        if provided_metadata.file_type().is_symlink() {
            return Err(format!("Symbolic links cannot be added as roots: {}", provided.display()));
        }

        let canonical = fs::canonicalize(&provided)
            .map_err(|error| format!("Cannot resolve selected path {}: {error}", provided.display()))?;
        if canonical_roots.iter().any(|root: &PathBuf| {
            canonical.starts_with(root) || root.starts_with(&canonical)
        }) {
            return Err(format!(
                "Selected paths overlap: {}. Choose each file once and do not select a folder together with items inside it.",
                provided.display()
            ));
        }
        canonical_roots.insert(canonical.clone());
        if canonical == output_dir {
            return Err("The SHIFT Archives folder cannot be archived into itself.".to_owned());
        }

        let metadata = fs::metadata(&canonical)
            .map_err(|error| format!("Cannot inspect selected path {}: {error}", provided.display()))?;
        let parent = canonical.parent()
            .ok_or_else(|| format!("The selected path has no parent: {}", provided.display()))?;

        if metadata.is_dir() {
            let walker = WalkDir::new(&canonical)
                .follow_links(false)
                .sort_by_file_name()
                .into_iter()
                .filter_entry(|entry| !is_output_entry(entry, output_dir));
            for result in walker {
                let item = result.map_err(|error| format!("Could not enumerate a selected folder: {error}"))?;
                let file_type = item.file_type();
                if file_type.is_symlink() || (!file_type.is_dir() && !file_type.is_file()) { continue; }
                add_entry(item.path(), parent, file_type.is_dir(), &mut entries, &mut archived_names, &mut input_bytes)?;
                if entries.len() > MAX_ARCHIVE_ENTRIES {
                    return Err(format!("The selection contains more than {MAX_ARCHIVE_ENTRIES} archive entries. Split it into smaller jobs."));
                }
            }
        } else if metadata.is_file() {
            add_entry(&canonical, parent, false, &mut entries, &mut archived_names, &mut input_bytes)?;
        } else {
            return Err(format!("Unsupported filesystem item: {}", provided.display()));
        }
    }
    Ok((entries, input_bytes))
}

fn is_output_entry(entry: &DirEntry, output_dir: &Path) -> bool {
    entry.path() == output_dir || entry.path().starts_with(output_dir)
}

fn add_entry(
    source: &Path,
    relative_to: &Path,
    is_directory: bool,
    entries: &mut Vec<Entry>,
    names: &mut HashSet<String>,
    input_bytes: &mut u64,
) -> Result<(), String> {
    let relative = source.strip_prefix(relative_to)
        .map_err(|_| format!("Could not create a safe archive path for {}", source.display()))?;
    if relative.components().any(|component| !matches!(component, Component::Normal(_))) {
        return Err(format!("Unsafe path rejected: {}", source.display()));
    }
    let archive_name = relative.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>().join("/");
    if archive_name.is_empty() {
        return Err("An empty archive entry name was rejected.".to_owned());
    }
    if !names.insert(archive_name.clone()) {
        return Err(format!("Two selected items map to the same archive path: {archive_name}. Remove overlapping selections and try again."));
    }

    if !is_directory {
        let size = fs::metadata(source)
            .map_err(|error| format!("Could not measure {}: {error}", source.display()))?.len();
        *input_bytes = input_bytes.checked_add(size)
            .ok_or_else(|| "The selected input size is too large to measure safely.".to_owned())?;
    }
    entries.push(Entry {
        source: source.to_path_buf(),
        archive_name: if is_directory { format!("{archive_name}/") } else { archive_name },
        is_directory,
    });
    Ok(())
}

fn validate_archive(path: &Path, expected_entries: usize) -> Result<(), String> {
    let file = File::open(path).map_err(|error| format!("Could not reopen the ZIP for validation: {error}"))?;
    let mut archive = ZipArchive::new(file).map_err(|error| format!("The ZIP archive could not be parsed: {error}"))?;
    if archive.len() != expected_entries {
        return Err(format!("ZIP validation failed: expected {expected_entries} entries but found {}.", archive.len()));
    }
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)
            .map_err(|error| format!("ZIP entry {index} could not be reopened: {error}"))?;
        if !entry.is_dir() {
            io::copy(&mut entry, &mut io::sink())
                .map_err(|error| format!("ZIP entry validation failed: {error}"))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn write_file(path: &Path, body: &[u8]) {
        if let Some(parent) = path.parent() { fs::create_dir_all(parent).unwrap(); }
        fs::write(path, body).unwrap();
    }

    #[test]
    fn creates_and_validates_a_standard_deflated_archive() {
        let temp = tempfile::tempdir().unwrap();
        let source_a = temp.path().join("alpha.txt");
        let source_b = temp.path().join("beta.txt");
        let output = temp.path().join("output");
        write_file(&source_a, b"alpha alpha alpha alpha alpha");
        write_file(&source_b, b"beta beta beta beta beta");

        let result = create_zip_in_dir(
            &[source_a.to_string_lossy().into_owned(), source_b.to_string_lossy().into_owned()],
            &output, Some("test-bundle"),
        ).unwrap();

        assert_eq!(result.files_archived, 2);
        assert_eq!(result.input_bytes, 53);
        assert!(Path::new(&result.output_path).exists());
        let mut archive = ZipArchive::new(File::open(&result.output_path).unwrap()).unwrap();
        assert_eq!(archive.len(), 2);
        let mut alpha = String::new();
        archive.by_name("alpha.txt").unwrap().read_to_string(&mut alpha).unwrap();
        assert_eq!(alpha, "alpha alpha alpha alpha alpha");
        let mut beta = String::new();
        archive.by_name("beta.txt").unwrap().read_to_string(&mut beta).unwrap();
        assert_eq!(beta, "beta beta beta beta beta");
    }

    #[test]
    fn preserves_nested_directories_and_empty_folders() {
        let temp = tempfile::tempdir().unwrap();
        let folder = temp.path().join("project");
        write_file(&folder.join("src").join("main.rs"), b"fn main() {}");
        fs::create_dir_all(folder.join("empty")).unwrap();
        let result = create_zip_in_dir(&[folder.to_string_lossy().into_owned()], &temp.path().join("output"), Some("project")).unwrap();
        let mut archive = ZipArchive::new(File::open(result.output_path).unwrap()).unwrap();
        assert!(archive.by_name("project/").is_ok());
        assert!(archive.by_name("project/src/main.rs").is_ok());
        assert!(archive.by_name("project/empty/").is_ok());
    }

    #[test]
    fn resolves_existing_name_collisions_without_overwriting() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("small.txt");
        let output = temp.path().join("output");
        write_file(&source, b"safe");
        fs::create_dir_all(&output).unwrap();
        write_file(&output.join("bundle.zip"), b"do not overwrite");
        let result = create_zip_in_dir(&[source.to_string_lossy().into_owned()], &output, Some("bundle")).unwrap();
        assert!(result.output_path.ends_with("bundle (1).zip"));
        assert_eq!(fs::read(output.join("bundle.zip")).unwrap(), b"do not overwrite");
    }

    #[test]
    fn rejects_empty_inputs_and_unsafe_archive_names() {
        let temp = tempfile::tempdir().unwrap();
        assert!(create_zip_in_dir(&[], temp.path(), None).is_err());
        let source = temp.path().join("source.txt");
        write_file(&source, b"no path traversal");
        assert!(create_zip_in_dir(&[source.to_string_lossy().into_owned()], &temp.path().join("out"), Some("../escaped")).is_err());
        assert!(!temp.path().join("escaped.zip").exists());
    }

    #[test]
    fn rejects_overlapping_inputs_that_would_create_duplicate_entries() {
        let temp = tempfile::tempdir().unwrap();
        let folder = temp.path().join("project");
        let child = folder.join("nested.txt");
        write_file(&child, b"overlap");
        let output = temp.path().join("out");
        let result = create_zip_in_dir(&[folder.to_string_lossy().into_owned(), child.to_string_lossy().into_owned()], &output, None);
        assert!(result.is_err());
        assert!(!output.join("SHIFT Archive.zip").exists());
    }


    #[cfg(windows)]
    #[test]
    fn generated_zip_extracts_with_the_independent_windows_expand_archive_cmdlet() {
        use std::process::Command;

        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("résumé 文件.txt");
        let original = b"Independent Windows ZIP interoperability check.\n";
        write_file(&source, original);
        let output = temp.path().join("archives");
        let result = create_zip_in_dir(
            &[source.to_string_lossy().into_owned()],
            &output,
            Some("interoperability-check"),
        )
        .unwrap();

        let destination = temp.path().join("extracted");
        let script = temp.path().join("extract.ps1");
        fs::write(
            &script,
            "param([string]$Archive, [string]$Destination)\n\
             $ErrorActionPreference = 'Stop'\n\
             Expand-Archive -LiteralPath $Archive -DestinationPath $Destination\n",
        )
        .unwrap();

        let status = Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
            ])
            .arg(&script)
            .arg(&result.output_path)
            .arg(&destination)
            .status()
            .expect("Windows PowerShell must be available on the Windows CI runner");

        assert!(status.success(), "Windows Expand-Archive rejected the generated ZIP");
        assert_eq!(fs::read(destination.join("résumé 文件.txt")).unwrap(), original);
    }
}
