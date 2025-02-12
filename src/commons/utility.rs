use std::collections::HashMap;
use std::fs::DirEntry;
use std::io::Write;

use anyhow::{Error, Result};

use crate::cli::interpreter::Interpreter;
use crate::cli::options::FailureHandlingOptions;
use crate::cli::options::StdoutStorageOptions;
use crate::cli::program::Program;
use crate::display_control::display_message;
use crate::display_control::Level;

pub fn get_paths(path: &std::path::Path) -> Result<Vec<DirEntry>, Error> {
    let mut paths: Vec<DirEntry> = Vec::new();
    let entries = std::fs::read_dir(path)?;
    for entry in entries {
        let entry = entry?;
        if entry.path().is_file()
            && entry.path().extension().map_or(false, |ext| ext == "json")
            && entry.file_name().to_string_lossy().starts_with("cchain_")
        {
            paths.push(entry);
        }
    }
    Ok(paths)
}

/// Generates a template configuration file.
///
/// This function creates a template configuration with example commands and arguments,
/// serializes it to JSON, and writes it to a file named `cchain_template.json`.
pub fn generate_template(name: Option<&str>) {
    let filename = if let Some(name) = name {
        name
    } else {
        "cchain_template.json"
    };

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
    std::fs::write(filename, template_json).expect("Failed to write template file");
    display_message(
        Level::Logging, 
        &format!("Template configuration file generated: {}", filename)
    );
}

pub fn input_message(prompt: &str) -> Result<String, Error> {
    // display the prompt message for inputting values
    display_message(Level::Logging, prompt);
    // collect the input as a string
    let mut input = String::new();
    // receive stdin
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut input)?;
    
    Ok(input)
}