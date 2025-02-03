use std::{path::Display, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Configuration {
    command: String,
    arguments: Vec<String>,
    retry: i32,
}

impl Configuration {
    pub fn new(command: String, arguments: Vec<String>, retry: i32) -> Self {
        Configuration {
            command,
            arguments,
            retry,
        }
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

    pub fn revise_argument(&mut self, argument_index: usize, new_argument: String) {
        self.arguments[argument_index] = new_argument;
    }
}

impl std::fmt::Display for Configuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.command, self.arguments.join(" "))
    }
}

impl FromStr for Configuration {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() < 2 {
            return Err("Invalid configuration".to_string());
        }

        let command = parts[0].to_string();
        let arguments = parts[1..].iter().map(|s| s.to_string()).collect();

        Ok(Configuration::new(command, arguments, 0))
    }
}
