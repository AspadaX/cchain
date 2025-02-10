use std::collections::HashMap;
use std::fs::canonicalize;
use std::fs::DirEntry;
use std::str::FromStr;

use anyhow::{Error, Result};

use crate::display_control::display_message;
use crate::display_control::Level;
use crate::function;
use crate::program::FailureHandlingOptions;
use crate::program::Interpreter;
use crate::program::Program;
use crate::program::StdoutStorageOptions;

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
        display_message(
            Level::Error, 
            "No configuration files provided in the paths argument"
        );
        std::process::exit(1);
    }

    // List available configuration files for the user to select
    display_message(
        Level::Logging, 
        "Available configuration files:"
    );
    for (i, path) in paths.iter().enumerate() {
        display_message(
            Level::Selection, 
        &format!("{}: {}", i, path)
        );
    }

    // Prompt the user to select a configuration file
    display_message(
        Level::Logging, 
        "Please select a configuration file by entering the corresponding number:"
    );
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
        Program::new(
            "example_command".to_string(),
            vec!["arg1".to_string(), "arg2".to_string()],
            Some(HashMap::new()),
            Some("<<hi>>".to_string()),
            StdoutStorageOptions::default(),
            Some(Interpreter::Sh),
            FailureHandlingOptions::default(),
            3,
        ),
        Program::new(
            "another_command".to_string(),
            vec!["argA".to_string(), "argB".to_string()],
            None,
            None,
            StdoutStorageOptions::default(),
            None,
            FailureHandlingOptions::default(),
            5,
        ),
    ];
    // Serialize the template to JSON
    let template_json =
        serde_json::to_string_pretty(&template).expect("Failed to serialize template");
    // Write the template JSON to a file
    std::fs::write("cchain_template.json", template_json).expect("Failed to write template file");
    display_message(
        Level::Logging, 
        "Template configuration file generated: cchain_template.json"
    );
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
pub async fn execute_argument_function(configuration: &mut Program) -> Result<(), Error> {
    // Iterate over each argument in the configuration
    for index in 0..configuration.get_arguments().len() {
        // Clone the current argument
        let argument: String = configuration.get_arguments()[index].clone();

        // Attempt to parse the argument as a function
        let function: function::Function = match function::Function::from_str(&argument) {
            Ok(f) => f,
            Err(_) => continue, // If parsing fails, skip to the next argument
        };

        display_message(
            Level::Logging, 
            &format!(
                "Detected function, {}, when executing command: {}, executing the function...",
                function.get_name(),
                configuration
            )
        );

        // Execute the function asynchronously and await the result
        let result: String = function.execute().await?;
        configuration.revise_argument(index, result);
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
        let path_str = canonicalize(entry.path())?.to_string_lossy().to_string();
        json_paths.push(path_str);
    }

    Ok(json_paths)
}

pub enum ExecutionType {
    Chain,
    Program,
    Function,
}

impl std::fmt::Display for ExecutionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionType::Chain => f.write_str("Chain"),
            ExecutionType::Program => f.write_str("Program"),
            ExecutionType::Function => f.write_str("Function"),
        }
    }
}

/// Anything that can be executed
pub trait Execution
where
    Self: std::fmt::Display,
{
    fn get_execution_type(&self) -> &ExecutionType;

    async fn execute(&mut self) -> Result<String, Error>;
}