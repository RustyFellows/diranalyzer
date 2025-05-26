//! Utility functions and helper types

use crate::scanner::ScanResults;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// File type classifier for organizing files by category
pub struct FileTypeClassifier {
    type_map: HashMap<String, String>,
}

impl FileTypeClassifier {
    /// Create a new file type classifier
    pub fn new() -> Self {
        let mut type_map = HashMap::new();
        
        // Documents
        let documents = vec!["pdf", "doc", "docx", "txt", "rtf", "odt", "pages"];
        for ext in documents {
            type_map.insert(ext.to_string(), "Documents".to_string());
        }
        
        // Images
        let images = vec!["jpg", "jpeg", "png", "gif", "bmp", "svg", "tiff", "webp", "ico"];
        for ext in images {
            type_map.insert(ext.to_string(), "Images".to_string());
        }
        
        // Videos
        let videos = vec!["mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v"];
        for ext in videos {
            type_map.insert(ext.to_string(), "Videos".to_string());
        }
        
        // Audio
        let audio = vec!["mp3", "wav", "flac", "aac", "ogg", "wma", "m4a"];
        for ext in audio {
            type_map.insert(ext.to_string(), "Audio".to_string());
        }
        
        // Archives
        let archives = vec!["zip", "tar", "gz", "bz2", "xz", "7z", "rar", "tar.gz", "tar.bz2"];
        for ext in archives {
            type_map.insert(ext.to_string(), "Archives".to_string());
        }
        
        // Code
        let code = vec!["rs", "py", "js", "ts", "html", "css", "cpp", "c", "h", "java", "go", "php"];
        for ext in code {
            type_map.insert(ext.to_string(), "Code".to_string());
        }
        
        // Executables
        let executables = vec!["exe", "bin", "app", "deb", "rpm", "msi", "dmg"];
        for ext in executables {
            type_map.insert(ext.to_string(), "Executables".to_string());
        }
        
        Self { type_map }
    }
    
    /// Classify a file by its extension
    pub fn classify(&self, path: &Path) -> String {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                let ext_lower = ext_str.to_lowercase();
                return self.type_map.get(&ext_lower)
                    .cloned()
                    .unwrap_or_else(|| "Other".to_string());
            }
        }
        "Other".to_string()
    }
}

/// Size breakdown categorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeBreakdown {
    pub small_files_count: u64,    // < 1MB
    pub small_files_size: u64,
    pub medium_files_count: u64,   // 1MB - 100MB
    pub medium_files_size: u64,
    pub large_files_count: u64,    // > 100MB
    pub large_files_size: u64,
}

impl SizeBreakdown {
    /// Create size breakdown from scan results
    pub fn from_scan_results(results: &ScanResults) -> Self {
        let mut breakdown = SizeBreakdown {
            small_files_count: 0,
            small_files_size: 0,
            medium_files_count: 0,
            medium_files_size: 0,
            large_files_count: 0,
            large_files_size: 0,
        };
        
        const ONE_MB: u64 = 1_024 * 1_024;
        const HUNDRED_MB: u64 = 100 * ONE_MB;
        
        for file in &results.files {
            match file.size {
                size if size < ONE_MB => {
                    breakdown.small_files_count += 1;
                    breakdown.small_files_size += size;
                }
                size if size < HUNDRED_MB => {
                    breakdown.medium_files_count += 1;
                    breakdown.medium_files_size += size;
                }
                size => {
                    breakdown.large_files_count += 1;
                    breakdown.large_files_size += size;
                }
            }
        }
        
        breakdown
    }
}

/// Format duration in a human-readable way
pub fn format_duration(duration: std::time::Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    let millis = duration.subsec_millis();
    
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else if seconds > 0 {
        format!("{}.{:03}s", seconds, millis)
    } else {
        format!("{}ms", millis)
    }
}

/// Calculate percentage with proper formatting
pub fn calculate_percentage(part: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (part as f64 / total as f64) * 100.0
    }
}

/// Validate directory path
pub fn validate_directory(path: &Path) -> anyhow::Result<()> {
    if !path.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", path.display()));
    }
    
    if !path.is_dir() {
        return Err(anyhow::anyhow!("Path is not a directory: {}", path.display()));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_file_type_classification() {
        let classifier = FileTypeClassifier::new();
        
        assert_eq!(classifier.classify(&PathBuf::from("test.rs")), "Code");
        assert_eq!(classifier.classify(&PathBuf::from("image.jpg")), "Images");
        assert_eq!(classifier.classify(&PathBuf::from("document.pdf")), "Documents");
        assert_eq!(classifier.classify(&PathBuf::from("archive.zip")), "Archives");
        assert_eq!(classifier.classify(&PathBuf::from("unknown.xyz")), "Other");
    }
    
    #[test]
    fn test_percentage_calculation() {
        assert_eq!(calculate_percentage(25, 100), 25.0);
        assert_eq!(calculate_percentage(0, 100), 0.0);
        assert_eq!(calculate_percentage(100, 0), 0.0);
    }
    
    #[test]
    fn test_duration_formatting() {
        let duration = std::time::Duration::from_millis(1500);
        assert_eq!(format_duration(duration), "1.500s");
        
        let duration = std::time::Duration::from_secs(65);
        assert_eq!(format_duration(duration), "1m 5s");
    }
}
