use std::{collections::HashMap, str::FromStr};

use anyhow::{anyhow, Error};
use serde::{Deserialize, Serialize};

use crate::{display_control::{display_message, Level}, function::Function};

use super::{command::CommandLine, interpreter::Interpreter, options::{FailureHandlingOptions, StdoutStorageOptions}, traits::{Execution, ExecutionType}};

#[derive(Debug, Deserialize, Serialize)]
pub struct Program {
    #[serde(flatten)]
    command_line: CommandLine,
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
            command_line: CommandLine::new(
                command, 
                arguments, 
                interpreter,
                environment_variables_override,
            ),
            stdout_stored_to,
            stdout_storage_options,
            failure_handling_options,
            retry,
        }
    }

    pub fn get_retry(&self) -> &i32 {
        &self.retry
    }

    pub fn get_awaitable_variable(&self) -> &Option<String> {
        &self.stdout_stored_to
    }
    
    pub fn get_command_line(&mut self) -> &mut CommandLine {
        &mut self.command_line
    }
    
    pub fn get_remedy_command_line(&mut self) -> Option<&mut CommandLine> {
        if let Some(command_line) = &mut self
            .failure_handling_options
            .remedy_command_line 
        {
            return Some(command_line);
        }
        
        None
    }

    /// In-place operation on the stdout string. 
    /// Directly apply the stdout storage options.
    fn apply_stdout_storage_options(&self, stdout_string: &mut String) {
        if self.stdout_storage_options.without_newline_characters {
            *stdout_string = stdout_string.trim_matches('\n').to_string();
        }
    }

    pub fn get_failure_handling_options(&mut self) -> &mut FailureHandlingOptions {
        &mut self.failure_handling_options
    }
    
    pub async fn execute_argument_functions(&mut self) -> Result<(), Error> {
        // Iterate over each argument in the configuration
        for index in 0..self.command_line.get_arguments().len() {
            // Clone the current argument
            let argument: String = self.command_line.get_arguments()[index].clone();
    
            // Attempt to parse the argument as a function
            let function = match Function::from_str(&argument) {
                Ok(f) => f,
                Err(_) => continue, // If parsing fails, skip to the next argument
            };
    
            display_message(
                Level::Logging, 
                &format!(
                    "Detected function, {}, when executing command: {}, executing the function...",
                    function.get_name(),
                    self.command_line
                )
            );
    
            // Execute the function asynchronously and await the result
            let result: String = function.execute().await?;
            self.command_line.revise_argument_by_index(index, result);
            display_message(
                Level::Logging, 
                &format!(
                    "Function, {}, executed successfully", 
                    function.get_name()
                )
            );
        }
        // Return the result of the function execution
        Ok(())
    }
    
    /// This method is supposed to be called when the program fails
    pub async fn execute_remedy_command_line(&mut self) -> Result<(), Error> {
        if let Some(command_line) = &mut self
            .failure_handling_options
            .remedy_command_line
        {
            command_line.execute().await?;
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
            command_line: CommandLine::new(
                command, 
                arguments, 
                None,
                None
            ),
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
        let mut attempts: i32 = 0;
        // In the case of retry==0 we never retry, so our only chance is the first attempt.
        // For retry == -1, we reattempt indefinitely.
        loop {
            // Attempt execution through the commandline’s execute method.
            match self.command_line.execute().await {
                Ok(mut output_stdout) => {
                    // On success: apply any stdout storage options
                    self.apply_stdout_storage_options(&mut output_stdout);
                    return Ok(output_stdout);
                },
                Err(err) => {
                    // If retry number is set to 0, 
                    // it should not display the retry messages.
                    if self.retry == 0 {
                        return Err(err);
                    }
                    // Increase attempt counter.
                    attempts += 1;
                    let warn_msg = format!(
                        "Retrying {}: {}, attempt: {}",
                        self.get_execution_type(),
                        &self,
                        attempts
                    );
                    display_message(Level::Warn, &warn_msg);

                    // Determine if we should break the retry loop.
                    // (retry 0 means no retries; any non-negative value means that many attempts;
                    // -1 means unlimited retries.)
                    if self.retry == 0 || (self.retry != -1 && attempts >= self.retry) {
                        // No stdout storage options to apply, as no output was captured.
                        self.execute_remedy_command_line().await?;
                        return Err(err);
                    }
                    // Otherwise, we loop again.
                }
            }
        }
    }
}

impl Default for Program {
    fn default() -> Self {
        Self {
            command_line: CommandLine::default(),
            stdout_stored_to: None,
            stdout_storage_options: StdoutStorageOptions::default(),
            failure_handling_options: FailureHandlingOptions::default(),
            retry: 0,
        }
    }
}