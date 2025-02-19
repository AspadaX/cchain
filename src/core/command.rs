use std::{collections::HashMap, io::{BufReader, Read}, process::{Command, Stdio}, sync::mpsc, thread, time::Duration,};

use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

use crate::display_control::{display_message, display_program_output, Level};

use super::{
    interpreter::Interpreter,
    traits::{Execution, ExecutionType},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandLineExecutionResult {
    output: String,
}

impl CommandLineExecutionResult {
    pub fn new(output: String) -> Self {
        Self { output }
    }

    pub fn get_output(&self) -> String {
        self.output.clone()
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
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
            environment_variables_override: None,
        }
    }
}

impl CommandLine {
    pub fn new(
        command: String,
        arguments: Vec<String>,
        interpreter: Option<Interpreter>,
        environment_variables_override: Option<HashMap<String, String>>,
    ) -> Self {
        Self {
            command,
            arguments,
            interpreter,
            environment_variables_override,
        }
    }
    /// Constructs a Tokio process command to execute the configured program.
    ///
    /// It determines the interpreter to use based on the user specification.
    ///
    /// Additionally, if the `environment_variables_override` field is set, its environment variables
    /// are applied to the command.
    pub fn get_process_command(&mut self) -> Command {
        let mut command: Command = match self.interpreter {
            Some(Interpreter::Sh) => {
                // Use `sh` if the user has specified.
                let mut cmd = Command::new("sh");
                let command_line: String = {
                    let command: String = self.get_command().to_string();
                    let arguments: String = self.get_arguments().join(" ");
                    format!("{} {}", command, arguments)
                };
                cmd.arg("-c").arg(command_line);
                cmd
            }
            _ => {
                // On non-Unix systems, execute the command directly.
                let mut cmd = Command::new(self.get_command());
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

    pub fn inject_value_to_variables(
        &mut self,
        raw_variable_name: &str,
        value: String,
    ) -> Result<(), Error> {
        for argument in &mut self.arguments {
            if argument.contains(raw_variable_name) {
                *argument = argument.replace(raw_variable_name, &value);
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

impl Execution<CommandLineExecutionResult> for CommandLine {
    fn get_execution_type(&self) -> &ExecutionType {
        &ExecutionType::CommandLine
    }

    fn execute(&mut self) -> Result<Vec<CommandLineExecutionResult>, Error> {
        let mut command = self.get_process_command();

        // Set stdout to piped so that we can capture it
        command.stdout(Stdio::piped());
        display_message(
            Level::Logging,
            &format!("Start executing command: {}", &self),
        );
    
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
    
        // Create a channel to receive output from the reader thread
        let (tx, rx) = mpsc::channel();
    
        // Spawn a thread to read and send the output
        let reader_handle = thread::spawn(move || -> Result<(), Error> {
            let mut reader = BufReader::new(stdout);
            let mut buffer = [0; 1024];
            loop {
                let n = reader.read(&mut buffer).map_err(|e| Error::msg(format!("Failed to read stdout: {}", e)))?;
                if n == 0 {
                    break; // EOF
                }
                let chunk = &buffer[..n];
                let text = String::from_utf8_lossy(chunk).to_string();
                tx.send(text).map_err(|e| Error::msg(format!("Failed to send output: {}", e)))?;
            }
            Ok(())
        });
    
        let mut collected_output = String::new();
    
        // Loop to receive and display output from the channel
        loop {
            // Try to receive output with a timeout
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(text) => {
                    display_program_output(&text);
                    collected_output.push_str(&text);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Check if the child process has exited
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            if !status.success() {
                                return Err(Error::msg(format!(
                                    "Process exited with non-zero status: {}",
                                    status
                                )));
                            }
                            break; // Process finished successfully
                        }
                        Ok(None) => {
                            // Process still running
                            continue;
                        }
                        Err(e) => {
                            return Err(Error::msg(format!("Failed to wait on child process: {}", e)));
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    // Reader thread has finished
                    break;
                }
            }
        }
    
        // Ensure the reader thread has finished
        reader_handle.join().map_err(|e| Error::msg(format!("Reader thread panicked: {:?}", e)))??;
    
        // Display a message when the command finished execution successfully
        display_message(
            Level::Logging,
            &format!("Finished executing command: {}", &self),
        );
    
        Ok(vec![CommandLineExecutionResult::new(collected_output)])
    }
}

impl std::fmt::Display for CommandLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.command, self.arguments.join(" "))
    }
}
