//! Export functionality for analysis results

use crate::analyzer::AnalysisResults;
use crate::cli::ExportFormat;
use anyhow::{Context, Result};
use serde_json;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Export analysis results to the specified format
pub fn export_results(
    results: &AnalysisResults,
    format: &ExportFormat,
    output_path: &Option<PathBuf>,
) -> Result<()> {
    let output_path = generate_output_path(format, output_path)?;
    
    match format {
        ExportFormat::Json => export_json(results, &output_path),
        ExportFormat::Csv => export_csv(results, &output_path),
    }
}

fn generate_output_path(format: &ExportFormat, output_path: &Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = output_path {
        Ok(path.clone())
    } else {
        let now = chrono::Utc::now();
        let timestamp = now.format("%Y%m%d_%H%M%S");
        let filename = format!("diranalyzer_report_{}.{}", timestamp, format);
        Ok(PathBuf::from(filename))
    }
}

fn export_json(results: &AnalysisResults, output_path: &Path) -> Result<()> {
    let json_data = serde_json::to_string_pretty(results)
        .context("Failed to serialize results to JSON")?;
    
    let mut file = File::create(output_path)
        .with_context(|| format!("Failed to create output file: {}", output_path.display()))?;
    
    file.write_all(json_data.as_bytes())
        .with_context(|| format!("Failed to write JSON data to: {}", output_path.display()))?;
    
    println!("📄 JSON report exported to: {}", output_path.display());
    Ok(())
}

fn export_csv(results: &AnalysisResults, output_path: &Path) -> Result<()> {
    let file = File::create(output_path)
        .with_context(|| format!("Failed to create output file: {}", output_path.display()))?;
    
    let mut writer = csv::Writer::from_writer(file);
    
    // Export file information
    writer.write_record(&[
        "Type", "Path", "Size", "FileType", "Modified", "Depth"
    ])?;
    
    for file in &results.largest_files {
        writer.write_record(&[
            "File",
            &file.path.display().to_string(),
            &file.size.to_string(),
            &file.file_type,
            &file.modified.map(|m| m.to_rfc3339()).unwrap_or_default(),
            "", // Depth not available in FileInfo
        ])?;
    }
    
    for dir in &results.largest_directories {
        writer.write_record(&[
            "Directory",
            &dir.path.display().to_string(),
            &dir.size.to_string(),
            "Directory",
            "",
            "",
        ])?;
    }
    
    // Export duplicate information if available
    if let Some(ref groups) = results.duplicate_groups {
        for group in groups {
            for file_path in &group.files {
                writer.write_record(&[
                    "Duplicate",
                    &file_path.display().to_string(),
                    &group.file_size.to_string(),
                    "Duplicate",
                    "",
                    "",
                ])?;
            }
        }
    }
    
    writer.flush()?;
    println!("📊 CSV report exported to: {}", output_path.display());
    Ok(())
}
