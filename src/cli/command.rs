use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use anyhow::{Result, Error};
use tokio::{io::{AsyncBufReadExt, BufReader}, task::JoinHandle};

use crate::display_control::{display_message, Level};

use super::{interpreter::Interpreter, traits::{Execution, ExecutionType}};


#[derive(Debug, Deserialize, Serialize)]
pub struct CommandLine {
    /// The command to execute.
    /// This should be the path or name of the program.
    command: String,
    /// A list of arguments to pass to the program.
    arguments: Vec<String>,
    /// Allow for declaring the type of interpreter to use when 
    /// running a command.
    interpreter: Option<Interpreter>,
    /// Optional environment variable overrides.
    /// Each entry maps a variable name to its override value for this
    /// execution.
    environment_variables_override: Option<HashMap<String, String>>,
}

impl Default for CommandLine {
    fn default() -> Self {
        CommandLine {
            command: "".to_string(), 
            arguments: vec![], 
            interpreter: None,
            environment_variables_override: None
        }
    }
}

impl CommandLine {
    pub fn new(
        command: String, 
        arguments: Vec<String>,
        interpreter: Option<Interpreter>,
        environment_variables_override: Option<HashMap<String,String>>
    ) -> Self {
        Self { 
            command, 
            arguments, 
            interpreter, 
            environment_variables_override
        }
    }
    /// Constructs a Tokio process command to execute the configured program.
    ///
    /// It determines the interpreter to use based on the user specification. 
    ///
    /// Additionally, if the `environment_variables_override` field is set, its environment variables
    /// are applied to the command.
    pub fn get_process_command(&mut self) -> tokio::process::Command {
        let mut command: tokio::process::Command = match self.interpreter {
            Some(Interpreter::Sh) => {
                // Use `sh` if the user has specified. 
                let mut cmd = tokio::process::Command::new("sh");
                let command_line: String = {
                    let command: String = self.get_command().to_string();
                    let arguments: String = self.get_arguments().join(" ");
                    format!("{} {}", command, arguments)
                };
                cmd.arg("-c").arg(command_line);
                cmd
            },
            _ => {
                // On non-Unix systems, execute the command directly.
                let mut cmd = tokio::process::Command::new(self.get_command());
                cmd.args(self.get_arguments());
                cmd
            }
        };

        // Override environment variables if provided.
        if let Some(ref env_vars) = self.environment_variables_override {
            command.envs(env_vars);
        }

        command
    }

    pub fn revise_argument_by_index(&mut self, argument_index: usize, new_argument: String) {
        self.arguments[argument_index] = new_argument;
    }
    
    pub fn inject_value_to_variables(&mut self, raw_variable_name: &str, value: String) -> Result<(), Error> {
        for argument in &mut self.arguments {
            if argument.contains(raw_variable_name) {
                *argument = argument.replace(
                    raw_variable_name,
                    &value,
                );
            }
        }
        
        Ok(())
    }

    pub fn get_command(&mut self) -> &str {
        &self.command
    }

    pub fn get_arguments(&mut self) -> &mut Vec<String> {
        &mut self.arguments
    }
}

impl Execution for CommandLine {
    fn get_execution_type(&self) -> &ExecutionType {
        &ExecutionType::CommandLine
    }

    async fn execute(&mut self) -> Result<String, Error> {
        let mut command = self.get_process_command();
        
        // Set stdout to piped so that we can capture it
        command.stdout(std::process::Stdio::piped());

        // Spawn the process
        let mut child = command.spawn().map_err(|e| {
            Error::msg(format!(
                "Failed to execute {}: {}",
                self.get_execution_type(),
                e
            ))
        })?;

        // Take the stdout handle
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| Error::msg("Failed to capture stdout"))?;

        // Spawn a concurrent task to read and print the output live
        let reader_handle: JoinHandle<Result<String, Error>> = tokio::spawn(
            async move {
                let mut collected_output = String::new();
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    display_message(
                        Level::ProgramOutput, 
                        &format!("{}", line)
                    );
                    collected_output.push_str(&line);
                    collected_output.push('\n');
                }

                Ok(collected_output)
            }
        );

        // Wait until child terminates (the output task will eventually finish as well)
        let status = child
            .wait()
            .await
            .map_err(|e| Error::msg(format!("Failed to wait on child process: {}", e)))?;

        if !status.success() {
            return Err(Error::msg(format!(
                "Process exited with non-zero status: {}",
                status
            )));
        }

        // Await the reader task to get the collected stdout contents.
        let collected = reader_handle
            .await
            .map_err(|e| Error::msg(format!("Reader task panicked: {}", e)))??;

        Ok(collected)
    }
}

impl std::fmt::Display for CommandLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.command, self.arguments.join(" "))
    }
}