use std::io::Write;

use anyhow::{anyhow, Error, Result};
use log::info;

use crate::{
    program::Program,
    utility::{execute_argument_function, Execution, ExecutionType},
    variable::{Variable, VariableGroupControl, VariableInitializationTime},
};

pub struct Chain {
    programs: Vec<Program>,
    variables: Vec<Variable>,
}

impl Chain {
    pub fn from_file(path: &str) -> Result<Self, Error> {
        let programs: Vec<Program> = serde_json::from_str(
            &std::fs::read_to_string(path).expect("Failed to load configurations"),
        )?;

        // check if there are variables being specified in the programs,
        // if so, register them in the chain.
        let mut variables: Vec<Variable> = Vec::new();
        for program in &programs {
            if let Some(awaitable_variable) = program
                .get_awaitable_variable() {
                    variables.push(
                        Variable::parse_await_variable(awaitable_variable)
                    );
            }

            for argument in program.get_arguments() {
                let variables_in_arguments: Vec<Variable> =
                    Variable::parse_variables_from_str(argument)?;
                
                if variables.len() != 0 {
                    for item in &variables_in_arguments {
                        if !variables
                                .iter()
                                .any(
                                    |v| 
                                    v.get_variable_name() == item.get_variable_name()
                                ) {
                                    variables.push(item.to_owned());
                        }
                    }
                } else {
                    variables.extend(variables_in_arguments);
                }
            }
        }
        
        Ok(Self {
            programs,
            variables,
        })
    }
}

impl std::fmt::Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut program_names: String = String::new();
        for program in &self.programs {
            program_names.push_str(&(program.to_string() + "\n"))
        }
        f.write_str(&format!("{}", program_names))
    }
}

impl VariableGroupControl for Chain {
    fn get_value(&self, variable_name: &str) -> Result<String, Error> {
        for variable in &self.variables {
            if variable.get_variable_name() == variable_name {
                return Ok(variable.get_value()?);
            }
        }

        return Err(anyhow!("Variable {} does not exist!", variable_name));
    }

    fn update_value(&mut self, variable_name: &str, value: String) {
        for variable in &mut self.variables {
            if variable.get_raw_variable_name() == variable_name {
                variable.register_value(value);
                break;
            }
        }
    }
}

impl Execution for Chain {
    fn get_execution_type(&self) -> &crate::utility::ExecutionType {
        &ExecutionType::Chain
    }

    async fn execute(&mut self) -> Result<String, Error> {
        // See if any program needs input on startup
        for variable in &mut self.variables {
            if variable.get_initialization_time() == VariableInitializationTime::OnChainStartup {
                info!(
                    "Please input a value for {}:",
                    variable.get_human_readable_name()
                );
                let mut input = String::new();
                std::io::stdout().flush().expect("Failed to flush stdout");
                std::io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read input");
                variable.register_value(input.trim().to_string());
            }
        }

        // Iterate over each program configuration in the chain and execute them sequentially.
        // For each program, we first process any argument functions, then insert the chain's variables
        // into the program, and finally execute the program. If the program provides an awaitable variable,
        // we capture its output and update the corresponding variable in the chain.
        for i in 0..self.programs.len() {
            // Record awaitable variable if any
            let mut awaitable_variable: Option<String> = None;
            let mut awaitable_value: Option<String> = None;
            
            // Get a mutable reference to the current program.
            let program = &mut self.programs[i];
            // Process any functions provided as arguments for the program.
            execute_argument_function(program).await?;
            // Insert available variables from the chain into the program's context.
            program.insert_variable(&self.variables)?;
            
            // Check if the program returns an awaitable variable.
            if let Some(variable) = program.get_awaitable_variable().clone() {
                // Execute the program and capture its output.
                let output: String = program.execute().await?;
                // Return the awaitable variable along with the captured output.
                awaitable_variable = Some(variable);
                awaitable_value = Some(output);
            } else {
                // If there is no awaitable variable, simply execute the program.
                program.execute().await?;
            }

            // If the program returned an awaitable variable and output, update the chain's variable.
            if awaitable_value.is_some() && awaitable_variable.is_some() {
                self.update_value(
                    &awaitable_variable.unwrap(), 
                    awaitable_value.unwrap()
                );
            }
        }

        Ok("Done".to_string())
    }
}
