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
    pub exit_on_failure: bool,
    pub execute_on_failure: Option<CommandLine>,
}

impl Default for FailureHandlingOptions {
    fn default() -> Self {
        Self {
            exit_on_failure: true,
            execute_on_failure: None
        }
    }
}
