use std::{collections::HashMap, str::FromStr};

use anyhow::Error;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::{
    utility::{Execution, ExecutionType},
    variable::Variable,
};

/// Currently supported interpreters
#[derive(Debug, Deserialize, Serialize, Eq, PartialEq, PartialOrd)]
pub enum Interpreter {
    #[serde(alias = "sh")]
    Sh,
}

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
    pub continue_on_failure: bool,
}

impl Default for FailureHandlingOptions {
    fn default() -> Self {
        Self { continue_on_failure: false }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Program {
    /// The command to execute.
    /// This should be the path or name of the program.
    command: String,
    /// A list of arguments to pass to the program.
    arguments: Vec<String>,
    /// Optional environment variable overrides.
    /// Each entry maps a variable name to its override value for this
    /// execution.
    environment_variables_override: Option<HashMap<String, String>>,
    /// Optional variable name where the standard output of the program
    /// will be stored.
    stdout_stored_to: Option<String>,
    /// Additional conditions when storaging the stdout to a variable
    stdout_storage_options: Option<StdoutStorageOptions>,
    /// Allow for declaring the type of interpreter to use when 
    /// running a command.
    interpreter: Option<Interpreter>,
    /// Failure handling options
    failure_handling_options: Option<FailureHandlingOptions>,
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
        stdout_storage_options: Option<StdoutStorageOptions>,
        interpreter: Option<Interpreter>,
        failure_handling_options: Option<FailureHandlingOptions>,
        retry: i32,
    ) -> Self {
        Program {
            command,
            arguments,
            environment_variables_override,
            stdout_stored_to,
            stdout_storage_options,
            interpreter,
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
        for argument in self.arguments.iter_mut() {
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

    pub fn revise_argument(&mut self, argument_index: usize, new_argument: String) {
        self.arguments[argument_index] = new_argument;
    }

    pub fn get_command(&self) -> &str {
        &self.command
    }

    pub fn get_arguments(&self) -> &Vec<String> {
        &self.arguments
    }

    pub fn get_retry(&self) -> &i32 {
        &self.retry
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

    pub fn get_awaitable_variable(&self) -> &Option<String> {
        &self.stdout_stored_to
    }

    /// In-place operation on the stdout string. 
    /// Directly apply the stdout storage options.
    fn apply_stdout_storage_options(&self, stdout_string: &mut String) {
        if let Some(options) = &self.stdout_storage_options {
            if options.without_newline_characters {
                *stdout_string = stdout_string.trim_matches('\n').to_string();
            }
        }
    }

    fn apply_failure_handling_options(&self, error_message: String) {
        if let Some(options) = &self.failure_handling_options {
            if options.continue_on_failure {
                error!("{}", error_message);
                panic!();
            }
        }
    }
}

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.command, self.arguments.join(" "))
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

        Ok(Self {
            command,
            arguments,
            ..Default::default()
        })
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
            warn!(
                "Retrying {}: {}, attempt: {}",
                self.get_execution_type(),
                &self,
                attempts
            );
            let (s, out) = run_attempt(self).await;
            status = s;
            output_stdout = out;

            if !status.success() && self.get_retry() != &-1 && &attempts >= self.get_retry() {
                let error_message: String = format!(
                    "Failed to execute {}: {}", self.get_execution_type(), &self
                );
                info!("Process output:\n{}", output_stdout);

                self.apply_stdout_storage_options(&mut output_stdout);
                self.apply_failure_handling_options(error_message);

                return Ok(output_stdout);
            }
        }

        // For an indefinite retry (retry == -1), keep trying until the process succeeds
        if !status.success() && self.get_retry() == &-1 {
            loop {
                attempts += 1;
                warn!(
                    "Retrying {}: {}, attempt: {}",
                    self.get_execution_type(),
                    &self,
                    attempts
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
            info!("Process output:\n{}", output_stdout);
            self.apply_stdout_storage_options(&mut output_stdout);
            self.apply_failure_handling_options(error_message);

            return Ok(output_stdout);
        }

        // Log separation / final output, using the collected output as needed.
        info!("===============================");
        info!("Finished executing command: {}", &self);
        info!("===============================");

        self.apply_stdout_storage_options(&mut output_stdout);

        Ok(output_stdout)
    }
}

// A helper async function that spawns the process,
// concurrently streams stdout to the terminal (via println!) and
// collects it into a String.
async fn run_attempt(program: &mut Program) -> (std::process::ExitStatus, String) {
    let mut command = program.get_process_command();

    // Set stdout to piped so that we can capture it
    command.stdout(std::process::Stdio::piped());

    // Spawn the process
    let mut child = command.spawn().expect(&format!(
        "Failed to execute {}",
        program.get_execution_type()
    ));

    // Take the stdout handle
    let stdout = child.stdout.take().expect("Failed to capture stdout");

    // Spawn a concurrent task to read and print the output live
    let reader_handle = tokio::spawn(async move {
        let mut collected_output = String::new();
        // Wrap stdout in a BufReader and read it line by line
        let mut reader = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            // Print output live to the screen
            println!("{}", line);
            // Append to the collected output variable (plus a newline)
            collected_output.push_str(&line);
            collected_output.push('\n');
        }
        collected_output
    });

    // Wait until child terminates (the output task will eventually finish as well)
    let status = child.wait().await.expect("Failed to wait on child");
    // Await the reader task to get the collected stdout contents.
    let collected = reader_handle.await.expect("Reader task panicked");

    (status, collected)
}

impl Default for Program {
    fn default() -> Self {
        Self {
            command: "".to_string(),
            arguments: vec![],
            environment_variables_override: Some(HashMap::new()),
            stdout_stored_to: None,
            stdout_storage_options: None,
            interpreter: None,
            failure_handling_options: Some(FailureHandlingOptions::default()),
            retry: 0,
        }
    }
}