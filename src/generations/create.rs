use std::collections::HashMap;

use anyhow::{Error, Result};
use serde::Deserialize;
use serde::Serialize;

use crate::core::interpreter::Interpreter;
use crate::core::options::FailureHandlingOptions;
use crate::core::options::StdoutStorageOptions;
use crate::core::program::Program;
use crate::display_control::display_message;
use crate::display_control::Level;

use super::llm::LLM;

#[derive(Debug, Serialize, Deserialize)]
pub struct ParsedCommands {
    pub commands: Vec<Program>,
}

pub struct ChainCreation {
    name: Option<String>
}

impl ChainCreation {

    pub fn new(name: Option<String>) -> Self {
        Self { name }
    }

    pub fn create_filename(&self) -> String {
        if let Some(name) = &self.name {
            return "cchain_".to_string() + name + ".json";
        } else {
            return "cchain_template.json".to_string();
        }
    }
    
    /// Get a template objects in Vec<Program>
    pub fn get_template_objects(&self) -> Vec<Program> {
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

        template
    }

    /// Generates a template configuration.
    pub fn generate_template(&self) -> Result<String, Error> {
        Ok(serde_json::to_string_pretty::<Vec<Program>>(&self.get_template_objects())?)
    }

    /// Create a chain by using the LLM
    pub fn generate_chain(&self, request: String) -> Result<String, Error> {
        let template = ParsedCommands { commands: self.get_template_objects() };
        let prompt: String = format!(
            "Your request is: {}\n Generate a json by strictly following this template: {}",
            &request,
            serde_json::to_string_pretty::<ParsedCommands>(&template)?
        );

        let llm = LLM::new()?;
        let result: String = llm.generate_json(prompt)?;
        
        // Parse the string 
        let parsed_commands: ParsedCommands = serde_json::from_str(&result)?;
        let commands_string: String = serde_json::to_string_pretty(&parsed_commands.commands)?;
        
        return Ok(commands_string);
    }

    /// Write the generated chain
    pub fn save(&self, json: String) -> Result<(), Error> {
        let filename: String = self.create_filename();
        // Write the template JSON to a file
        std::fs::write(&filename, json)?;
        display_message(
            Level::Logging,
            &format!("Template chain generated: {}", &filename),
        );

        Ok(())
    }
}