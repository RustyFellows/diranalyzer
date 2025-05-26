//! Command-line interface definitions and argument parsing

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "diranalyzer",
    about = "A high-performance CLI tool for comprehensive directory analysis",
    long_about = "DirAnalyzer provides detailed insights into directory structures including size breakdowns, file type distributions, and duplicate file detection. Perfect for system administration, cleanup operations, and storage optimization."
)]
pub struct Args {
    /// Directory path to analyze
    #[arg(value_name = "PATH", help = "Path to the directory to analyze")]
    pub path: PathBuf,

    /// Maximum depth for directory traversal
    #[arg(
        short = 'd',
        long = "depth",
        default_value = "10",
        help = "Maximum depth for directory traversal"
    )]
    pub max_depth: usize,

    /// Enable duplicate file detection
    #[arg(
        long = "duplicates",
        help = "Enable duplicate file detection using SHA-256 hashing"
    )]
    pub find_duplicates: bool,

    /// Minimum file size for duplicate detection (in bytes)
    #[arg(
        long = "min-size",
        default_value = "1024",
        help = "Minimum file size for duplicate detection (bytes)"
    )]
    pub min_duplicate_size: u64,

    /// Show hidden files and directories
    #[arg(
        short = 'a',
        long = "all",
        help = "Include hidden files and directories in analysis"
    )]
    pub show_hidden: bool,

    /// Export results to file
    #[arg(
        short = 'e',
        long = "export",
        value_enum,
        help = "Export results to specified format"
    )]
    pub export: Option<ExportFormat>,

    /// Output file path for export
    #[arg(
        short = 'o',
        long = "output",
        help = "Output file path for export (default: auto-generated)"
    )]
    pub output: Option<PathBuf>,

    /// Number of top items to display in reports
    #[arg(
        short = 'n',
        long = "top",
        default_value = "20",
        help = "Number of top items to display in size and type reports"
    )]
    pub top_count: usize,

    /// Exclude patterns (glob syntax)
    #[arg(
        long = "exclude",
        help = "Exclude files/directories matching patterns (glob syntax)",
        action = clap::ArgAction::Append
    )]
    pub exclude_patterns: Vec<String>,

    /// Follow symbolic links
    #[arg(
        long = "follow-links",
        help = "Follow symbolic links during traversal"
    )]
    pub follow_links: bool,

    /// Verbose output
    #[arg(
        short = 'v',
        long = "verbose",
        help = "Enable verbose output with detailed progress information"
    )]
    pub verbose: bool,

    /// Quiet mode (minimal output)
    #[arg(
        short = 'q',
        long = "quiet",
        help = "Quiet mode - show only essential information"
    )]
    pub quiet: bool,

    /// Number of threads for parallel processing
    #[arg(
        short = 't',
        long = "threads",
        help = "Number of threads for parallel processing (default: auto-detect)"
    )]
    pub threads: Option<usize>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ExportFormat {
    /// Export as JSON
    Json,
    /// Export as CSV
    Csv,
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportFormat::Json => write!(f, "json"),
            ExportFormat::Csv => write!(f, "csv"),
        }
    }
}
