use std::{cell::Cell, collections::HashSet, sync::Arc};

use futures::{future::join_all, FutureExt};
use anyhow::{anyhow, Error, Result};
use tokio::sync::Mutex;

use crate::{
    cli::{program::Program, traits::{Execution, ExecutionType}}, commons::utility::input_message, display_control::{display_message, Level}, variable::{
        Variable, 
        VariableGroupControl, 
        VariableInitializationTime
    }
};

#[derive(Debug, PartialEq, Eq)]
pub struct ConcurrencyGroupExecutionResult {
    program_index: usize,
    output: Option<String>,
    error: Option<Error>
}

impl ConcurrencyGroupExecutionResult {
    pub fn new(program_index: usize, result: Result<String, Error>) -> Self {
        match result {
            Ok(result) => return Self {
                program_index: program_index, 
                output: Some(result),
                error: None
            },
            Err(error) => return Self {
                program_index: program_index,
                output: None,
                error: Some(error)
            }
        }
    }
}

#[derive(Debug)]
pub struct ConcurrencyGroup {
    group_number: Option<usize>,
    programs: Vec<Arc<Mutex<Program>>>,
    failed_program_executions: Cell<usize>
}

impl ConcurrencyGroup {
    pub fn new() -> Self {
        Self {
            group_number: None, 
            programs: vec![], 
            failed_program_executions: Cell::new(0)
        }
    }

    pub fn get_group_number(&self) -> Result<usize, Error> {
        match self.group_number {
            Some(number) => Ok(number),
            None => Err(
                anyhow!(
                    "Group number is None. This concurrency group may not yet be populated."
                )
            )
        }
    }

    /// A method to add a new program into the concurrency group.
    /// It will automatically check if this is the first item being
    /// added. If so, it will align the concurency group number to 
    /// the program's. Otherwise, it will check if the program belongs
    /// to this group, return false if not.
    /// 
    /// Also, ANY PROGRAM that DOES NOT use concurrency, will be put
    /// into group number 0
    pub fn push(&mut self, program: Arc<Mutex<Program>>) -> bool {
        if self.group_number.is_none() {
            if program.blocking_lock().get_concurrency_group() {
                self.programs.push(program);

                return true;
            } else {
                self.group_number = 0;
                self.programs.push(program);

                return true;
            }
        }

        if self.group_number.is_some() {
            if program.blocking_lock()
                .get_concurrency_group() == self.group_number 
            {
                self.programs.push(program);

                return true;
            }
        }

        false
    }

    pub fn from_chain(chain: Chain) -> Result<Vec<ConcurrencyGroup>, Error> {
        // Determine how many concurrency groups are there
        let mut concurrency_group_numbers: HashSet<usize> = HashSet::new();
        for program in chain.programs {
            program.blocking_lock().get_concurrency_group();
        }
        
        // Initialize a vec to contain all concurrency groups
        let mut concurrency_groups: Vec<ConcurrencyGroup> = Vec::new();
        for program in chain.programs {
            // concurrency groups contain a group that has this program
            for concurrency_group in concurrency_groups {
                if concurrency_group.push(program) {
                    break;
                }
            } 

            // concurrency groups contain a group that DOES NOt have this program
            // &&
            // concurrency group is empty
            let mut concurrency_group = ConcurrencyGroup::new();
            concurrency_group.push(program);
        }

        // If all concurrency groups contain only one program, 
        // that means there is no concurrency, so return None
        if concurrency_groups
            .iter()
            .all(|item| item.len() == 1) 
        {
            return Err(
                anyhow!("No concurrency groups are found. Please execute the chain directly.")
            );
        }
        
        Ok(concurrency_groups)
    }
}

impl std::fmt::Display for ConcurrencyGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Concurrency group number: {}", self.group_number))
    }
}

impl Execution for ConcurrencyGroup {
    fn get_execution_type(&self) -> &ExecutionType {
        &ExecutionType::Chain
    }

    async fn execute<T: Eq + PartialEq>(&mut self) -> Result<Vec<T>, Error> {
        let mut tasks = Vec::new();
        if self.group_number != Some(0) {
            for program in self.programs {
                tasks.push(
                    tokio::spawn(program.lock().await.execute())
                );
            }
        }

        let results = join_all(tasks).await;

        let final_results = Vec::new();
        for result in results {
            let result = result?;
            final_results.push(result);
        }

        Ok(
            ConcurrencyGroupExecutionResult::new(program_index, output)
        )
    }
}

