//! Core directory analysis functionality

use crate::cli::Args;
use crate::scanner::{DirectoryScanner, ScanResults};
use crate::duplicates::DuplicateFinder;
use crate::utils::{FileTypeClassifier, SizeBreakdown};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Main analyzer that orchestrates the analysis process
pub struct DirectoryAnalyzer {
    args: Args,
    scanner: DirectoryScanner,
    duplicate_finder: Option<DuplicateFinder>,
    classifier: FileTypeClassifier,
}

/// Complete analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResults {
    pub scan_info: ScanInfo,
    pub size_breakdown: SizeBreakdown,
    pub file_type_distribution: HashMap<String, TypeStats>,
    pub largest_files: Vec<FileInfo>,
    pub largest_directories: Vec<DirectoryInfo>,
    pub duplicate_groups: Option<Vec<DuplicateGroup>>,
    pub statistics: Statistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanInfo {
    pub path: PathBuf,
    pub timestamp: DateTime<Utc>,
    pub depth_limit: usize,
    pub total_files: u64,
    pub total_directories: u64,
    pub total_size: u64,
    pub scan_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeStats {
    pub count: u64,
    pub total_size: u64,
    pub average_size: u64,
    pub largest_file: Option<FileInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub file_type: String,
    pub modified: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryInfo {
    pub path: PathBuf,
    pub size: u64,
    pub file_count: u64,
    pub subdirectory_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub hash: String,
    pub file_size: u64,
    pub files: Vec<PathBuf>,
    pub wasted_space: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    pub files_per_second: f64,
    pub bytes_per_second: u64,
    pub memory_usage_mb: f64,
    pub duplicate_files: u64,
    pub wasted_space: u64,
    pub compression_ratio: f64,
}

impl DirectoryAnalyzer {
    /// Create a new analyzer with the given configuration
    pub fn new(args: Args) -> Result<Self> {
        let scanner = DirectoryScanner::new(&args)?;
        let duplicate_finder = if args.find_duplicates {
            Some(DuplicateFinder::new(args.min_duplicate_size, args.threads))
        } else {
            None
        };
        let classifier = FileTypeClassifier::new();

        Ok(Self {
            args,
            scanner,
            duplicate_finder,
            classifier,
        })
    }

    /// Perform comprehensive directory analysis
    pub async fn analyze(&mut self) -> Result<AnalysisResults> {
        let start_time = std::time::Instant::now();
        
        // Phase 1: Scan directory structure
        if !self.args.quiet {
            println!("🔍 Scanning directory structure...");
        }
        
        let scan_results = self.scanner.scan().await?;
        
        // Phase 2: Analyze file types and sizes
        if !self.args.quiet {
            println!("📊 Analyzing file types and sizes...");
        }
        
        let (size_breakdown, file_type_distribution, largest_files, largest_directories) = 
            self.analyze_files_and_directories(&scan_results).await?;

        // Phase 3: Find duplicates if requested
        let duplicate_groups = if let Some(ref mut finder) = self.duplicate_finder {
            if !self.args.quiet {
                println!("🔎 Detecting duplicate files...");
            }
            Some(finder.find_duplicates(&scan_results.files).await?)
        } else {
            None
        };

        let scan_duration = start_time.elapsed();
        
        // Calculate statistics
        let statistics = self.calculate_statistics(&scan_results, &duplicate_groups, scan_duration);

        let results = AnalysisResults {
            scan_info: ScanInfo {
                path: self.args.path.clone(),
                timestamp: Utc::now(),
                depth_limit: self.args.max_depth,
                total_files: scan_results.total_files,
                total_directories: scan_results.total_directories,
                total_size: scan_results.total_size,
                scan_duration_ms: scan_duration.as_millis() as u64,
            },
            size_breakdown,
            file_type_distribution,
            largest_files,
            largest_directories,
            duplicate_groups,
            statistics,
        };

        Ok(results)
    }

    async fn analyze_files_and_directories(&self, scan_results: &ScanResults) -> Result<(
        SizeBreakdown,
        HashMap<String, TypeStats>,
        Vec<FileInfo>,
        Vec<DirectoryInfo>
    )> {
        let mut file_type_distribution: HashMap<String, TypeStats> = HashMap::new();
        let mut largest_files = Vec::new();
        let mut largest_directories = Vec::new();

        // Analyze files
        for file_entry in &scan_results.files {
            let file_type = self.classifier.classify(&file_entry.path);
            let file_info = FileInfo {
                path: file_entry.path.clone(),
                size: file_entry.size,
                file_type: file_type.clone(),
                modified: file_entry.modified,
            };

            // Update file type statistics
            let type_stats = file_type_distribution.entry(file_type).or_insert(TypeStats {
                count: 0,
                total_size: 0,
                average_size: 0,
                largest_file: None,
            });

            type_stats.count += 1;
            type_stats.total_size += file_entry.size;
            type_stats.average_size = type_stats.total_size / type_stats.count;

            if type_stats.largest_file.as_ref().map(|f| f.size).unwrap_or(0) < file_entry.size {
                type_stats.largest_file = Some(file_info.clone());
            }

            largest_files.push(file_info);
        }

        // Sort and limit largest files
        largest_files.sort_by(|a, b| b.size.cmp(&a.size));
        largest_files.truncate(self.args.top_count);

        // Analyze directories
        for dir_entry in &scan_results.directories {
            largest_directories.push(DirectoryInfo {
                path: dir_entry.path.clone(),
                size: dir_entry.total_size,
                file_count: dir_entry.file_count,
                subdirectory_count: dir_entry.subdirectory_count,
            });
        }

        // Sort and limit largest directories
        largest_directories.sort_by(|a, b| b.size.cmp(&a.size));
        largest_directories.truncate(self.args.top_count);

        let size_breakdown = SizeBreakdown::from_scan_results(scan_results);

        Ok((size_breakdown, file_type_distribution, largest_files, largest_directories))
    }

    fn calculate_statistics(
        &self,
        scan_results: &ScanResults,
        duplicate_groups: &Option<Vec<DuplicateGroup>>,
        duration: std::time::Duration,
    ) -> Statistics {
        let duration_secs = duration.as_secs_f64();
        let files_per_second = scan_results.total_files as f64 / duration_secs;
        let bytes_per_second = (scan_results.total_size as f64 / duration_secs) as u64;

        let (duplicate_files, wasted_space) = if let Some(groups) = duplicate_groups {
            let duplicate_files = groups.iter().map(|g| g.files.len() as u64).sum::<u64>();
            let wasted_space = groups.iter().map(|g| g.wasted_space).sum();
            (duplicate_files, wasted_space)
        } else {
            (0, 0)
        };

        let compression_ratio = if scan_results.total_size > 0 {
            (scan_results.total_size - wasted_space) as f64 / scan_results.total_size as f64
        } else {
            1.0
        };

        Statistics {
            files_per_second,
            bytes_per_second,
            memory_usage_mb: self.estimate_memory_usage(),
            duplicate_files,
            wasted_space,
            compression_ratio,
        }
    }

    fn estimate_memory_usage(&self) -> f64 {
        // Rough estimation based on system info
        // In a real implementation, you might use a proper memory profiling crate
        50.0 // Placeholder: 50MB estimated
    }
}
