use std::collections::HashMap;

use anyhow::{Error, Result};

use crate::commons::utility::generate_text_with_llm;
use crate::core::interpreter::Interpreter;
use crate::core::options::FailureHandlingOptions;
use crate::core::options::StdoutStorageOptions;
use crate::core::program::Program;
use crate::display_control::display_message;
use crate::display_control::Level;

pub struct ChainCreation {
    name: Option<String>,
    template: Vec<Program>
}

impl ChainCreation {

    pub fn new(name: Option<String>) -> Self {
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
                None,
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
                None,
                5,
            ),
        ];

        Self { name, template }
    }

    pub fn create_filename(&self) -> String {
        if let Some(name) = &self.name {
            return "cchain_".to_string() + name + ".json";
        } else {
            return "cchain_template.json".to_string();
        }
    }

    /// Generates a template configuration file.
    ///
    /// This function creates a template configuration with example commands and arguments,
    /// serializes it to JSON, and writes it to a file named `cchain_template.json`.
    pub fn generate_template(&self) -> Result<(), Error> {
        let filename: String = self.create_filename();
        // Serialize the template to JSON
        let template_json: String = serde_json::to_string_pretty(&self.template)?;
        // Write the template JSON to a file
        std::fs::write(&filename, template_json)?;
        display_message(
            Level::Logging,
            &format!("Template chain generated: {}", &filename),
        );

        Ok(())
    }

    pub fn generate_chain(&self) -> Result<(), Error> {
        let prompt: String = format!();
        let result: String = generate_text_with_llm(prompt)?;
        return Ok(result);
    }
}