use std::{process::Command, str::FromStr};

use anyhow::anyhow;
use console::Term;
use regex;

use crate::{commons::utility::input_message, display_control::{display_command_line, display_message, Level}, generations::llm::LLM};

#[derive(Debug, Clone)]
pub struct Function {
    name: String,
    parameters: Vec<String>,
}

impl FromStr for Function {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = regex::Regex::new(r"(\w+)\s*$\s*'((?:[^']|\\')*)'\s*,\s*'((?:[^']|\\')*)'\s*$")?;

        if let Some(caps) = re.captures(s) {
            let func_name: String = caps
                .get(1)
                .ok_or_else(|| anyhow::anyhow!("Failed to capture function name"))?
                .as_str()
                .to_string();
            let arg1 = caps
                .get(2)
                .ok_or_else(|| anyhow::anyhow!("Failed to capture first argument"))?
                .as_str()
                .to_string();
            let arg2 = caps
                .get(3)
                .ok_or_else(|| anyhow::anyhow!("Failed to capture second argument"))?
                .as_str()
                .to_string();

            return Ok(Function {
                name: func_name,
                parameters: vec![arg1, arg2],
            });
        }

        Err(anyhow::anyhow!("No function found"))
    }
}

impl Function {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_parameters(&self) -> &Vec<String> {
        &self.parameters
    }

    pub fn execute(&self) -> Result<String, anyhow::Error> {
        match self.name.as_str() {
            "llm_generate" => self.llm_generate(),
            _ => Err(anyhow::anyhow!("Function not found")),
        }
    }

    fn llm_generate(&self) -> Result<String, anyhow::Error> {
        // execute the second parameter in the terminal and then get the output
        let command_output: String = if self.parameters.len() > 1 {
            let parts: Vec<&str> = self.parameters[1].split_whitespace().collect();
            let output = Command::new(parts[0])
                .args(&parts[1..])
                .output()
                .expect("Failed to execute command");

            if !output.status.success() {
                // Check if the command failed
                let error_message = if !output.stderr.is_empty() {
                    String::from_utf8_lossy(&output.stderr).to_string()
                } else {
                    format!("Command exited with status: {}", output.status)
                };
                return Err(anyhow::anyhow!("Command failed: {}", error_message));
            }

            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::new()
        };

        // Create an LLM instance for calling LLMs
        let llm = LLM::new()?;
        let prompt: String = format!("{}\n{}\n", self.parameters[0], command_output);

        loop {
            let response: String =
                match llm.generate(prompt.clone()) {
                    std::result::Result::Ok(response) => response,
                    Err(e) => {
                        anyhow::bail!("Failed to execute function: {}", e);
                    }
                };

            display_message(
                Level::Logging,
                &format!(
                    "Function executed successfully with result: "
                ),
            );
            display_command_line(&Term::stdout(), &response);

            let user_input: String = input_message(
                "Do you want to proceed with this result? (yes/retry/abort)"
            )?;
            let user_input: String = user_input.trim().to_lowercase();

            match user_input.as_str() {
                "yes" => {
                    // Proceed with the result
                    return Ok(response);
                }
                "retry" => {
                    // Retry the function execution
                    continue;
                }
                "abort" => {
                    return Err(anyhow!("Execution aborted by the user"));
                }
                _ => {
                    display_message(
                        Level::Warn,
                        "Invalid input, please enter 'yes', 'retry', or 'abort'.",
                    );
                }
            }
        }

    }
}
