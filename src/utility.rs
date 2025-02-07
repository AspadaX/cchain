use std::collections::HashMap;
use std::fmt::Write;
use std::fs::canonicalize;
use std::fs::DirEntry;
use std::str::FromStr;

use anyhow::{Error, Result};
use log::{error, info, warn};

use crate::configuration::Command;
use crate::function;

fn get_paths(path: &std::path::Path) -> Vec<DirEntry> {
    let mut paths: Vec<DirEntry> = Vec::new();
    let entries = std::fs::read_dir(path).expect("Failed to read directory");
    for entry in entries {
        let entry = entry.expect("Failed to read entry");
        if entry.path().is_file()
            && entry.path().extension().map_or(false, |ext| ext == "json")
            && entry.file_name().to_string_lossy().starts_with("cchain_")
        {
            paths.push(entry);
        }
    }
    paths
}

/// Resolves the configuration file to use based on the provided paths.
///
/// This function lists available configuration files and prompts the user to select one.
///
/// # Arguments
///
/// * `paths` - A vector of strings representing the paths to configuration files.
///
/// # Returns
///
/// A `String` representing the path to the selected configuration file.
pub fn configuration_selection(paths: Vec<String>) -> String {
    if paths.is_empty() {
        error!("No configuration files provided in the paths argument");
        std::process::exit(1);
    }

    // List available configuration files for the user to select
    info!("Available configuration files:");
    for (i, path) in paths.iter().enumerate() {
        info!("     {}: {}", i, path);
    }

    // Prompt the user to select a configuration file
    info!("Please select a configuration file by entering the corresponding number:");
    let mut selection = String::new();
    std::io::stdin()
        .read_line(&mut selection)
        .expect("Failed to read input");
    let index: usize = selection.trim().parse().expect("Invalid selection");

    // Return the selected configuration file path
    paths[index].to_string()
}

/// Generates a template configuration file.
///
/// This function creates a template configuration with example commands and arguments,
/// serializes it to JSON, and writes it to a file named `cchain_template.json`.
pub fn generate_template() {
    // Create a template configuration
    let template = vec![
        Command::new(
            "example_command".to_string(),
            vec!["arg1".to_string(), "arg2".to_string()],
            HashMap::new(),
            3,
        ),
        Command::new(
            "another_command".to_string(),
            vec!["argA".to_string(), "argB".to_string()],
            HashMap::new(),
            5,
        ),
    ];
    // Serialize the template to JSON
    let template_json =
        serde_json::to_string_pretty(&template).expect("Failed to serialize template");
    // Write the template JSON to a file
    std::fs::write("cchain_template.json", template_json).expect("Failed to write template file");
    info!("Template configuration file generated: cchain_template.json");
}

/// Executes functions specified in the configuration arguments.
///
/// This function iterates over each argument in the configuration, attempts to parse it as a function,
/// and if successful, executes the function asynchronously.
///
/// # Arguments
///
/// * `configuration` - A mutable reference to the `Configuration` struct containing the arguments.
///
/// # Returns
///
/// A `Result` indicating the success or failure of the function execution.
pub async fn execute_argument_function(configuration: &mut Command) -> Result<(), Error> {
    // Iterate over each argument in the configuration
    for index in 0..configuration.get_arguments().len() {
        // Clone the current argument
        let argument: String = configuration.get_arguments()[index].clone();

        // Attempt to parse the argument as a function
        let function: function::Function = match function::Function::from_str(&argument) {
            Ok(f) => f,
            Err(_) => continue, // If parsing fails, skip to the next argument
        };

        info!(
            "Detected function, {}, when executing command: {}, executing the function...",
            function.get_name(),
            configuration
        );

        // Execute the function asynchronously and await the result
        let result: String = function.execute().await?;
        configuration.revise_argument(index, result);
        info!("Function, {}, executed successfully", function.get_name());
    }
    // Return the result of the function execution
    Ok(())
}

/// Collects paths of all JSON files starting with 'cchain_' from the specified directory.
///
/// This function reads the specified directory and collects all files that have a '.json' extension
/// and start with 'cchain_'. It then returns a vector of the paths to these files as strings.
///
/// # Arguments
///
/// * `path` - A reference to the path of the directory to read the JSON files from.
///
/// # Returns
///
/// A `Result` containing a vector of strings, each representing the path to a JSON file, or an error if any occurs.
pub fn resolve_cchain_configuration_filepaths(
    path: &std::path::Path,
) -> Result<Vec<String>, Error> {
    let mut json_paths: Vec<String> = Vec::new();
    let paths = get_paths(path);

    for entry in paths {
        let path_str = canonicalize(entry.path())?
            .to_string_lossy()
            .to_string();
        json_paths.push(path_str);
    }

    Ok(json_paths)
}

pub enum ExecutionType {
    Command,
    Function
}

impl std::fmt::Display for ExecutionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionType::Command => f.write_str("Command"),
            ExecutionType::Function => f.write_str("Function")
        }
    }
}

/// Executes a command based on the provided configuration.
///
/// # Arguments
///
/// * `configuration` - A reference to the `Configuration` struct containing the command and its arguments.
pub trait Execution 
where
    Self: std::fmt::Display
{
    fn get_command(&self) -> &str;

    fn get_arguments(&self) -> &Vec<String>;

    fn get_retry(&self) -> &i32;

    fn get_execution_type(&self) -> &ExecutionType;

    fn execute(&self) -> Result<(), Error> {
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