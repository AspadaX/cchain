use std::{collections::HashMap, path::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::utility::{Execution, ExecutionType};

#[derive(Deserialize, Serialize)]
pub struct Command {
    command: String,
    arguments: Vec<String>,
    environment_variables_override: HashMap<String, String>,
    retry: i32
}

impl Command {
    pub fn new(
        command: String, 
        arguments: Vec<String>, 
        environment_variables_override: HashMap<String, String>, 
        retry: i32
    ) -> Self {
        Command {
            command,
            arguments,
            environment_variables_override,
            retry,
        }
    }

    pub fn revise_argument(&mut self, argument_index: usize, new_argument: String) {
        self.arguments[argument_index] = new_argument;
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.command, self.arguments.join(" "))
    }
}

impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() < 2 {
            return Err("Invalid configuration".to_string());
        }

        let command = parts[0].to_string();
        let arguments = parts[1..].iter().map(|s| s.to_string()).collect();

        Ok(
            Command::new(
                command, 
                arguments, 
                HashMap::new(),
                0
            )
        )
    }
}

impl Execution for Command {
    fn get_command(&self) -> &str {
        &self.command
    }

    fn get_arguments(&self) -> &Vec<String> {
        &self.arguments
    }

    fn get_retry(&self) -> &i32 {
        &self.retry
    }

    fn get_execution_type(&self) -> &ExecutionType {
        &ExecutionType::Command
    }
}