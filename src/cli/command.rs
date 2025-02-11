use serde::{Deserialize, Serialize};
use anyhow::{Result, Error};

use crate::commons::utility::run_attempt;

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
}

impl Default for CommandLine {
    fn default() -> Self {
        CommandLine {
            command: "".to_string(), 
            arguments: vec![], 
            interpreter: None
        }
    }
}

impl CommandLine {
    pub fn new(
        command: String, 
        arguments: Vec<String>,
        interpreter: Option<Interpreter>
    ) -> Self {
        Self { command, arguments, interpreter }
    }
    /// Constructs a Tokio process command to execute the configured program.
    ///
    /// It determines the interpreter to use based on the user specification. 
    ///
    /// Additionally, if the `environment_variables_override` field is set, its environment variables
    /// are applied to the command.
    pub fn get_process_command(&self) -> tokio::process::Command {
        let mut command: tokio::process::Command = match self.interpreter {
            Some(Interpreter::Sh) => {
                // Use `sh` if the user has specified. 
                let mut cmd = tokio::process::Command::new("sh");
                let command_line: String =
                    format!("{} {}", self.get_command(), self.get_arguments().join(" "));
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

    pub fn revise_argument(&mut self, argument_index: usize, new_argument: String) {
        self.arguments[argument_index] = new_argument;
    }

    pub fn get_command(&self) -> &str {
        &self.command
    }

    pub fn get_arguments(&self) -> &Vec<String> {
        &self.arguments
    }
}

impl Execution for CommandLine {
    fn get_execution_type(&self) -> &ExecutionType {
        &ExecutionType::CommandLine
    }

    async fn execute(&mut self) -> Result<String, Error> {
        let (mut status, mut output_stdout) = run_attempt(
            self
        ).await;

        Ok(output_stdout)
    }
}

impl std::fmt::Display for CommandLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.command, self.arguments.join(" "))
    }
}