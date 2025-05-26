//! DirAnalyzer - A high-performance CLI tool for comprehensive directory analysis
//! 
//! This tool provides size breakdowns, file type distributions, and duplicate detection
//! for Linux/Unix systems with excellent performance and user experience.

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use std::time::Instant;

mod cli;
mod analyzer;
mod scanner;
mod duplicates;
mod reporter;
mod export;
mod utils;

use cli::Args;
use analyzer::DirectoryAnalyzer;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Print banner
    print_banner();
    
    let start_time = Instant::now();
    
    // Initialize analyzer with configuration
    let mut analyzer = DirectoryAnalyzer::new(args.clone())?;
    
    // Perform analysis
    let results = analyzer.analyze().await?;
    
    let duration = start_time.elapsed();
    
    // Generate and display report
    reporter::generate_report(&results, &args, duration)?;
    
    // Export results if requested
    if let Some(export_format) = &args.export {
        export::export_results(&results, export_format, &args.output)?;
        println!("{} Results exported successfully!", "✓".green().bold());
    }
    
    Ok(())
}

fn print_banner() {
    println!("{}", "
██████╗ ██╗██████╗  █████╗ ███╗   ██╗ █████╗ ██╗  ██╗   ██╗███████╗███████╗██████╗ 
██╔══██╗██║██╔══██╗██╔══██╗████╗  ██║██╔══██╗██║  ╚██╗ ██╔╝╚══███╔╝██╔════╝██╔══██╗
██║  ██║██║██████╔╝███████║██╔██╗ ██║███████║██║   ╚████╔╝   ███╔╝ █████╗  ██████╔╝
██║  ██║██║██╔══██╗██╔══██║██║╚██╗██║██╔══██║██║    ╚██╔╝   ███╔╝  ██╔══╝  ██╔══██╗
██████╔╝██║██║  ██║██║  ██║██║ ╚████║██║  ██║███████╗██║   ███████╗███████╗██║  ██║
╚═════╝ ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝╚══════╝╚═╝   ╚══════╝╚══════╝╚═╝  ╚═╝
    ".cyan().bold());
    println!("{}", "High-Performance Directory Analysis Tool".yellow().bold());
    println!("{}", "========================================".cyan());
    println!();
}
