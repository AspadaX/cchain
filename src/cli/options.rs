use serde::{Deserialize, Serialize};

use super::command::CommandLine;


#[derive(Debug, Deserialize, Serialize)]
pub struct StdoutStorageOptions {
    pub without_newline_characters: bool
}

impl Default for StdoutStorageOptions {
    fn default() -> Self {
        Self {
            without_newline_characters: true
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FailureHandlingOptions {
    /// Indicates whether the chain will exit when a failure is captured
    pub exit_on_failure: bool,
    /// A command line to execute when the program fails and will exit
    /// This provides a remedy measure. For example, when a git commit 
    /// fails, it allows you to `git reset` the commit for starting 
    /// a new commit after fixing the issues
    pub remedy_command_line: Option<CommandLine>,
}

impl Default for FailureHandlingOptions {
    fn default() -> Self {
        Self {
            exit_on_failure: true,
            remedy_command_line: None
        }
    }
}
