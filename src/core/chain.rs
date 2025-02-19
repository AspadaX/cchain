use std::{cell::Cell, sync::{Arc, Mutex, MutexGuard}, thread};

use anyhow::{anyhow, Error, Result};

use crate::{
    core::{
        program::Program,
        traits::{Execution, ExecutionType},
    },
    commons::utility::input_message,
    display_control::{display_message, Level},
    variable::{Variable, VariableGroupControl, VariableInitializationTime},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ChainExecutionResult {
    output: String,
}

impl ChainExecutionResult {
    pub fn new(output: String) -> Self {
        Self { output }
    }
}

#[derive(Debug)]
pub struct Chain {
    programs: Vec<Arc<Mutex<Program>>>,
    variables: Vec<Arc<Mutex<Variable>>>,
    failed_program_executions: Cell<usize>,
}

impl Chain {
    pub fn from_file(path: &str) -> Result<Self, Error> {
        let programs: Vec<Program> = serde_json::from_str(&std::fs::read_to_string(path)?)?;

        let mut programs: Vec<Arc<Mutex<Program>>> = programs
            .into_iter()
            .map(|item| Arc::new(Mutex::new(item)))
            .collect();

        // check if there are variables being specified in the programs,
        // if so, register them in the chain.
        let mut variables: Vec<Arc<Mutex<Variable>>> = Vec::new();
        for (index, program) in programs.iter_mut().enumerate() {
            if let Some(awaitable_variable) = program.lock().unwrap().get_awaitable_variable() {
                variables.push(Arc::new(Mutex::new(Variable::parse_await_variable(
                    awaitable_variable,
                    index,
                ))));
            }

            for argument in program.lock().unwrap().get_command_line().get_arguments() {
                let variables_in_arguments: Vec<Arc<Mutex<Variable>>> =
                    Variable::parse_variables_from_str(argument, index)?
                        .into_iter()
                        .map(|variable| Arc::new(Mutex::new(variable)))
                        .collect();

                if variables.len() != 0 {
                    for item in variables_in_arguments {
                        if !variables.iter().any(|v| {
                            v.lock().unwrap().get_variable_name()
                                == item.lock().unwrap().get_variable_name()
                        }) {
                            variables.push(item);
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
            failed_program_executions: Cell::new(0),
        })
    }

    pub fn validate_syntax(&mut self) -> Result<(), Error> {
        // Collect problematic variables
        let mut variables_used_without_being_initialized: Vec<Variable> = Vec::new();

        for (index, program) in self.programs.iter_mut().enumerate() {
            let mut variables_involved: Vec<Variable> = Vec::new();
            let mut program = program.lock().unwrap();
            // Get all variables involed in this program
            // Get the variables in arguments first
            for argument in program.get_command_line().get_arguments() {
                variables_involved.extend(Variable::parse_variables_from_str(argument, index)?);
            }
            // Get the variables in remedy command if any
            if let Some(remedy_command_line) = program.get_remedy_command_line() {
                for argument in remedy_command_line.get_arguments() {
                    variables_involved.extend(Variable::parse_variables_from_str(argument, index)?);
                }
            }
            // Check the lifetime validity of the variables
            for variable_involved in variables_involved {
                let is_initialized = variable_involved
                    .get_initialization_time()
                    .is_initialized(index);

                if !is_initialized {
                    variables_used_without_being_initialized.push(variable_involved);
                }
            }
        }

        if variables_used_without_being_initialized.len() != 0 {
            display_message(
                Level::Error,
                &format!(
                    "{} variables are used without being initialized. 😭",
                    variables_used_without_being_initialized.len()
                ),
            );

            for variable in variables_used_without_being_initialized {
                display_message(
                    Level::Error,
                    &format!("Problematic variable: {}", variable.get_raw_variable_name()),
                );
            }

            return Err(anyhow!("Check is not passed. 😢"));
        }

        display_message(Level::Logging, &format!("Check is passed! 😄"));

        Ok(())
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
            let variable = variable.lock().unwrap();
            // skip the `None` value variables
            if variable.get_value().is_ok() {
                let mut program = self.programs[program_index].lock().unwrap();
                program
                    .get_command_line()
                    .inject_value_to_variables(
                        &variable.get_raw_variable_name(),
                        variable.get_value()?,
                    )?;

                if let Some(command_line) = program
                    .get_remedy_command_line()
                {
                    command_line.inject_value_to_variables(
                        &variable.get_raw_variable_name(),
                        variable.get_value()?,
                    )?;
                }
            }
        }

        Ok(())
    }

    pub fn initialize_variables_on_chain_startup(&mut self) -> Result<(), Error> {
        for variable in &mut self.variables {
            let mut variable = variable.lock().unwrap();
            if let VariableInitializationTime::OnChainStartup(_) =
                variable.get_initialization_time()
            {
                let input: String = input_message(&format!(
                    "Please input a value for {}:",
                    variable.get_human_readable_name()
                ))?;
                variable
                    .register_value(input.trim().to_string());
            }
        }

        Ok(())
    }

    /// Initializes variables for the program execution phase.
    ///
    /// This method iterates over each argument of the specified program and extracts variables from these arguments.
    /// For each variable that requires initialization at program execution (i.e., its initialization time is
    /// `VariableInitializationTime::OnProgramExecution`), the method prompts the user to input a value. The provided
    /// value is then registered with the corresponding variable in the chain.
    ///
    /// # Arguments
    ///
    /// * `program_index` - The index of the program in the chain whose arguments will be inspected for variables
    ///                     needing initialization during program execution.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if initialization is successful for all applicable variables, or an `Error` if any step fails.
    pub fn initialize_variables_on_program_execution(
        &mut self,
        program_index: usize,
    ) -> Result<(), Error> {
        // Acquire the lock first
        let mut program = self.programs[program_index].lock().unwrap();

        for argument in program 
            .get_command_line()
            .get_arguments()
        {
            let mut program_variables: Vec<Variable> =
                Variable::parse_variables_from_str(argument, program_index)?;

            for program_variable in &mut program_variables {
                for variable in &mut self.variables {
                    let mut variable = variable.lock().unwrap();

                    if program_variable.get_raw_variable_name()
                        == variable.get_raw_variable_name()
                        && matches!(
                            program_variable.get_initialization_time(),
                            VariableInitializationTime::OnProgramExecution(_)
                        )
                    {
                        let input: String = input_message(&format!(
                            "Please input a value for {}:",
                            variable.get_human_readable_name()
                        ))?;
                        variable.register_value(input.trim().to_string());
                    }
                }
            }
        }
        Ok(())
    }

    pub fn handle_program_execution_failures(
        &self,
        program: &mut MutexGuard<'_, Program>,
        error_message: &str,
    ) -> Result<(), Error> {
        // Increment the failure count
        self.increment_failed_execution();
        // Display error message
        display_message(Level::Error, &error_message);

        if let Some(command) = program.get_remedy_command_line() {
            display_message(
                Level::Logging,
                &format!("Remedy command is set. Try executing: {}", command),
            );
            // execute the remedy command line if any
            program.execute_remedy_command_line()?;
        }

        if !program.get_failure_handling_options().exit_on_failure {
            display_message(
                Level::Warn,
                &format!("`exit_on_failure` is set to false. Continue executing the chain..."),
            );
            return Ok(());
        } else {
            return Err(anyhow!(error_message.to_string()));
        }
    }

    pub fn increment_failed_execution(&self) {
        let number: usize = self.failed_program_executions.get();
        self.failed_program_executions.set(number + 1);
    }

    pub fn show_statistics(&self) {
        display_message(
            Level::Error,
            &format!(
                "{} failures occurred when executing programs.",
                self.failed_program_executions.get()
            ),
        );
        display_message(
            Level::Logging,
            &format!(
                "{} successes occurred when executing programs.",
                (self.programs.len() - self.failed_program_executions.get())
            ),
        );
    }
}

impl std::fmt::Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut program_names: String = String::new();
        for program in &self.programs {
            let program = program.lock().unwrap();
            program_names.push_str(&(program.to_string() + "\n"))
        }
        f.write_str(&format!("{}", program_names))
    }
}

impl VariableGroupControl for Chain {
    fn get_value(&self, variable_name: &str) -> Result<String, Error> {
        for variable in &self.variables {
            let variable = variable.lock().unwrap();
            if variable.get_variable_name() == variable_name {
                return Ok(variable.get_value()?);
            }
        }

        return Err(anyhow!("Variable {} does not exist!", variable_name));
    }

    fn update_value(&mut self, variable_name: &str, value: String) {
        for variable in &mut self.variables {
            let mut variable = variable.lock().unwrap();
            if variable.get_raw_variable_name() == variable_name {
                variable.register_value(value);
                break;
            }
        }
    }
}

impl Execution<ChainExecutionResult> for Chain {
    fn get_execution_type(&self) -> &ExecutionType {
        &ExecutionType::Chain
    }

    fn execute(&mut self) -> Result<Vec<ChainExecutionResult>, Error> {
        // See if any program needs input on startup
        self.initialize_variables_on_chain_startup()?;
        
        // Capture the concurrency groups
        let mut current_concurrency_group_number: usize = 0;
        let mut concurrency_group: Vec<Arc<Mutex<Program>>> = Vec::new();

        // Iterate over each program configuration in the chain and execute them sequentially.
        // For each program, we first process any argument functions, then insert the chain's variables
        // into the program, and finally execute the program. If the program provides an awaitable variable,
        // we capture its output and update the corresponding variable in the chain.
        for i in 0..self.programs.len() {
            // Check if the current program needs input to a value's intialization
            // time that is `on_program_execution`. If so, prompt the user for
            // inputting a value
            self.initialize_variables_on_program_execution(i)?;

            // Record awaitable variable if any
            let mut awaitable_variable: Option<String> = None;
            let mut awaitable_value: Option<String> = None;

            // Create a single block for clearing the mut ref to the
            // self.programs.
            {
                // Get a mutable reference to the current program.
                let program = &mut self.programs[i].lock().unwrap();
                // Process any functions provided as arguments for the program.
                program.execute_argument_functions()?;
            }

            // Insert available variables from the chain into the program's context.
            self.insert_variable(i)?;

            // Get the number of programs that are currently added to the 
            // concurrent group for executions
            let number_of_concurrent_programs_to_be_executed: usize = concurrency_group.len();

            // Determine whether to add this to the concurrency group, 
            // or execute the concurrency group
            // or continue with the sequential order
            // We use a new block to handle the borrowing issue when using this_program as 
            // mutable in the later context
            {
                // Acquire the lock of the program first
                let mut this_program: MutexGuard<'_, Program> = self.programs[i].lock().unwrap();

                if let Some(concurrency_group_number_for_this_program) = this_program
                    .get_concurrency_group() 
                {
                    // Determine whetehr the concurrency group can be executed
                    if number_of_concurrent_programs_to_be_executed > 0 {
                        if current_concurrency_group_number != concurrency_group_number_for_this_program
                        {
                            let mut tasks = Vec::new();
                            for program in &concurrency_group {
                                let program_clone = program.clone();
                                tasks.push(
                                    thread::spawn(
                                        move || {
                                            let mut program_clone = program_clone.lock().unwrap();
                                            let result = program_clone.execute();
                                            result
                                        }
                                    )
                                );
                            }

                            let mut results = Vec::new();
                            for task in tasks {
                                results.push(task.join().unwrap());
                            }

                            for result in results {
                                match result {
                                    // The output of concurrency resutls are not going to be recorded
                                    // for now.
                                    Ok(_) => continue,
                                    Err(error) => match self.handle_program_execution_failures(&mut this_program, &error.to_string()) {
                                        Ok(_) => continue,
                                        Err(error) => return Err(error)
                                    }
                                }
                            }
                        
                            concurrency_group.clear();
                        }
                    }
                    
                    // Set the concurrent concurrency group number
                    current_concurrency_group_number = concurrency_group_number_for_this_program;
                    // Push the program to the concurrency group, 
                    // if the concurrency group is not eligible for execution
                    concurrency_group.push(self.programs[i].clone());
                    display_message(
                        Level::Logging, 
                        &format!(
                            "Concurrent program, {}, is collected...", 
                            this_program.get_command_line()
                        )
                    );
                    continue;
                }

                // Check if the program returns an awaitable variable.
                let awaitable_variable_this_program: Option<String> = this_program.get_awaitable_variable().clone();
                if let Some(variable) = awaitable_variable_this_program
                {
                    // Execute the program and capture its output.
                    let output: String = match this_program.execute() {
                        Ok(result) => result[0].clone().get_output(),
                        Err(error) => match self.handle_program_execution_failures(&mut this_program, &error.to_string()) {
                            Ok(_) => continue,
                            Err(error) => return Err(error)
                        }
                    };
                    // Return the awaitable variable along with the captured output.
                    awaitable_variable = Some(variable.to_string());
                    awaitable_value = Some(output);
                } else {
                    // If there is no awaitable variable, simply execute the program.
                    match this_program.execute() {
                        Ok(result) => result,
                        Err(error) => match self.handle_program_execution_failures(&mut this_program, &error.to_string()) {
                            Ok(_) => continue,
                            Err(error) => return Err(error)
                        }
                    };
                }
            }

            // If the program returned an awaitable variable and output, update the chain's variable.
            if awaitable_value.is_some() && awaitable_variable.is_some() {
                self.update_value(&awaitable_variable.unwrap(), awaitable_value.unwrap());
            }
        }

        // Execute any remaining programs in the concurrency group after the loop
        if !concurrency_group.is_empty() {
            let mut tasks = Vec::new();
            for program in &concurrency_group {
                let program_clone = program.clone();
                tasks.push(thread::spawn(move || {
                    let mut program = program_clone.lock().unwrap();
                    program.execute()
                }));
            }

            let mut results = Vec::new();
            for task in tasks {
                match task.join().unwrap() {
                    Ok(result) => results.extend(result),
                    Err(e) => return Err(e),
                }
            }
        }

        Ok(vec![ChainExecutionResult::new("Done".to_string())])
    }
}