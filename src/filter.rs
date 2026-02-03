use crate::config::Config;

/// Command filtering logic for tellme
///
/// Determines whether a command should have its output captured
/// based on a list of skip commands.
#[derive(Debug, Clone)]
pub struct CommandFilter {
    /// List of commands to skip
    skip_commands: Vec<String>,
}

impl CommandFilter {
    /// Create a new CommandFilter with default skip commands
    pub fn new(config: &Config) -> Self {
        Self {
            skip_commands: config.skip_commands(),
        }
    }

    /// Check if a command should be captured
    ///
    /// Returns `true` if the command output should be captured,
    /// `false` if it should be skipped.
    pub fn should_capture(&self, command: &str) -> bool {
        if command.trim().is_empty() {
            return false;
        }

        // Get the base command (first word)
        let base_cmd = command.split_whitespace().next().unwrap_or("");

        // Check if any skip pattern matches
        // Support both exact match and prefix match
        !self.skip_commands.iter().any(|skip| {
            if skip.ends_with('*') {
                // Prefix match (e.g., "git*" matches "git status")
                let prefix = &skip[..skip.len() - 1];
                base_cmd.starts_with(prefix)
            } else {
                // Exact match
                base_cmd == skip
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::test_utils::create_test_config;

    #[test]
    fn test_should_capture_default_commands() {
        let config = create_test_config();
        let filter = CommandFilter::new(&config);

        // These should NOT be captured (skipped)
        assert!(!filter.should_capture("tellme"));
        assert!(!filter.should_capture("clear"));
        assert!(!filter.should_capture("cd /tmp"));
        assert!(!filter.should_capture("vim file.txt"));
        assert!(!filter.should_capture("ssh user@host"));
        assert!(!filter.should_capture("exit"));
    }

    #[test]
    fn test_should_capture_normal_commands() {
        let config = create_test_config();
        let filter = CommandFilter::new(&config);

        // These should be captured
        assert!(filter.should_capture("make build"));
        assert!(filter.should_capture("cargo test"));
        assert!(filter.should_capture("echo hello"));
        assert!(filter.should_capture("ls -la"));
        assert!(filter.should_capture("pytest -v tests/"));
    }

    #[test]
    fn test_empty_command() {
        let config = create_test_config();
        let filter = CommandFilter::new(&config);

        assert!(!filter.should_capture(""));
        assert!(!filter.should_capture("   "));
    }
}
