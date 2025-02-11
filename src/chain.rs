use anyhow::{anyhow, Error, Result};

use crate::{
    cli::{program::Program, traits::{Execution, ExecutionType}}, commons::utility::input_message, display_control::display_message, variable::{
        Variable, 
        VariableGroupControl, 
        VariableInitializationTime
    }
};

pub struct Chain {
    programs: Vec<Program>,
    variables: Vec<Variable>,
}

impl Chain {
    pub fn from_file(path: &str) -> Result<Self, Error> {
        let mut programs: Vec<Program> = serde_json::from_str(
            &std::fs::read_to_string(path).expect("Failed to load configurations"),
        )?;

        // check if there are variables being specified in the programs,
        // if so, register them in the chain.
        let mut variables: Vec<Variable> = Vec::new();
        for program in &mut programs {
            if let Some(awaitable_variable) = program
                .get_awaitable_variable() {
                    variables.push(
                        Variable::parse_await_variable(awaitable_variable)
                    );
            }

            for argument in program
                .get_command_line()
                .get_arguments() 
            {
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
    
    /// Inserts provided variables into the program's arguments.
    ///
    /// This method iterates over each argument in the program and replaces occurrences of
    /// raw variable names with their corresponding values. If retrieving the value of a variable
    /// fails, it returns an error.
    ///
    /// # Arguments
    ///
    /// * `variables` - A vector of `Variable` instances whose raw names will be replaced with their values.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if all variables are inserted successfully, or an `Error` if any variable's
    /// value retrieval fails.
    pub fn insert_variable(&mut self, program_index: usize) -> Result<(), Error> {
        for variable in &mut self.variables {
            // skip the `None` value variables
            if variable.get_value().is_ok() {
                self.programs[program_index]
                    .get_command_line()
                    .inject_value_to_variables(
                        &variable.get_raw_variable_name(), 
                        variable.get_value()?
                    )?;
                
                if let Some(command_line) = self
                    .programs[program_index]
                    .get_remedy_command_line() 
                {
                    command_line.inject_value_to_variables(
                        &variable.get_raw_variable_name(), 
                        variable.get_value()?
                    )?;
                }
            }
        }

        Ok(())
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
    fn get_execution_type(&self) -> &ExecutionType {
        &ExecutionType::Chain
    }

    async fn execute(&mut self) -> Result<String, Error> {
        // See if any program needs input on startup
        for variable in &mut self.variables {
            if variable.get_initialization_time() == VariableInitializationTime::OnChainStartup {
                let input: String = input_message(
                    &format!(
                        "Please input a value for {}:",
                        variable.get_human_readable_name()
                    )
                )?;
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
            
            // Create a single block for clearing the mut ref to the 
            // self.programs.
            {
                // Get a mutable reference to the current program.
                let program = &mut self.programs[i];
                // Process any functions provided as arguments for the program.
                program.execute_argument_functions().await?;
            }
            
            // Insert available variables from the chain into the program's context.
            self.insert_variable(i)?;
            // Get a mutable reference to the current program again.
            let program = &mut self.programs[i];
            
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
