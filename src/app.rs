use arboard::Clipboard;
use chrono::Local;
use clap::{Parser, Subcommand};
use colored::*;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;

use crate::capture::CaptureSession;
use crate::config::{Config, DEFAULT_SKIP_COMMANDS};

#[derive(Parser, Debug)]
#[command(author, version, about = "Captures the output of the last command.")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Optional output filename. If not provided, generates a timestamped name.
    #[arg(short, long)]
    output: Option<String>,

    /// Whether to copy the output to clipboard as well.
    #[arg(short, long)]
    clipboard: bool,

    /// Keep ANSI colors in the file (default is to strip them)
    #[arg(short, long)]
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

    /// Internal commands for shell integration.
    Internal {
        #[arg(long)]
        should_prepare: Option<String>,

        #[arg(long)]
        prepare: Option<String>,

        #[arg(long)]
        cleanup: bool,
    },

    /// Configure skip commands (commands that won't be captured).
    Config {
        #[arg(long)]
        list: bool,

        #[arg(long)]
        add: Option<String>,

        #[arg(long)]
        remove: Option<String>,

        #[arg(long)]
        clear: bool,

        #[arg(long)]
        reset: bool,
    },
}

fn handle_internal_command(cli: &Cli, config: &Config) -> anyhow::Result<()> {
    if let Commands::Internal {
        should_prepare,
        prepare,
        cleanup,
    } = &cli.command.as_ref().unwrap()
    {
        let session = CaptureSession::new(config);

        if let Some(cmd) = should_prepare {
            let should = session.should_prepare(cmd);
            println!("{}", if should { "true" } else { "false" });
            return Ok(());
        }

        if let Some(cmd) = prepare {
            // Prepare session and return paths
            let result = session.prepare_new_command(cmd)?;
            println!("{}", result.display());
            return Ok(());
        }

        if *cleanup {
            session.cleanup()?;
            return Ok(());
        }
    }

    Ok(())
}

fn handle_config_command(cli: &Cli, config: &Config) -> anyhow::Result<()> {
    if let Commands::Config {
        list,
        add,
        remove,
        clear,
        reset,
    } = &cli.command.as_ref().unwrap()
    {
        let config = config.clone();

        if *list {
            let skip_commands = config.skip_commands();
            if skip_commands.is_empty() {
                println!("{}", "No commands in skip list.".dimmed());
            } else {
                println!("{}", "Commands to skip:".bold().underline());
                for cmd in skip_commands {
                    println!("  {} {}", "•".cyan(), cmd);
                }
            }
            return Ok(());
        }

        if let Some(cmd) = add {
            let mut skip_commands = config.skip_commands();
            if !skip_commands.contains(cmd) {
                skip_commands.push(cmd.clone());
                config.save_skip_commands(&skip_commands)?;
                println!("{} Added '{}' to skip list", "✔".green(), cmd.bold());
            } else {
                println!("{} '{}' is already in skip list", "!".yellow(), cmd.bold());
            }
            return Ok(());
        }

        if let Some(cmd) = remove {
            let mut skip_commands = config.skip_commands();
            if skip_commands.contains(cmd) {
                skip_commands.retain(|c| c != cmd);
                config.save_skip_commands(&skip_commands)?;
                println!("{} Removed '{}' from skip list", "✔".green(), cmd.bold());
            } else {
                println!("{} '{}' is not in skip list", "✘".red(), cmd.bold());
            }
            return Ok(());
        }

        if *clear {
            config.save_skip_commands(&Vec::new())?;
            println!("{} Cleared all skip commands", "✔".green());
            return Ok(());
        }

        if *reset {
            let defaults: Vec<String> = DEFAULT_SKIP_COMMANDS
                .iter()
                .map(|s| s.to_string())
                .collect();
            config.save_skip_commands(&defaults)?;
            println!("{} Reset skip commands to defaults", "✔".green());
            return Ok(());
        }
    }

    Ok(())
}

fn handle_get_last_output(cli: &Cli, config: &Config) -> anyhow::Result<()> {
    let session = CaptureSession::new(config);

    if !session.has_previous() {
        eprintln!("{}", "Error: No previous command record found.".red());
        eprint!("Maybe recording was disabled? Run 'tellme status' to check, ");
        eprintln!("or last command was skipped.");
        return Ok(());
    }

    let last_cmd = session.read_cmd_file()?;
    let last_content = session.read_output()?;

    let final_content = if cli.raw {
        last_content
    } else {
        strip_ansi_escapes::strip(&last_content)
    };

    let content_str = String::from_utf8_lossy(&final_content).to_string();

    if cli.clipboard {
        let mut clipboard = Clipboard::new().unwrap();
        clipboard.set_text(content_str).unwrap();
        std::thread::sleep(Duration::from_millis(100));

        println!("{} Output copied to clipboard.", "✔".green());
    } else {
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
        writeln!(file, "Command:\n{}", last_cmd)?;
        writeln!(file, "=============================\n")?;
        writeln!(file, "{}", content_str)?;

        println!("{} Output saved to {}", "✔".green(), target_file.bold());
    }

    Ok(())
}

pub fn app() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = Config::new()?;

    match cli.command {
        Some(Commands::On) => {
            let mut config_mut = config;
            config_mut.set_recording_enabled(true)?;
            println!(
                "{} tellme recording is now {}",
                "✔".green(),
                "ENABLED".bold().green()
            );
        }
        Some(Commands::Off) => {
            let mut config_mut = config;
            config_mut.set_recording_enabled(false)?;
            println!(
                "{} tellme recording is now {}",
                "✔".green(),
                "DISABLED".bold().yellow()
            );
        }
        Some(Commands::Status) => {
            if config.is_recording_enabled() {
                println!("tellme recording is {}", "ENABLED".bold().green());
            } else {
                println!("tellme recording is {}", "DISABLED".bold().yellow());
            }
        }
        Some(Commands::Internal { .. }) => {
            handle_internal_command(&cli, &config)?;
        }
        Some(Commands::Config { .. }) => {
            handle_config_command(&cli, &config)?;
        }
        None => {
            // Default behavior: save the last output
            handle_get_last_output(&cli, &config)?;
        }
    }

    Ok(())
}
