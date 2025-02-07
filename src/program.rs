use std::{collections::HashMap, path::Display, str::FromStr};

use log::{error, info, warn};
use serde::{Deserialize, Serialize};

use crate::utility::{Execution, ExecutionType};

#[derive(Deserialize, Serialize)]
pub struct Program {
    command: String,
    arguments: Vec<String>,
    environment_variables_override: HashMap<String, String>,
    retry: i32
}

impl Program {
    pub fn new(
        command: String, 
        arguments: Vec<String>, 
        environment_variables_override: HashMap<String, String>, 
        retry: i32
    ) -> Self {
        Program {
            command,
            arguments,
            environment_variables_override,
            retry,
        }
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

        Ok(
            Program::new(
                command, 
                arguments, 
                HashMap::new(),
                0
            )
        )
    }
}

impl Execution for Program {
    fn get_execution_type(&self) -> &ExecutionType {
        &ExecutionType::Program
    }

    fn execute(&self) -> anyhow::Result<(), anyhow::Error> {
        // Create a new command based on the configuration
        let mut command = std::process::Command::new(
            self.get_command()
        );
        // Add the arguments to the command
        command.args(self.get_arguments());

        // Spawn the command as a child process
        let mut child = command.spawn().expect(
            &format!("Failed to execute {}", self.get_execution_type())
        );
        // Wait for the child process to finish
        let status = child.wait().expect("Failed to wait on child");

        // If the command failed and retry is enabled, try to execute it again
        let mut attempts = 0;
        while !status.success()
            && (self.get_retry() == &-1 || &attempts < self.get_retry())
        {
            attempts += 1;
            warn!(
                "Retrying {}: {}, attempt: {}", 
                self.get_execution_type(), 
                &self, 
                attempts
            );
            // Spawn the command again as a child process
            let status = command
                .spawn()
                .expect("Failed to execute command")
                .wait()
                .expect("Failed to wait on child");
            // If the command fails again and retry limit is reached, print an error message and stop the chain
            if !status.success()
                && self.get_retry() != &-1
                && &attempts >= self.get_retry()
            {
                error!(
                    "Failed to execute {}: {}", 
                    self.get_execution_type(), 
                    &self
                );
                return Ok(());
            }
        }

        // If the command fails and retry is -1, keep retrying indefinitely
        if !status.success() && self.get_retry() == &-1 {
            loop {
                attempts += 1;
                warn!(
                    "Retrying {}: {}, attempt: {}", 
                    self.get_execution_type(),
                    &self, 
                    attempts
                );
                let status = command
                    .spawn()
                    .expect("Failed to execute command")
                    .wait()
                    .expect("Failed to wait on child");
                if status.success() {
                    break;
                }
            }
        }

        // If the command fails and retry is 0, stop the chain
        if !status.success() && self.get_retry() == &0 {
            error!(
                "Failed to execute {}: {}\n", 
                self.get_execution_type(), 
                &self
            );
            return Ok(());
        }

        // Separation between commands
        info!("===============================");
        info!("Finished executing command: {}", &self);
        info!("===============================");

        Ok(())
    }
}