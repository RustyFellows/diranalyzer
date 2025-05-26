//! File system scanning functionality

use crate::cli::Args;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::{DirEntry, WalkDir};

/// Directory scanner that traverses the file system
pub struct DirectoryScanner {
    args: Args,
    exclude_patterns: Vec<Regex>,
    progress_bar: Option<ProgressBar>,
}

/// Results from scanning the directory structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResults {
    pub files: Vec<FileEntry>,
    pub directories: Vec<DirectoryEntry>,
    pub total_files: u64,
    pub total_directories: u64,
    pub total_size: u64,
    pub errors: Vec<ScanError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
    pub modified: Option<DateTime<Utc>>,
    pub is_symlink: bool,
    pub depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryEntry {
    pub path: PathBuf,
    pub total_size: u64,
    pub file_count: u64,
    pub subdirectory_count: u64,
    pub depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanError {
    pub path: PathBuf,
    pub error: String,
    pub error_type: ErrorType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    PermissionDenied,
    FileNotFound,
    IoError,
    Other,
}

impl DirectoryScanner {
    /// Create a new scanner with the given configuration
    pub fn new(args: &Args) -> Result<Self> {
        let exclude_patterns = args.exclude_patterns
            .iter()
            .map(|pattern| Regex::new(pattern))
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to compile exclude patterns")?;

        let progress_bar = if !args.quiet {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                    .template("{spinner:.green} {msg} [{elapsed_precise}] {pos} files")
                    .unwrap()
            );
            pb.set_message("Scanning...");
            Some(pb)
        } else {
            None
        };

        Ok(Self {
            args: args.clone(),
            exclude_patterns,
            progress_bar,
        })
    }

    /// Scan the directory structure
    pub async fn scan(&mut self) -> Result<ScanResults> {
        let mut files = Vec::new();
        let mut directories = HashMap::new();
        let mut errors = Vec::new();
        let mut total_size = 0u64;

        let walker = WalkDir::new(&self.args.path)
            .max_depth(self.args.max_depth)
            .follow_links(self.args.follow_links);

        for entry in walker {
            match entry {
                Ok(entry) => {
                    if let Some(ref pb) = self.progress_bar {
                        pb.inc(1);
                        if files.len() % 100 == 0 {
                            pb.set_message(format!("Scanning... {} files found", files.len()));
                        }
                    }

                    if self.should_exclude(&entry) {
                        continue;
                    }

                    if entry.file_type().is_file() {
                        if let Ok(file_entry) = self.process_file_entry(&entry) {
                            total_size += file_entry.size;
                            files.push(file_entry);
                        } else {
                            errors.push(ScanError {
                                path: entry.path().to_path_buf(),
                                error: "Failed to process file".to_string(),
                                error_type: ErrorType::IoError,
                            });
                        }
                    } else if entry.file_type().is_dir() {
                        self.process_directory_entry(&entry, &mut directories);
                    }
                }
                Err(error) => {
                    let error_type = if error.to_string().contains("Permission denied") {
                        ErrorType::PermissionDenied
                    } else {
                        ErrorType::Other
                    };

                    errors.push(ScanError {
                        path: error.path().unwrap_or_else(|| Path::new("unknown")).to_path_buf(),
                        error: error.to_string(),
                        error_type,
                    });
                }
            }
        }

        if let Some(ref pb) = self.progress_bar {
            pb.finish_with_message(format!("Scan complete! {} files, {} directories", 
                files.len(), directories.len()));
        }

        // Calculate directory sizes and convert to vector
        let directories = self.calculate_directory_sizes(&files, directories);

        Ok(ScanResults {
            total_files: files.len() as u64,
            total_directories: directories.len() as u64,
            total_size,
            files,
            directories,
            errors,
        })
    }

    fn should_exclude(&self, entry: &DirEntry) -> bool {
        let path_str = entry.path().to_string_lossy();
        
        // Check if hidden and hidden files are disabled
        if !self.args.show_hidden && entry.file_name().to_string_lossy().starts_with('.') {
            return true;
        }

        // Check exclude patterns
        for pattern in &self.exclude_patterns {
            if pattern.is_match(&path_str) {
                return true;
            }
        }

        false
    }

    fn process_file_entry(&self, entry: &DirEntry) -> Result<FileEntry> {
        let metadata = entry.metadata()?;
        let modified = metadata.modified()
            .ok()
            .and_then(|time| {
                time.duration_since(SystemTime::UNIX_EPOCH)
                    .ok()
                    .map(|duration| DateTime::from_timestamp(duration.as_secs() as i64, 0))
                    .flatten()
            });

        Ok(FileEntry {
            path: entry.path().to_path_buf(),
            size: metadata.len(),
            modified,
            is_symlink: metadata.file_type().is_symlink(),
            depth: entry.depth(),
        })
    }

    fn process_directory_entry(
        &self,
        entry: &DirEntry,
        directories: &mut HashMap<PathBuf, DirectoryEntry>,
    ) {
        directories.insert(
            entry.path().to_path_buf(),
            DirectoryEntry {
                path: entry.path().to_path_buf(),
                total_size: 0, // Will be calculated later
                file_count: 0,
                subdirectory_count: 0,
                depth: entry.depth(),
            },
        );
    }

    fn calculate_directory_sizes(
        &self,
        files: &[FileEntry],
        mut directories: HashMap<PathBuf, DirectoryEntry>,
    ) -> Vec<DirectoryEntry> {
        // Calculate sizes and counts for each directory
        for file in files {
            let mut current_path = file.path.parent();
            
            while let Some(dir_path) = current_path {
                if let Some(dir_entry) = directories.get_mut(dir_path) {
                    dir_entry.total_size += file.size;
                    if dir_path == file.path.parent().unwrap_or_else(|| Path::new("")) {
                        dir_entry.file_count += 1;
                    }
                }
                current_path = dir_path.parent();
            }
        }

        // Calculate subdirectory counts
        let dir_paths: Vec<PathBuf> = directories.keys().cloned().collect();
        for dir_path in &dir_paths {
            let subdirs = dir_paths.iter()
                .filter(|other_path| {
                    other_path.parent() == Some(dir_path) && *other_path != dir_path
                })
                .count();
            
            if let Some(dir_entry) = directories.get_mut(dir_path) {
                dir_entry.subdirectory_count = subdirs as u64;
            }
        }

        directories.into_values().collect()
    }
}
