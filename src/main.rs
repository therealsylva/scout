use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;

use clap::Parser;
use chrono::{DateTime, Local, Utc};
use colored::*;
use ignore::Walk;
use rayon::prelude::*;

#[derive(Parser, Debug)]
#[command(name = "scout")]
#[command(about = "Blazing fast extension counter and file analyzer", long_about = None)]
struct Cli {
    #[arg(short, long)]
    extension: Option<String>,

    #[arg(short, long, default_value = ".")]
    target: String,

    #[arg(short, long)]
    sort_by: Option<SortBy>,

    #[arg(short, long, default_value = "false")]
    json: bool,

    #[arg(short, long, default_value = "false")]
    detailed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum SortBy {
    Count,
    Size,
    Date,
    Extension,
}

#[derive(Debug, Clone, serde::Serialize)]
struct ExtensionStats {
    extension: String,
    count: usize,
    total_size: u64,
    oldest: Option<DateTime<Utc>>,
    newest: Option<DateTime<Utc>>,
    avg_size: u64,
}

fn main() {
    let cli = Cli::parse();

    let mut stats: HashMap<String, ExtensionStats> = HashMap::new();

    let walker = Walk::new(&cli.target).parallel();
    let entries: Vec<_> = walker
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().map_or(false, |ft| ft.is_file()))
        .collect();

    if let Some(ref ext) = cli.extension {
        entries.par_iter().for_each(|entry| {
            if let Some(file_ext) = entry.path().extension() {
                if file_ext.to_string_lossy().to_lowercase() == ext.to_lowercase() {
                    process_file(entry, &mut stats);
                }
            }
        });
    } else {
        entries.par_iter().for_each(|entry| {
            process_file(entry, &mut stats);
        });
    }

    let mut results: Vec<ExtensionStats> = stats.into_values().collect();

    match cli.sort_by.unwrap_or(SortBy::Count) {
        SortBy::Count => results.sort_by(|a, b| b.count.cmp(&a.count)),
        SortBy::Size => results.sort_by(|a, b| b.total_size.cmp(&a.total_size)),
        SortBy::Date => results.sort_by(|a, b| {
            let a_date = a.newest.unwrap_or_else(|| DateTime::<Utc>::MIN_UTC);
            let b_date = b.newest.unwrap_or_else(|| DateTime::<Utc>::MIN_UTC);
            b_date.cmp(&a_date)
        }),
        SortBy::Extension => results.sort_by(|a, b| a.extension.cmp(&b.extension)),
    }

    if cli.json {
        println!("{}", serde_json::to_string_pretty(&results).unwrap());
    } else {
        print_results(&results, cli.detailed);
    }
}

fn process_file(entry: &ignore::DirEntry, stats: &mut HashMap<String, ExtensionStats>) {
    let path = entry.path();

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("<no extension>")
        .to_lowercase();

    let metadata = match path.metadata() {
        Ok(m) => m,
        Err(_) => return,
    };

    let size = metadata.len();
    let modified = metadata.modified().ok();

    let entry_stats = stats.entry(ext.clone()).or_insert_with(|| ExtensionStats {
        extension: ext.clone(),
        count: 0,
        total_size: 0,
        oldest: None,
        newest: None,
        avg_size: 0,
    });

    entry_stats.count += 1;
    entry_stats.total_size += size;

    if let Some(modified) = modified {
        let modified_dt: DateTime<Utc> = modified.into();

        if entry_stats.oldest.is_none() || Some(modified_dt) < entry_stats.oldest {
            entry_stats.oldest = Some(modified_dt);
        }
        if entry_stats.newest.is_none() || Some(modified_dt) > entry_stats.newest {
            entry_stats.newest = Some(modified_dt);
        }
    }
}

fn print_results(results: &[ExtensionStats], detailed: bool) {
    println!();
    println!(
        "{}",
        "┌─────────────────────────────────────────────────────────────────┐"
            .bright_black()
    );
    println!(
        "{}",
        "│  SCOUT - Fast Extension Analyzer                                 │"
            .bright_cyan()
            .bold()
    );
    println!(
        "{}",
        "└─────────────────────────────────────────────────────────────────┘"
            .bright_black()
    );
    println!();

    if results.is_empty() {
        println!("  {} No files found", "!".yellow());
        return;
    }

    let total_files: usize = results.iter().map(|r| r.count).sum();
    let total_size: u64 = results.iter().map(|r| r.total_size).sum();

    println!(
        "  {} Total files: {}",
        "📁".bright_black(),
        total_files.to_string().cyan().bold()
    );
    println!(
        "  {} Total size: {}",
        "💾".bright_black(),
        format_size(total_size).green().bold()
    );
    println!(
        "  {} Extensions: {}",
        "📊".bright_black(),
        results.len().to_string().yellow().bold()
    );
    println!();
    println!(
        "{}",
        "─".repeat(75).bright_black()
    );

    if detailed {
        println!(
            "{:<15} {:>8} {:>15} {:>20} {:>20}",
            "Extension".cyan().bold(),
            "Count".cyan().bold(),
            "Total Size".cyan().bold(),
            "Newest".cyan().bold(),
            "Oldest".cyan().bold()
        );
        println!(
            "{}",
            "─".repeat(75).bright_black()
        );

        for stat in results {
            let newest = stat.newest.map(|d| format_datetime(&d)).unwrap_or_else(|| "N/A".to_string());
            let oldest = stat.oldest.map(|d| format_datetime(&d)).unwrap_or_else(|| "N/A".to_string());

            println!(
                "{:<15} {:>8} {:>15} {:>20} {:>20}",
                format_extension(&stat.extension),
                stat.count.to_string().white(),
                format_size(stat.total_size),
                newest.bright_black(),
                oldest.bright_black()
            );
        }
    } else {
        println!(
            "{:<15} {:>8} {:>15} {:>12}",
            "Extension".cyan().bold(),
            "Count".cyan().bold(),
            "Total Size".cyan().bold(),
            "Avg Size".cyan().bold()
        );
        println!(
            "{}",
            "─".repeat(75).bright_black()
        );

        for stat in results {
            let avg = if stat.count > 0 {
                stat.total_size / stat.count as u64
            } else {
                0
            };

            let count_color = if stat.count > 100 {
                "green"
            } else if stat.count > 50 {
                "yellow"
            } else {
                "white"
            };

            println!(
                "{:<15} {:>8} {:>15} {:>12}",
                format_extension(&stat.extension),
                stat.count.to_string().color(count_color),
                format_size(stat.total_size),
                format_size(avg)
            );
        }
    }

    println!(
        "{}",
        "─".repeat(75).bright_black()
    );
    println!();
}

fn format_extension(ext: &str) -> String {
    if ext == "<no extension>" {
        ext.to_string().bright_black().to_string()
    } else {
        format!(".{}", ext)
    }
}

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    const TB: u64 = 1024 * GB;

    if size >= TB {
        format!("{:.2} TB", size as f64 / TB as f64)
    } else if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

fn format_datetime(dt: &DateTime<Utc>) -> String {
    let local: DateTime<Local> = dt.with_timezone(&Local);
    local.format("%Y-%m-%d %H:%M").to_string()
}
