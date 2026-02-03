use std::fs;
use std::path::PathBuf;

pub const DEFAULT_SKIP_COMMANDS: &[&str] = &[
    "tellme", "clear", "exit", "cd", "vim", "vi", "nano", "less", "man", "htop", "top", "ssh",
    "tmux", "source",
];

/// Configuration management for tellme
#[derive(Debug, Clone)]
pub struct Config {
    /// Whether recording is enabled
    recording_enabled: bool,

    /// Directory for configuration files (default: ~/.config/tellme)
    config_dir: PathBuf,

    /// Directory for temporary log files (default: system temp dir)
    temp_dir: PathBuf,

    /// Shell PID for file naming
    shell_pid: u32,
}

impl Config {
    /// Create a new Config instance
    pub fn new() -> anyhow::Result<Self> {
        Self::with_paths(None, None, None)
    }

    /// Get the configuration directory, respecting TELLME_CONFIG_DIR env var
    fn config_dir() -> PathBuf {
        if let Ok(dir) = std::env::var("TELLME_CONFIG_DIR") {
            PathBuf::from(dir)
        } else {
            match dirs::config_dir() {
                Some(dir) => dir.join("tellme"),
                None => std::env::temp_dir().join("tellme_config"),
            }
        }
    }

    /// Get the temporary directory, respecting TELLME_TEMP_DIR env var
    fn temp_dir() -> PathBuf {
        if let Ok(dir) = std::env::var("TELLME_TEMP_DIR") {
            PathBuf::from(dir)
        } else {
            std::env::temp_dir().join("tellme")
        }
    }

    /// Get shell pid, respecting TELLME_SHELL_PID env var
    fn shell_pid() -> anyhow::Result<u32> {
        std::env::var("TELLME_SHELL_PID")
            .unwrap_or_default()
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid TELLME_SHELL_PID value"))
    }

    /// Create a new Config instance with explicit paths
    pub(crate) fn with_paths(
        shell_pid: Option<u32>,
        config_dir: Option<PathBuf>,
        temp_dir: Option<PathBuf>,
    ) -> anyhow::Result<Self> {
        let shell_pid = shell_pid.map_or_else(|| Self::shell_pid(), |p| Ok(p))?;
        let config_dir = config_dir.unwrap_or_else(Self::config_dir);
        let temp_dir = temp_dir.unwrap_or_else(Self::temp_dir);

        fs::create_dir_all(&config_dir)?;
        fs::create_dir_all(&temp_dir)?;

        Ok(Self {
            recording_enabled: Self::load_recording_status(&config_dir),
            config_dir,
            temp_dir,
            shell_pid,
        })
    }

    /// Load recording status from status file in specific directory
    fn load_recording_status(config_dir: &PathBuf) -> bool {
        let status_file = config_dir.join("status");

        match fs::read_to_string(&status_file) {
            Ok(content) => content.trim() == "enabled",
            Err(_) => {
                fs::write(&status_file, "disabled").ok();
                false
            }
        }
    }

    /// Get the status file path
    fn status_file(&self) -> PathBuf {
        self.config_dir.join("status")
    }

    /// Get the skip commands file path
    fn skip_commands_file(&self) -> PathBuf {
        self.config_dir.join("skip_commands")
    }

    /// Get the command file path
    pub fn cmd_file(&self) -> PathBuf {
        self.temp_dir
            .join(format!(".tellme_cmd_{}", self.shell_pid))
    }

    /// Get the log file path
    pub fn output_file(&self) -> PathBuf {
        self.temp_dir
            .join(format!(".tellme_output_{}", self.shell_pid))
    }

    /// Get all temp files for this shell
    pub fn temp_files(&self) -> Vec<PathBuf> {
        vec![self.cmd_file(), self.output_file()]
    }

    /// Check if recording is enabled
    pub fn is_recording_enabled(&self) -> bool {
        self.recording_enabled
    }

    /// Set recording status
    pub fn set_recording_enabled(&mut self, enabled: bool) -> anyhow::Result<()> {
        fs::write(
            &self.status_file(),
            if enabled { "enabled" } else { "disabled" },
        )?;

        self.recording_enabled = enabled;
        Ok(())
    }

    /// Load skip commands from file, or return defaults
    pub fn skip_commands(&self) -> Vec<String> {
        let skip_file = self.skip_commands_file();

        if let Ok(content) = fs::read_to_string(&skip_file) {
            content
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            DEFAULT_SKIP_COMMANDS
                .iter()
                .map(|s| s.to_string())
                .collect()
        }
    }

