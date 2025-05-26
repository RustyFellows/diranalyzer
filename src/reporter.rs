//! Report generation and formatting

use crate::analyzer::AnalysisResults;
use crate::cli::Args;
use anyhow::Result;
use colored::Colorize;
use humansize::{format_size, DECIMAL};
use std::time::Duration;

/// Generate and display comprehensive analysis report
pub fn generate_report(results: &AnalysisResults, args: &Args, duration: Duration) -> Result<()> {
    if args.quiet {
        print_summary_only(results);
        return Ok(());
    }

    print_header(results, duration);
    print_size_breakdown(results, args.top_count);
    print_file_type_distribution(results, args.top_count);
    print_largest_files(results, args.top_count);
    print_largest_directories(results, args.top_count);
    
    if results.duplicate_groups.is_some() {
        print_duplicate_analysis(results, args.top_count);
    }
    
    print_performance_statistics(results);
    print_footer();

    Ok(())
}

fn print_summary_only(results: &AnalysisResults) {
    println!("{}: {} files, {} directories, {} total",
        "Summary".bold(),
        results.scan_info.total_files,
        results.scan_info.total_directories,
        format_size(results.scan_info.total_size, DECIMAL).cyan()
    );
}

fn print_header(results: &AnalysisResults, duration: Duration) {
    println!("\n{}", "📋 ANALYSIS REPORT".cyan().bold());
    println!("{}", "=".repeat(50).cyan());
    
    println!("\n{}", "📁 Scan Information".yellow().bold());
    println!("  Path: {}", results.scan_info.path.display().to_string().green());
    println!("  Timestamp: {}", results.scan_info.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("  Duration: {:.2}s", duration.as_secs_f64());
    println!("  Depth Limit: {}", results.scan_info.depth_limit);
    
    println!("\n{}", "📊 Overview".yellow().bold());
    println!("  Total Files: {}", results.scan_info.total_files.to_string().cyan());
    println!("  Total Directories: {}", results.scan_info.total_directories.to_string().cyan());
    println!("  Total Size: {}", format_size(results.scan_info.total_size, DECIMAL).cyan().bold());
    
    if let Some(ref groups) = results.duplicate_groups {
        let duplicate_files = groups.iter().map(|g| g.files.len()).sum::<usize>();
        let wasted_space: u64 = groups.iter().map(|g| g.wasted_space).sum();
        println!("  Duplicate Files: {}", duplicate_files.to_string().red());
        println!("  Wasted Space: {}", format_size(wasted_space, DECIMAL).red().bold());
    }
}

fn print_size_breakdown(results: &AnalysisResults, top_count: usize) {
    println!("\n{}", "📏 Size Breakdown".yellow().bold());
    
    let breakdown = &results.size_breakdown;
    
    println!("  Small files (<1MB): {} files, {}",
        breakdown.small_files_count.to_string().cyan(),
        format_size(breakdown.small_files_size, DECIMAL).cyan()
    );
    println!("  Medium files (1MB-100MB): {} files, {}",
        breakdown.medium_files_count.to_string().cyan(),
        format_size(breakdown.medium_files_size, DECIMAL).cyan()
    );
    println!("  Large files (>100MB): {} files, {}",
        breakdown.large_files_count.to_string().cyan(),
        format_size(breakdown.large_files_size, DECIMAL).cyan()
    );
}

fn print_file_type_distribution(results: &AnalysisResults, top_count: usize) {
    println!("\n{}", "📄 File Type Distribution".yellow().bold());
    
    let mut types: Vec<_> = results.file_type_distribution.iter().collect();
    types.sort_by(|a, b| b.1.total_size.cmp(&a.1.total_size));
    
    let total_size = results.scan_info.total_size;
    
    for (i, (file_type, stats)) in types.iter().take(top_count).enumerate() {
        let percentage = if total_size > 0 {
            (stats.total_size as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };
        
        println!("  {}. {} files ({}) - {} ({:.1}%)",
            (i + 1).to_string().cyan(),
            file_type.green().bold(),
            stats.count,
            format_size(stats.total_size, DECIMAL).cyan(),
            percentage
        );
        
        if let Some(ref largest) = stats.largest_file {
            println!("     Largest: {} ({})",
                largest.path.display(),
                format_size(largest.size, DECIMAL)
            );
        }
    }
}

fn print_largest_files(results: &AnalysisResults, top_count: usize) {
    println!("\n{}", "🗂️  Largest Files".yellow().bold());
    
    for (i, file) in results.largest_files.iter().take(top_count).enumerate() {
        println!("  {}. {} - {}",
            (i + 1).to_string().cyan(),
            format_size(file.size, DECIMAL).red().bold(),
            file.path.display().to_string().green()
        );
        
        if let Some(modified) = file.modified {
            println!("     Modified: {} | Type: {}",
                modified.format("%Y-%m-%d %H:%M"),
                file.file_type
            );
        }
    }
}

fn print_largest_directories(results: &AnalysisResults, top_count: usize) {
    println!("\n{}", "📁 Largest Directories".yellow().bold());
    
    for (i, dir) in results.largest_directories.iter().take(top_count).enumerate() {
        println!("  {}. {} - {}",
            (i + 1).to_string().cyan(),
            format_size(dir.size, DECIMAL).red().bold(),
            dir.path.display().to_string().green()
        );
        
        println!("     {} files, {} subdirectories",
            dir.file_count,
            dir.subdirectory_count
        );
    }
}

fn print_duplicate_analysis(results: &AnalysisResults, top_count: usize) {
    if let Some(ref groups) = results.duplicate_groups {
        println!("\n{}", "🔍 Duplicate File Analysis".yellow().bold());
        
        if groups.is_empty() {
            println!("  {} No duplicate files found!", "✓".green());
            return;
        }
        
        let total_groups = groups.len();
        let total_duplicates: usize = groups.iter().map(|g| g.files.len()).sum();
        let total_wasted: u64 = groups.iter().map(|g| g.wasted_space).sum();
        
        println!("  Duplicate Groups: {}", total_groups.to_string().red());
        println!("  Total Duplicate Files: {}", total_duplicates.to_string().red());
        println!("  Total Wasted Space: {}", format_size(total_wasted, DECIMAL).red().bold());
        
        println!("\n  Top Duplicate Groups:");
        
        for (i, group) in groups.iter().take(top_count).enumerate() {
            println!("    {}. {} ({} files) - {} wasted",
                (i + 1).to_string().cyan(),
                format_size(group.file_size, DECIMAL).yellow(),
                group.files.len(),
                format_size(group.wasted_space, DECIMAL).red()
            );
            
            for (j, file_path) in group.files.iter().take(3).enumerate() {
                let prefix = if j == group.files.len() - 1 || j == 2 { "└─" } else { "├─" };
                println!("       {} {}", prefix, file_path.display());
            }
            
            if group.files.len() > 3 {
                println!("       └─ ... and {} more files", (group.files.len() - 3));
            }
        }
    }
}

fn print_performance_statistics(results: &AnalysisResults) {
    println!("\n{}", "⚡ Performance Statistics".yellow().bold());
    
    let stats = &results.statistics;
    
    println!("  Scanning Speed: {:.0} files/sec", stats.files_per_second);
    println!("  Throughput: {}/sec", format_size(stats.bytes_per_second, DECIMAL));
    println!("  Memory Usage: {:.1} MB", stats.memory_usage_mb);
    
    if stats.duplicate_files > 0 {
        println!("  Duplicate Detection: {} files analyzed", stats.duplicate_files);
        println!("  Space Efficiency: {:.1}%", stats.compression_ratio * 100.0);
    }
}

fn print_footer() {
    println!("\n{}", "=".repeat(50).cyan());
    println!("{}", "Analysis complete! 🎉".green().bold());
    println!("{}", "Use --export to save results to file.");
}
