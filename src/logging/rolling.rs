use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy)]
pub struct LogRotationConfig {
    pub max_file_size: u64,
    pub max_archive_files: usize,
}

impl Default for LogRotationConfig {
    fn default() -> Self {
        Self {
            max_file_size: 5 * 1024 * 1024,
            max_archive_files: 3,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RollingError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("max archive files reached for {0}")]
    ArchiveLimitReached(String),
}

fn current_log_size(path: &Path) -> io::Result<u64> {
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.len())
}

fn resolved_log_path(path: &Path) -> (PathBuf, String) {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("log") => {
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("log");
            let parent = path.parent().map(|p| p.to_path_buf()).unwrap_or_default();
            (parent.join(format!("{stem}.1.log")), format!("{stem}.1.log"))
        }
        _ => (path.to_path_buf(), path.to_string_lossy().to_string()),
    }
}

pub fn roll_file_if_needed(path: &Path, config: LogRotationConfig) -> Result<(), RollingError> {
    if !path.exists() {
        return Ok(());
    }

    let size = current_log_size(path)?;
    if size >= config.max_file_size {
        perform_roll(path, config)?;
    }

    Ok(())
}

fn perform_roll(path: &Path, config: LogRotationConfig) -> Result<(), RollingError> {
    let (target_path, _label) = resolved_log_path(path);

    if target_path.exists() {
        prune_archives(&target_path, config.max_archive_files.saturating_sub(1))?;
    }

    std::fs::rename(path, target_path)?;
    prune_archives(path, config.max_archive_files)?;

    Ok(())
}

fn prune_archives(path: &Path, max_archives: usize) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("log");
    let parent = path.parent().unwrap_or(Path::new("."));

    let mut ordered = Vec::new();
    for entry in std::fs::read_dir(parent)? {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            if name == format!("{stem}.log") {
                continue;
            }
            if name.starts_with(&format!("{stem}.")) && name.ends_with(".log") {
                ordered.push((entry.metadata().ok().map(|m| m.len()), entry.path()));
            }
        }
    }

    ordered.sort_unstable_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    for (_, to_remove) in ordered.into_iter().skip(max_archives) {
        let _ = std::fs::remove_file(to_remove);
    }

    Ok(())
}

pub fn write_to_rolling_file(
    path: &Path,
    config: LogRotationConfig,
    bytes: &[u8],
) -> Result<(), RollingError> {
    roll_file_if_needed(path, config)?;

    let mut file = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(bytes)?;
    file.flush()?;

    Ok(())
}