    /// Save skip commands to file
    pub fn save_skip_commands(&self, commands: &[String]) -> anyhow::Result<()> {
        fs::create_dir_all(&self.config_dir)?;
        let content = commands.join("\n");
        fs::write(&self.skip_commands_file(), content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::env;
    use std::sync::Mutex;
    use tempfile::tempdir;

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn test_config_dir_env_var() {
        let _lock = ENV_LOCK.lock().unwrap();
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().join("tellme_config");

        unsafe {
            env::set_var("TELLME_CONFIG_DIR", &path);
        }

        assert_eq!(Config::config_dir(), path);

        unsafe {
            env::remove_var("TELLME_CONFIG_DIR");
        }
    }

    #[test]
    fn test_temp_dir_env_var() {
        let _lock = ENV_LOCK.lock().unwrap();
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().join("tellme");

        unsafe {
            env::set_var("TELLME_TEMP_DIR", &path);
        }

        assert_eq!(Config::temp_dir(), path);

        unsafe {
            env::remove_var("TELLME_TEMP_DIR");
        }
    }

    #[test]
    fn test_shell_pid_env_var() {
        let _lock = ENV_LOCK.lock().unwrap();

        unsafe {
            env::set_var("TELLME_SHELL_PID", "12345");
        }

        assert_eq!(Config::shell_pid().unwrap(), 12345)
    }

    #[test]
    fn test_file_paths() {
        let tmp_dir = tempdir().unwrap().path().join("tellme");

        let config = Config::with_paths(Some(99999), None, Some(tmp_dir.clone())).unwrap();
        assert_eq!(config.cmd_file(), tmp_dir.join(".tellme_cmd_99999"));
        assert_eq!(config.output_file(), tmp_dir.join(".tellme_output_99999"));
    }

    #[test]
    fn test_skip_commands_default() {
        let temp_config = tempdir().unwrap().path().join("tellme_config");

        let config = Config::with_paths(Some(99999), Some(temp_config), None).unwrap();
        let skip_commands = config.skip_commands();

        assert!(skip_commands.contains(&"vim".to_string()));
        assert!(skip_commands.contains(&"ssh".to_string()));
        assert!(skip_commands.len() >= DEFAULT_SKIP_COMMANDS.len());
    }

    #[test]
    fn test_skip_commands_custom() {
        let temp_config = tempdir().unwrap().path().join("tellme_config");

        // Create custom skip commands file
        fs::create_dir_all(&temp_config).unwrap();
        let skip_file = temp_config.join("skip_commands");
        fs::write(&skip_file, "custom_cmd\nanother_cmd\n").unwrap();

        let config = Config::with_paths(Some(99999), Some(temp_config), None).unwrap();
        let skip_commands = config.skip_commands();

        assert!(skip_commands.contains(&"custom_cmd".to_string()));
        assert!(skip_commands.contains(&"another_cmd".to_string()));
        // Should not contain defaults when custom file exists
        assert!(!skip_commands.contains(&"vim".to_string()));
    }

    #[test]
    fn test_set_recording_enabled() {
        let temp_config = tempdir().unwrap().path().join("tellme_config");

        let mut config = Config::with_paths(Some(99999), Some(temp_config), None).unwrap();

        // Initially should be disabled (or whatever status file says)
        let initial_status = config.is_recording_enabled();

        // Toggle
        config.set_recording_enabled(!initial_status).unwrap();
        assert_ne!(config.is_recording_enabled(), initial_status);

        // Toggle back
        config.set_recording_enabled(initial_status).unwrap();
        assert_eq!(config.is_recording_enabled(), initial_status);
    }
}

#[cfg(test)]
pub(crate) mod test_utils {
    use super::*;
    use tempfile::{TempDir, tempdir};

    pub fn create_test_config() -> Config {
        let temp_dir = tempdir().unwrap();

        Config::with_paths(
            Some(99999),
            Some(temp_dir.path().join("my_tellme_config")),
            Some(temp_dir.path().join("tellme")),
        )
        .unwrap()
    }

    pub fn create_test_config_with_tempdir() -> (Config, TempDir) {
        let temp_dir = tempdir().unwrap();

        let config = Config::with_paths(
            Some(99999),
            Some(temp_dir.path().join("my_tellme_config")),
            Some(temp_dir.path().join("tellme")),
        )
        .unwrap();

        (config, temp_dir)
    }
}