#[derive(Debug)]
pub struct Chain {
    programs: Vec<Arc<Mutex<Program>>>,
    variables: Vec<Arc<Mutex<Variable>>>,
    failed_program_executions: Cell<usize>
}

impl Chain {
    pub fn from_file(path: &str) -> Result<Self, Error> {
        let programs: Vec<Program> = serde_json::from_str(
            &std::fs::read_to_string(path)?,
        )?;

        let mut programs: Vec<Arc<Mutex<Program>>> = programs
            .into_iter()
            .map(|item| Arc::new(Mutex::new(item)))
            .collect();

        // check if there are variables being specified in the programs,
        // if so, register them in the chain.
        let mut variables: Vec<Arc<Mutex<Variable>>> = Vec::new();
        for (index, program) in programs
            .iter_mut()
            .enumerate()
        {
            if let Some(awaitable_variable) = program
                .blocking_lock()
                .get_awaitable_variable()
            {
                    variables.push(
                        Arc::new(Mutex::new(
                            Variable::parse_await_variable(
                                awaitable_variable, 
                                index
                            )
                        ))
                    );
            }

            for argument in program
                .blocking_lock()
                .get_command_line()
                .get_arguments() 
            {
                let variables_in_arguments: Vec<Arc<Mutex<Variable>>> =
                    Variable::parse_variables_from_str(argument, index)?
                    .into_iter()
                    .map(|variable| Arc::new(Mutex::new(variable)))
                    .collect();
                
                if variables.len() != 0 {
                    for item in variables_in_arguments {
                        if !variables
                            .iter()
                            .any(
                                |v|
                                v.blocking_lock().get_variable_name() == item.blocking_lock().get_variable_name()
                            ) {
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
            failed_program_executions: Cell::new(0)
        })
    }
    
    pub async fn validate_syntax(&mut self) -> Result<(), Error> {
        // Collect problematic variables
        let mut variables_used_without_being_initialized: Vec<Variable> = Vec::new();
        
        for (index, program) in self.programs
            .iter_mut()
            .enumerate() 
        {
            let mut variables_involved: Vec<Variable> = Vec::new();
            // Get all variables involed in this program
            // Get the variables in arguments first
            for argument in program
                .blocking_lock()
                .get_command_line()
                .get_arguments()
            {
                variables_involved.extend(
                    Variable::parse_variables_from_str(
                        argument, 
                        index
                    )?
                );
            }
            // Get the variables in remedy command if any
            if let Some(remedy_command_line) = program
                .blocking_lock()
                .get_remedy_command_line() 
            {
                for argument in remedy_command_line
                    .get_arguments()
                {
                    variables_involved.extend(
                        Variable::parse_variables_from_str(
                            argument, 
                            index
                        )?
                    );
                }
            }
            // Check the lifetime validity of the variables
            for variable_involved in variables_involved {
                let is_initialized = variable_involved
                    .get_initialization_time()
                    .is_initialized(index);
                
                if !is_initialized {
                    variables_used_without_being_initialized.push(
                        variable_involved
                    );
                }
            }
        }
        
        if variables_used_without_being_initialized.len() != 0 {
            display_message(
                Level::Error, 
                &format!(
                    "{} variables are used without being initialized. 😭",
                    variables_used_without_being_initialized.len()
                )
            );
            
            for variable in variables_used_without_being_initialized {
                display_message(
                    Level::Error, 
                    &format!(
                        "Problematic variable: {}",
                        variable.get_raw_variable_name()
                    )
                );
            }
            
            return Err(anyhow!("Check is not passed. 😢"))
        }
        
        display_message(
            Level::Logging, 
            &format!(
                "Check is passed! 😄"
            )
        );
        
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
            // skip the `None` value variables
            if variable.blocking_lock().get_value().is_ok() {
                self.programs[program_index]
                    .clone()
                    .lock_owned()
                    .now_or_never()
                    .unwrap()
                    .get_command_line()
                    .inject_value_to_variables(
                        &variable.blocking_lock().get_raw_variable_name(), 
                        variable.blocking_lock().get_value()?
                    )?;
                
                if let Some(command_line) = self
                    .programs[program_index]
                    .clone()
                    .lock_owned()
                    .now_or_never()
                    .unwrap()
                    .get_remedy_command_line() 
                {
                    command_line.inject_value_to_variables(
                        &variable.blocking_lock().get_raw_variable_name(), 
                        variable.blocking_lock().get_value()?
                    )?;
                }
            }
        }

        Ok(())
    }
    
    pub fn initialize_variables_on_chain_startup(&mut self) -> Result<(), Error> {
        for variable in &mut self.variables {
            if let VariableInitializationTime::OnChainStartup(_) = variable
                .blocking_lock()
                .get_initialization_time() 
            {
                let input: String = input_message(
                    &format!(
                        "Please input a value for {}:",
                        variable
                            .blocking_lock()
                            .get_human_readable_name()
                    )
                )?;
                variable
                    .blocking_lock()
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
    pub fn initialize_variables_on_program_execution(&mut self, program_index: usize) -> Result<(), Error> {
        for argument in self
            .programs[program_index]
            .blocking_lock()
            .get_command_line()
            .get_arguments()
        {
            let mut program_variables: Vec<Variable> = Variable::parse_variables_from_str(
                argument,
                program_index
            )?;
            for program_variable in &mut program_variables {
                for variable in &mut self.variables {
                    if program_variable.get_raw_variable_name() == variable
                        .blocking_lock()
                        .get_raw_variable_name() &&
                        matches!(
                            program_variable.get_initialization_time(),
                            VariableInitializationTime::OnProgramExecution(_)
                        )
                    {
                        let input: String = input_message(
                            &format!(
                                "Please input a value for {}:",
                                variable.blocking_lock().get_human_readable_name()
                            )
                        )?;
                        variable.blocking_lock().register_value(input.trim().to_string());
                    }
                }
            }
        }
        Ok(())
    }
    
    pub async fn handle_program_execution_failures(
        &self, 
        program_index: usize,
        error_message: &str
    ) -> Result<(), Error> {
        // Increment the failure count
        self.increment_failed_execution();
        // Acquire a mut reference to the program
        let mut program = self.programs[program_index]
            .blocking_lock();
        // Display error message
        display_message(Level::Error, &error_message);
        
        if let Some(command) = program
            .get_remedy_command_line() 
        {
            display_message(
                Level::Logging, 
                &format!(
                    "Remedy command is set. Try executing: {}", 
                    command
                )
            );
            // execute the remedy command line if any
            program.execute_remedy_command_line().await?;
        }
        
        if !program.get_failure_handling_options().exit_on_failure {
            display_message(
                Level::Warn, 
                &format!(
                    "`exit_on_failure` is set to false. Continue executing the chain..."
                )
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
            )
        );
    }
    
    /// Try making the chain execution into concurrent batches. 
    /// If no concurrent groups are found, it will return None
    pub fn try_get_concurrent_groups(&self) -> Option<Vec<Vec<Arc<Mutex<Program>>>>> {}
}

impl std::fmt::Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut program_names: String = String::new();
        for program in &self.programs {
            program_names.push_str(
                &(
                    program.blocking_lock().to_string() + "\n"
                )
            )
        }
        f.write_str(&format!("{}", program_names))
    }
}

impl VariableGroupControl for Chain {
    fn get_value(&self, variable_name: &str) -> Result<String, Error> {
        for variable in &self.variables {
            if variable.blocking_lock().get_variable_name() == variable_name {
                return Ok(variable.blocking_lock().get_value()?);
            }
        }
        
        return Err(anyhow!("Variable {} does not exist!", variable_name));
    }

    fn update_value(&mut self, variable_name: &str, value: String) {
        for variable in &mut self.variables {
            if variable.blocking_lock().get_raw_variable_name() == variable_name {
                variable.blocking_lock().register_value(value);
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
        self.initialize_variables_on_chain_startup()?;
        
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
                let program = &mut self.programs[i];
                // Process any functions provided as arguments for the program.
                program.blocking_lock().execute_argument_functions().await?;
            }
            
            // Insert available variables from the chain into the program's context.
            self.insert_variable(i)?;
            
            // Check if the program returns an awaitable variable.
            if let Some(variable) = self.programs[i]
                .blocking_lock()
                .get_awaitable_variable()
                .clone() 
            {
                // Execute the program and capture its output.
                let output: String = match self.programs[i]
                    .blocking_lock()
                    .execute()
                    .await 
                {
                    Ok(result) => result,
                    Err(error) => {
                        self.handle_program_execution_failures(
                            i, 
                            &error.to_string()
                        ).await?;
                        
                        continue;
                    }
                };
                // Return the awaitable variable along with the captured output.
                awaitable_variable = Some(variable);
                awaitable_value = Some(output);
            } else {
                // If there is no awaitable variable, simply execute the program.
                match self.programs[i].blocking_lock().execute().await {
                    Ok(result) => result,
                    Err(error) => {
                        self.handle_program_execution_failures(
                            i, 
                            &error.to_string()
                        ).await?;
                        
                        continue;
                    }
                };
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

impl ConcurrentExecution for Chain {
    fn get_execution_type(&self) -> &ExecutionType {
        &ExecutionType::Chain
    }
    
    async fn execute_concurrently(&mut self) -> Result<String, Error> {
        // See if any program needs input on startup
        self.initialize_variables_on_chain_startup()?;

        // Determine the concurrency groups and the sequential execution orders
        if let Some(concurrency_groups) = self
            .try_get_concurrent_groups() 
        {
            // Execute in order with the concurrency groups
            for group in concurrency_groups {
                let mut tasks = Vec::new();

                for program in group {
                    let mut program_clone = program.clone();
                    let program_index: usize = self.programs
                        .iter()
                        .position(
                            |program|
                            *program.blocking_lock() == *program_clone.blocking_lock()
                        )
                        .unwrap();

                    tasks.push(
                        tokio::spawn(
                            async move {
                                // Create a variable to record outputs from the command
                                let output: String;
                                // Create a single block for clearing the mut ref to the 
                                // self.programs.
                                {
                                    // Get a mutable reference to the current program.
                                    let program = &mut self.programs[program_index];
                                    // Process any functions provided as arguments for the program.
                                    program.blocking_lock().execute_argument_functions().await?;
                                }

                                // Record awaitable variable if any
                                let mut awaitable_variable: Option<String> = None;
                                let mut awaitable_value: Option<String> = None;

                                // Insert available variables from the chain into the program's context.
                                self.insert_variable(program_index)?;

                                // Check if the program returns an awaitable variable.
                                if let Some(variable) = self.programs[program_index]
                                    .blocking_lock()
                                    .get_awaitable_variable()
                                    .clone() 
                                {
                                    // Execute the program and capture its output.
                                    output = self.programs[program_index]
                                        .blocking_lock()
                                        .execute()
                                        .await?;
                                    // Return the awaitable variable along with the captured output.
                                    awaitable_variable = Some(variable);
                                    awaitable_value = Some(output);
                                } else {
                                    // If there is no awaitable variable, simply execute the program.
                                    output = self.programs[program_index].blocking_lock().execute().await?;
                                }

                                // If the program returned an awaitable variable and output, update the chain's variable.
                                if awaitable_value.is_some() && awaitable_variable.is_some() {
                                    self.update_value(
                                        &awaitable_variable.unwrap(), 
                                        awaitable_value.unwrap()
                                    );
                                }

                                Ok::<ConcurrencyGroupExecutionResult, Error>(
                                    ConcurrencyGroupExecutionResult::new(program_index, output)
                                )
                            }
                        )
                    );
                }

                let results = join_all(tasks).await;
                for result in results {
                    let result = result??;
                    match result {
                        Ok((program_index, output)) => {
                            if is_error {
                                self.handle_program_execution_failures(program_index, &output)
                                    .await?;
                            } else {
                                // Handle successful execution if needed
                            }
                        }
                        Err(error) => {
                            // Handle error that occurred while setting up or running the task
                            eprintln!("Task failed with error: {}", error);
                        }
                    }
                }
            }
        } else {
            return Err(anyhow!("No concurrency groups are detected"))
        }
        
        Ok("Done".to_string())
    }
}