//! PenguinWash CLI entry point

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::info;
use tracing_subscriber::EnvFilter;

use penguinwash_lib::config::{load_config, save_config};
use penguinwash_lib::{
    run_large_file_scan, run_scan, Config, ScanResult,
};

#[derive(Parser)]
#[command(
    name = "penguinwash",
    version = "0.1.0",
    about = "Free, open-source Linux system cleaner"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Output JSON instead of human-readable
    #[arg(long)]
    json: bool,

    /// Launch the graphical user interface
    #[arg(long)]
    gui: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan all categories for cleanable junk
    Scan {
        /// Categories to scan (default: all)
        #[arg(short, long)]
        categories: Vec<String>,
    },
    /// Clean selected categories
    Clean {
        /// Categories to clean (default: all auto-cleanable)
        #[arg(short, long)]
        categories: Vec<String>,

        /// Actually delete files (default: dry-run)
        #[arg(short, long)]
        force: bool,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
    /// Print current configuration
    ShowConfig,
    /// Reset configuration to defaults
    ResetConfig,
    /// Scan for large files (for remote diagnosis)
    LargeFiles {
        /// Minimum file size in MB (default: 100)
        #[arg(short, long, default_value = "100")]
        min_mb: u64,
        /// Paths to scan (default: /)
        #[arg(short, long, num_args = 1..)]
        paths: Vec<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Init logging
    let filter = if cli.verbose {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"))
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    };
    tracing_subscriber::fmt().with_env_filter(filter).init();

    // GUI mode
    if cli.gui {
        #[cfg(feature = "gui")]
        {
            if let Err(e) = penguinwash_lib::gui::run_gui() {
                eprintln!("GUI error: {}", e);
                std::process::exit(1);
            }
        }
        #[cfg(not(feature = "gui"))]
        {
            println!("GUI not compiled. Run with `--features gui` to enable.");
        }
        return Ok(());
    }

    let config = load_config()?;

    // Use blocking tokio runtime
    let rt = tokio::runtime::Runtime::new()?;
    let _guard = rt.enter();

    match &cli.command {
        Some(Commands::Scan { categories: _ }) => {
            info!("Starting scan...");
            let result = rt.block_on(run_scan(&config))?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                print_scan_result(&result);
            }
        }
        Some(Commands::Clean { categories: _, force, yes: _ }) => {
            if !force {
                info!("Dry run mode. Use --force to actually delete.");
            }
            info!("Clean command not yet implemented. Use --force when ready.");
        }
        Some(Commands::ShowConfig) => {
            println!("{}", toml::to_string_pretty(&config)?);
        }
        Some(Commands::ResetConfig) => {
            let default = Config::default();
            save_config(&default)?;
            println!("Config reset to defaults.");
        }
        Some(Commands::LargeFiles { min_mb, paths }) => {
            info!("Scanning for files larger than {} MB...", min_mb);
            let scan_paths: Vec<std::path::PathBuf> = if paths.is_empty() {
                vec![std::path::PathBuf::from("/")]
            } else {
                paths.iter().map(|p| std::path::PathBuf::from(p)).collect()
            };
            let files = rt.block_on(run_large_file_scan(scan_paths, *min_mb))?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&files)?);
            } else {
                println!("\n{:>12}  {}", "Size", "Path");
                println!("{}", "-".repeat(80));
                for f in &files {
                    println!("{:>12}  {}", f.size_formatted(), f.path);
                }
                println!("\n{} files found", files.len());
            }
        }
        None => {
            info!("PenguinWash v0.1.0 — Linux system cleaner");
            info!("Run 'penguinwash scan' or 'penguinwash --gui' to start.");
            let result = rt.block_on(run_scan(&config))?;
            print_scan_result(&result);
        }
    }

    Ok(())
}

fn print_scan_result(result: &ScanResult) {
    println!("\n═══════════════════════════════════════");
    println!("  PenguinWash Scan Results");
    println!("═══════════════════════════════════════\n");
    println!(
        "  Total reclaimable: {} ({} categories)\n",
        result.grand_total_size_formatted(),
        result.categories.len()
    );
    println!("  {:<25} {:>10} {:>8}", "Category", "Size", "Items");
    println!("  {}", "-".repeat(45));

    for cat in &result.categories {
        let flag = if cat.auto_cleanable { "•" } else { "◦" };
        println!(
            "  {} {:<23} {:>10} {:>8}",
            flag, cat.name, cat.total_size_formatted(), cat.item_count
        );
    }

    println!("\n  Scanned in {}ms", result.scan_duration_ms);
}
