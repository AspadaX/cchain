use std::io::{BufReader, Read};
use std::sync::mpsc::channel;
use std::{collections::HashMap, process::Command};

use anyhow::{Error, Result};
use console::{StyledObject, Term};
use serde::{Deserialize, Serialize};

use crate::display_control::{display_command_line, display_message, Level};

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
    /// Set the working directory for this program. 
    /// Null means the current working directory.
    working_directory: Option<String>,
}

impl Default for CommandLine {
    fn default() -> Self {
        CommandLine {
            command: "".to_string(),
            arguments: vec![],
            interpreter: None,
            environment_variables_override: None,
            working_directory: None,
        }
    }
}

impl CommandLine {
    pub fn new(
        command: String,
        arguments: Vec<String>,
        interpreter: Option<Interpreter>,
        environment_variables_override: Option<HashMap<String, String>>,
        working_directory: Option<String>,
    ) -> Self {
        Self {
            command,
            arguments,
            interpreter,
            environment_variables_override,
            working_directory
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
                // On non-Unix systems and not specified cases, execute the command directly.
                let mut cmd = Command::new(self.get_command());
                cmd.args(self.get_arguments());
                cmd
            }
        };
        
        // Set the working directory for the command.
        if let Some(working_directory) = &self.working_directory {
            command.current_dir(working_directory);
        }

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
        let mut command: Command = self.get_process_command();
        
        // Set stdout to piped so that we can capture it
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());
        let command_in_text: String = format!(r#"{}"#, &self.to_string());
        let command_string: &StyledObject<&String> = &console::style(&command_in_text).bold();
        display_message(
            Level::Logging, 
            &format!("Start executing command: {}", command_string)
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
            .unwrap();
        // Take the stderr handle
        let stderr = child
            .stderr
            .take()
            .unwrap();
        
        let (tx, rx) = channel();
    
        // Spawn a thread to read stdout
        let tx_clone = tx.clone();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            let mut buffer = [0; 1024];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        let text = String::from_utf8_lossy(&buffer[..n]).to_string();
                        tx_clone.send(text).unwrap();
                    },
                    Err(_) => break,
                }
            }
        });
    
        // Spawn a thread to read stderr
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stderr);
            let mut buffer = [0; 1024];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        let text = String::from_utf8_lossy(&buffer[..n]).to_string();
                        tx.send(text).unwrap();
                    },
                    Err(_) => break,
                }
            }
        });
        
        let mut collected_output = String::new();
        let terminal = Term::stdout();
        for received in rx {
            display_command_line(&terminal, &received);
            collected_output.push_str(&received);
        }
    
        // Wait for process completion
        let status = child.wait()
            .map_err(|e| Error::msg(format!("Failed to wait on child process: {}", e)))?;
        
        if !status.success() {
            return Err(Error::msg(format!(
                "Process exited with non-zero status: {}",
                status
            )));
        }
    
        display_message(Level::Logging, &format!("Finished executing command: {}", command_string));
    
        Ok(vec![CommandLineExecutionResult::new(collected_output)])
    }
}

impl std::fmt::Display for CommandLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.command, self.arguments.join(" "))
    }
}
