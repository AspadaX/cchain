use std::{collections::HashMap, str::FromStr};

use anyhow::{anyhow, Error};
use serde::{Deserialize, Serialize};

use crate::{
    commons::utility::run_attempt, display_control::{display_message, Level}, variable::Variable
};

use super::{command::CommandLine, interpreter::Interpreter, options::{FailureHandlingOptions, StdoutStorageOptions}, traits::{Execution, ExecutionType}};

#[derive(Debug, Deserialize, Serialize)]
pub struct Program {
    #[serde(flatten)]
    command_line: CommandLine,
    /// Optional environment variable overrides.
    /// Each entry maps a variable name to its override value for this
    /// execution.
    environment_variables_override: Option<HashMap<String, String>>,
    /// Optional variable name where the standard output of the program
    /// will be stored.
    stdout_stored_to: Option<String>,
    /// Additional conditions when storaging the stdout to a variable
    #[serde(default)]
    stdout_storage_options: StdoutStorageOptions,
    /// Failure handling options
    #[serde(default)]
    failure_handling_options: FailureHandlingOptions,
    /// Retry policy for executing the command.
    ///
    /// Use -1 to retry indefinitely, or any non-negative value to specify
    /// the maximum number of retries.
    retry: i32,
}

impl Program {
    pub fn new(
        command: String,
        arguments: Vec<String>,
        environment_variables_override: Option<HashMap<String, String>>,
        stdout_stored_to: Option<String>,
        stdout_storage_options: StdoutStorageOptions,
        interpreter: Option<Interpreter>,
        failure_handling_options: FailureHandlingOptions,
        retry: i32,
    ) -> Self {
        Program {
            command_line: CommandLine::new(command, arguments, interpreter),
            environment_variables_override,
            stdout_stored_to,
            stdout_storage_options,
            failure_handling_options,
            retry,
        }
    }

    /// Inserts provided variables into the program's arguments.
    ///
    /// This method iterates over each argument in the program and replaces occurrences of
    /// raw variable names with their corresponding values. If retrieving the value of a variable
    /// fails, it returns an error.
    ///
    /// # Arguments
    ///
    /// * `variables` - A vector of `Variable` instances whose raw names will be replaced with their values.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if all variables are inserted successfully, or an `Error` if any variable's
    /// value retrieval fails.
    pub fn insert_variable(&mut self, variables: &Vec<Variable>) -> Result<(), Error> {
        for argument in self.command_line.get_arguments().iter_mut() {
            for variable in variables {
                if argument.contains(variable.get_raw_variable_name().as_str()) {
                    *argument = argument.replace(
                        variable.get_raw_variable_name().as_str(),
                        &variable.get_value()?,
                    );
                }
            }
        }

        Ok(())
    }

    pub fn get_retry(&self) -> &i32 {
        &self.retry
    }

    pub fn get_awaitable_variable(&self) -> &Option<String> {
        &self.stdout_stored_to
    }

    /// In-place operation on the stdout string. 
    /// Directly apply the stdout storage options.
    fn apply_stdout_storage_options(&self, stdout_string: &mut String) {
        if self.stdout_storage_options.without_newline_characters {
            *stdout_string = stdout_string.trim_matches('\n').to_string();
        }
    }

    fn apply_failure_handling_options(&self, error_message: String) -> Result<(), Error> {
        if self.failure_handling_options.exit_on_failure {
            display_message(Level::Error, &error_message);
            return Err(anyhow!("{}", error_message));
        }

        Ok(())
    }
}

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.command_line)
    }
}

impl FromStr for Program {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() < 2 {
            return Err("Invalid configuration".to_string());
        }

        let command = parts[0].to_string();
        let arguments = parts[1..].iter().map(|s| s.to_string()).collect();

        Ok(
            Self {
            command_line: CommandLine::new(command, arguments, None),
            ..Default::default()
            }
        )
    }
}

impl Execution for Program {
    fn get_execution_type(&self) -> &ExecutionType {
        &ExecutionType::Program
    }

    async fn execute(&mut self) -> Result<String, anyhow::Error> {
        // First attempt
        let (mut status, mut output_stdout) = run_attempt(self).await;
        let mut attempts = 0;

        // Retry loop for a fixed number of attempts (or unlimited if retry == -1)
        while !status.success() && (self.get_retry() == &-1 || &attempts < self.get_retry()) {
            attempts += 1;
            display_message(
                Level::Warn, 
                &format!(
                    "Retrying {}: {}, attempt: {}",
                    self.get_execution_type(),
                    &self,
                    attempts
                )
            );
            let (s, out) = run_attempt(self).await;
            status = s;
            output_stdout = out;

            if !status.success() && self.get_retry() != &-1 && &attempts >= self.get_retry() {
                let error_message: String = format!(
                    "Failed to execute {}: {}", self.get_execution_type(), &self
                );
                display_message(
                    Level::ProgramOutput, 
                    &format!("Process output:\n{}", output_stdout)
                );

                self.apply_stdout_storage_options(&mut output_stdout);

                if let Err(result) = self.apply_failure_handling_options(error_message) {
                    return Err(result);
                } else {
                    return Ok(output_stdout)
                }
            }
        }

        // For an indefinite retry (retry == -1), keep trying until the process succeeds
        if !status.success() && self.get_retry() == &-1 {
            loop {
                attempts += 1;
                display_message(
                    Level::Warn, 
                    &format!(
                        "Retrying {}: {}, attempt: {}",
                        self.get_execution_type(),
                        &self,
                        attempts
                    )
                );
                let (s, out) = run_attempt(self).await;
                status = s;
                output_stdout = out;
                if status.success() {
                    break;
                }
            }
        }

        // If retry is set to 0, we shouldn’t retry.
        if !status.success() && self.get_retry() == &0 {
            let error_message: String = format!(
                "Failed to execute {}: {}\n",
                self.get_execution_type(),
                &self
            );
            display_message(
                Level::ProgramOutput, 
                &format!("Process output:\n{}", output_stdout)
            );
            self.apply_stdout_storage_options(&mut output_stdout);

            if let Err(result) = self.apply_failure_handling_options(error_message) {
                return Err(result);
            } else {
                return Ok(output_stdout)
            }
        }

        // Log separation / final output, using the collected output as needed.
        display_message(
            Level::Logging, 
            &format!("Finished executing command: {}", &self)
        );

        self.apply_stdout_storage_options(&mut output_stdout);

        Ok(output_stdout)
    }
}

impl Default for Program {
    fn default() -> Self {
        Self {
            command_line: CommandLine::default(),
            environment_variables_override: Some(HashMap::new()),
            stdout_stored_to: None,
            stdout_storage_options: StdoutStorageOptions::default(),
            failure_handling_options: FailureHandlingOptions::default(),
            retry: 0,
        }
    }
}