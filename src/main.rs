use chrono::Local;
use clap::{Parser, Subcommand};
use colored::*;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "Captures the output of the last command.")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Optional output filename. If not provided, generates a timestamped name.
    #[arg(short, long, global = true)]
    output: Option<String>,

    /// Keep ANSI colors in the file (default is to strip them)
    #[arg(short, long, global = true)]
    raw: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Enable tellme output recording.
    On,
    /// Disable tellme output recording.
    Off,
    /// Show the current recording status.
    Status,
}

// 获取配置目录的辅助函数
fn get_config_dir() -> PathBuf {
    let config_dir_env = std::env::var("TELLME_CONFIG_DIR");
    if let Ok(dir) = config_dir_env {
        PathBuf::from(dir)
    } else {
        dirs::home_dir().unwrap().join(".config").join("tellme")
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let config_dir = get_config_dir();
    let status_file = config_dir.join("status");

    match cli.command {
        Some(Commands::On) => {
            fs::write(&status_file, "enabled")?;
            println!(
                "{} tellme recording is now {}",
                "✔".green(),
                "ENABLED".bold().green()
            );
        }
        Some(Commands::Off) => {
            fs::write(&status_file, "disabled")?;
            println!(
                "{} tellme recording is now {}",
                "✔".green(),
                "DISABLED".bold().yellow()
            );
        }
        Some(Commands::Status) => {
            let status =
                fs::read_to_string(&status_file).unwrap_or_else(|_| "disabled".to_string());
            if status.trim() == "enabled" {
                println!("tellme recording is {}", "ENABLED".bold().green());
            } else {
                println!("tellme recording is {}", "DISABLED".bold().yellow());
            }
        }
        None => {
            // 默认行为：保存日志
            save_last_output(&cli)?;
        }
    }

    Ok(())
}

fn save_last_output(cli: &Cli) -> anyhow::Result<()> {
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
        eprintln!("(Maybe recording was disabled? Run 'tellme status' to check)");
        return Ok(());
    }

    let raw_content = fs::read(&log_file_path)?;
    let last_cmd = fs::read_to_string(&cmd_file_path).unwrap_or_else(|_| "unknown".to_string());

    let final_content = if cli.raw {
        raw_content
    } else {
        strip_ansi_escapes::strip(&raw_content)
    };

    let target_file = match &cli.output {
        Some(name) => name.clone(),
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
