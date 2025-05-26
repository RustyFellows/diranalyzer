//! Duplicate file detection using hash comparison

use anyhow::Result;
use dashmap::DashMap;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::sync::Arc;

use crate::analyzer::DuplicateGroup;
use crate::scanner::FileEntry;

/// Duplicate file finder using SHA-256 hashing
pub struct DuplicateFinder {
    min_size: u64,
    thread_count: usize,
}

impl DuplicateFinder {
    /// Create a new duplicate finder
    pub fn new(min_size: u64, thread_count: Option<usize>) -> Self {
        let thread_count = thread_count.unwrap_or_else(num_cpus::get);
        
        Self {
            min_size,
            thread_count,
        }
    }

    /// Find duplicate files in the given file list
    pub async fn find_duplicates(&self, files: &[FileEntry]) -> Result<Vec<DuplicateGroup>> {
        // Filter files by minimum size
        let candidates: Vec<&FileEntry> = files
            .iter()
            .filter(|file| file.size >= self.min_size && !file.is_symlink)
            .collect();

        if candidates.is_empty() {
            return Ok(Vec::new());
        }

        // Group files by size first (optimization)
        let mut size_groups: HashMap<u64, Vec<&FileEntry>> = HashMap::new();
        for file in candidates {
            size_groups.entry(file.size).or_default().push(file);
        }

        // Only process size groups with multiple files
        let potential_duplicates: Vec<&FileEntry> = size_groups
            .into_values()
            .filter(|group| group.len() > 1)
            .flatten()
            .collect();

        if potential_duplicates.is_empty() {
            return Ok(Vec::new());
        }

        // Set up progress bar
        let progress_bar = ProgressBar::new(potential_duplicates.len() as u64);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} files ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );
        progress_bar.set_message("Hashing files...");

        // Configure rayon thread pool
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.thread_count)
            .build()?;

        // Hash files in parallel
        let hash_map: Arc<DashMap<String, Vec<PathBuf>>> = Arc::new(DashMap::new());
        let progress_bar = Arc::new(progress_bar);

        pool.install(|| {
            potential_duplicates
                .par_iter()
                .for_each(|file| {
                    if let Ok(hash) = calculate_file_hash(&file.path) {
                        hash_map.entry(hash).or_default().push(file.path.clone());
                    }
                    progress_bar.inc(1);
                });
        });

        progress_bar.finish_with_message("Hashing complete!");

        // Convert to duplicate groups
        let mut duplicate_groups = Vec::new();
        
        for entry in hash_map.iter() {
            let (hash, file_paths) = (entry.key(), entry.value());
            
            if file_paths.len() > 1 {
                // Get file size from the first file
                let file_size = files
                    .iter()
                    .find(|f| f.path == file_paths[0])
                    .map(|f| f.size)
                    .unwrap_or(0);

                let wasted_space = file_size * (file_paths.len() as u64 - 1);

                duplicate_groups.push(DuplicateGroup {
                    hash: hash.clone(),
                    file_size,
                    files: file_paths.clone(),
                    wasted_space,
                });
            }
        }

        // Sort by wasted space (descending)
        duplicate_groups.sort_by(|a, b| b.wasted_space.cmp(&a.wasted_space));

        Ok(duplicate_groups)
    }
}

/// Calculate SHA-256 hash of a file
fn calculate_file_hash(path: &PathBuf) -> Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192]; // 8KB buffer

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

// External dependency for CPU count detection
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    }
}
