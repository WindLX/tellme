use chrono::Local;
use clap::Parser;
use colored::*;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long)]
    output: Option<String>,
    #[arg(short, long)]
    raw: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let pid = std::env::var("TELLME_SHELL_PID").unwrap_or_else(|_| "unknown".to_string());

    let root_dir = if let Ok(root) = std::env::var("TELLME_ROOT") {
        PathBuf::from(root)
    } else {
        std::env::temp_dir()
    };

    let log_file_path = root_dir.join(format!(".tellme_last_{}", pid));
    let cmd_file_path = root_dir.join(format!(".tellme_last_cmd_{}", pid));

    if !log_file_path.exists() {
        eprintln!("{}", "Error: No previous command record found.".red());
        eprintln!("(Path searched: {:?})", log_file_path);
        return Ok(());
    }

    let raw_content = fs::read(&log_file_path)?;

    let last_cmd = fs::read_to_string(&cmd_file_path).unwrap_or_else(|_| "unknown".to_string());

    let final_content = if args.raw {
        raw_content
    } else {
        strip_ansi_escapes::strip(&raw_content)
    };

    let target_file = match args.output {
        Some(name) => name,
        None => {
            let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
            format!("tellme_{}.log", timestamp)
        }
    };

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&target_file)?;

    writeln!(file, "Command: {}", last_cmd.trim())?;
    writeln!(file, "----------------------------------------")?;
    file.write_all(&final_content)?;

    println!("{} Output saved to {}", "✔".green(), target_file.bold());

    Ok(())
}
