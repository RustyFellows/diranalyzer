//! DirAnalyzer library providing directory analysis functionality
//! 
//! This library offers comprehensive directory scanning, file type analysis,
//! and duplicate detection capabilities with high performance.

pub mod cli;
pub mod analyzer;
pub mod scanner;
pub mod duplicates;
pub mod reporter;
pub mod export;
pub mod utils;

pub use analyzer::{DirectoryAnalyzer, AnalysisResults};
pub use cli::Args;
