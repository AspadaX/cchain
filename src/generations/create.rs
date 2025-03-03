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
            r#"
            This is a chain example: {}
            To create a chain of program executions based on the JSON configuration:
            1. **Define Execution Steps**: 
               Create a list of program objects with fields:
               - `command`: Main executable (e.g., "python")
               - `arguments`: Parameters including {{variable}} placeholders
               - `interpreter`: Shell to use (e.g., "sh") or null for direct execution
               - `environment_variables_override`: Key-value pairs to override env vars
               - `stdout_stored_to`: Variable name to store output (supports <<>> syntax)
               - `failure_handling_options`: Configure exit behavior and remedy commands
               - `concurrency_group`: Null for sequential, same value for parallel steps
               - `retry`: Number of retry attempts (-1 = infinite, 0 = none)

            2. **Variable Handling**:
               - <<variable_name>>: Prompt user for value at chain startup
               - <<variable_name:on_program_execution>>: Prompt when command executes
               - Replace variables in arguments/env vars with user-provided values

            3. **Execution Flow**:
               a. Process steps sequentially by default
               b. Group steps with identical `concurrency_group` values to run in parallel
               c. For parallel groups:
                  - Wait for all commands in group to complete
                  - Apply failure handling after group completion

            4. **Output Handling**:
               - Capture stdout to variables specified in `stdout_stored_to`
               - Apply `without_newline_characters` formatting when storing
               - Make captured values available to subsequent steps via <<>> syntax

            5. **Failure Management**:
               - If `exit_on_failure`=true, abort chain on non-zero exit code
               - Execute `remedy_command_line` if provided on failure
               - Retry up to `retry` times before proceeding/failing

            6. **Environment Setup**:
               - Apply environment overrides before command execution
               - Empty string values override existing vars with empty values
               - Restore original environment after command completion

            Example Flow:
            1. Prompt for <<variable_name>> at startup
            2. Execute first command through sh interpreter:
               - Set hello="world", goodbye=""
               - On success: store processed output to <<hi>>
               - On failure: retry 3 times -> if still failing, run remedy command
            3. Proceed to next command only after success/retries
            4. Repeat for subsequent commands, handling variables/concurrency as configured
            Now, I need you to generate a `chain` based on this request: {}"#,
            serde_json::to_string_pretty(&template)?,
            &request
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