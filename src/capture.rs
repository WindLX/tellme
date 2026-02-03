use crate::config::Config;
use crate::filter::CommandFilter;
use std::fs;
use std::path::PathBuf;

/// Manages temporary output capture files
///
/// Handles creation, rotation, and cleanup of temporary files
/// used to capture command output in the shell.
#[derive(Debug)]
pub struct CaptureSession<'s> {
    /// Configuration
    config: &'s Config,
}

impl<'s> CaptureSession<'s> {
    /// Create a new CaptureSession
    pub fn new(config: &'s Config) -> Self {
        Self { config }
    }

    /// Create command file
    ///
    /// Returns the path to the command file.
    fn create_cmd_file(&self, command: &str) -> anyhow::Result<PathBuf> {
        let path = self.config.cmd_file();
        // Ensure the parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, command)?;

        Ok(path)
    }

    /// Create output file
    ///
    /// Returns the path to the output file.
    fn create_output_file(&self) -> anyhow::Result<PathBuf> {
        let path = self.config.output_file();
        // Ensure the parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::File::create(&path)?;

        Ok(path)
    }

    /// Read the command
    pub fn read_cmd_file(&self) -> anyhow::Result<String> {
        fs::read_to_string(&self.config.cmd_file()).map_err(|e| anyhow::anyhow!(e))
    }

    /// Read the last captured command output
    pub fn read_output(&self) -> anyhow::Result<Vec<u8>> {
        fs::read(&self.config.output_file()).map_err(|e| anyhow::anyhow!(e))
    }

    /// Check if there's a previous command to capture
    pub fn has_previous(&self) -> bool {
        self.config.cmd_file().exists() && self.config.output_file().exists()
    }

    pub fn should_prepare(&self, command: &str) -> bool {
        // Check if recording is enabled
        if !self.config.is_recording_enabled() {
            return false;
        }

        // Check if this command should be captured
        let filter = CommandFilter::new(self.config);
        filter.should_capture(command)
    }

    /// Prepare session for a new command
    ///
    /// This handles:
    /// 1. Creating new current files
    /// 2. Returning paths for the shell to use
    pub fn prepare_new_command(&self, command: &str) -> anyhow::Result<PathBuf> {
        // Create new files
        self.create_cmd_file(command)?;
        self.create_output_file()?;

        Ok(self.config.output_file())
    }

    /// Clean up all temporary files for this session
    pub fn cleanup(&self) -> anyhow::Result<()> {
        for file in self.config.temp_files() {
            if file.exists() {
                fs::remove_file(file)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::test_utils::{create_test_config, create_test_config_with_tempdir};

    #[test]
    fn test_create_cmd_file() {
        let config = create_test_config();
        let session = CaptureSession::new(&config);

        let cmd_path = session.create_cmd_file("make build").unwrap();
        let content = std::fs::read_to_string(&cmd_path).unwrap();
        assert_eq!(content, "make build");
    }

    #[test]
    fn test_cleanup() {
        let config = create_test_config();
        let session = CaptureSession::new(&config);

        // Create files
        session.create_cmd_file("test").unwrap();

        // All should exist
        assert!(config.cmd_file().exists());

        // Cleanup
        session.cleanup().unwrap();

        // All should be gone
        assert!(!config.cmd_file().exists());
    }

    #[test]
    fn test_has_previous() {
        let config = create_test_config();
        let session = CaptureSession::new(&config);

        // Initially no previous
        assert!(!session.has_previous());

        // Create
        session.create_cmd_file("test").unwrap();
        fs::write(config.output_file(), b"output").unwrap();

        // Now should have previous
        assert!(session.has_previous());
    }

    #[test]
    fn test_read_cmd_file() {
        let config = create_test_config();
        let session = CaptureSession::new(&config);

        // Create current files and write content
        session.create_cmd_file("last cmd").unwrap();
        fs::write(config.output_file(), b"last output").unwrap();

        // Read last log and command
        let last_cmd = session.read_cmd_file().unwrap();
        let last_output = session.read_output().unwrap();
        assert_eq!(last_cmd, "last cmd");
        assert_eq!(last_output, b"last output");
    }

    #[test]
    fn test_should_prepare() {
        let (mut config, _temp_dir) = create_test_config_with_tempdir();

        {
            let session = CaptureSession::new(&config);
            // By default, recording is disabled
            assert!(!session.should_prepare("make build"));
        }

        // Enable recording
        config.set_recording_enabled(true).unwrap();

        let session = CaptureSession::new(&config);
        // Now should prepare for normal commands
        assert!(session.should_prepare("make build"));
        // Should not prepare for skipped commands
        assert!(!session.should_prepare("vim file.txt"));
    }

    #[test]
    fn test_prepare_new_command() {
        let (mut config, _temp_dir) = create_test_config_with_tempdir();

        config.set_recording_enabled(true).unwrap();
        let session = CaptureSession::new(&config);
        let log_path = session.prepare_new_command("cargo test").unwrap();
        assert!(log_path.exists());
        let cmd_content = std::fs::read_to_string(config.cmd_file()).unwrap();
        assert_eq!(cmd_content, "cargo test");
    }
}
