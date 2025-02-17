use std::{cell::Cell, collections::HashSet, sync::Arc};

use anyhow::{anyhow, Chain, Error, Result};
use futures::future::join_all;
use tokio::sync::Mutex;

use super::{program::{Program, ProgramExecutionResult}, traits::{Execution, ExecutionType}};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ConcurrencyGroupExecutionResult {
    program_index: usize,
    output: Option<String>,
    error: Option<Error>,
}

impl ConcurrencyGroupExecutionResult {
    pub fn new(program_index: usize, result: Result<Vec<ProgramExecutionResult>, Error>) -> Self {
        match result {
            Ok(result) => {
                return Self {
                    program_index,
                    output: Some(result[0].get_output()),
                    error: None,
                }
            }
            Err(error) => {
                return Self {
                    program_index,
                    output: None,
                    error: Some(error),
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConcurrencyProgram {
    program: Arc<Mutex<Program>>,
    index: usize,
}

impl ConcurrencyProgram {
    pub fn new(program: Arc<Mutex<Program>>, index: usize) -> Self {
        Self { program, index }
    }
}

#[derive(Debug)]
pub struct ConcurrencyGroup {
    group_number: Option<usize>,
    programs: Vec<Arc<Mutex<ConcurrencyProgram>>>,
    failed_program_executions: Cell<usize>,
}

impl ConcurrencyGroup {
    pub fn new() -> Self {
        Self {
            group_number: None,
            programs: vec![],
            failed_program_executions: Cell::new(0),
        }
    }

    pub fn get_group_number(&self) -> Result<usize, Error> {
        match self.group_number {
            Some(number) => Ok(number),
            None => Err(anyhow!(
                "Group number is None. This concurrency group may not yet be populated."
            )),
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
    pub fn push(&mut self, program: Arc<Mutex<Program>>, program_index: usize) -> bool {
        if self.group_number.is_none() {
            if program.blocking_lock().get_concurrency_group() {
                self.programs.push(ConcurrencyProgram::new(program, program_index));

                return true;
            } else {
                self.group_number = 0;
                self.programs.push(ConcurrencyProgram::new(program, program_index));

                return true;
            }
        }

        if self.group_number.is_some() {
            if program.blocking_lock().get_concurrency_group() == self.group_number {
                self.programs.push(ConcurrencyProgram::new(program, program_index));

                return true;
            }
        }

        false
    }

    pub fn from_chain(chain: Chain) -> Result<Vec<ConcurrencyGroup>, Error> {
        // Initialize a vec to contain all concurrency groups
        let mut concurrency_groups: Vec<ConcurrencyGroup> = Vec::new();
        for (index, program) in chain.programs.into_iter().enumerate() {
            // concurrency groups contain a group that has this program
            for concurrency_group in concurrency_groups {
                if concurrency_group.push(program, index) {
                    break;
                }
            }
            
            // concurrency groups contain a group that DOES NOT have this program
            // &&
            // concurrency group is empty
            let mut concurrency_group = ConcurrencyGroup::new();
            concurrency_group.push(program, index);
            concurrency_groups.push(concurrency_group);
        }

        // If all concurrency groups contain only one program,
        // that means there is no concurrency, so return None
        if concurrency_groups.iter().all(|item| item.len() == 1) {
            return Err(anyhow!(
                "No concurrency groups are found. Please execute the chain directly."
            ));
        }

        Ok(concurrency_groups)
    }
}

impl std::fmt::Display for ConcurrencyGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Concurrency group number: {}", self.group_number))
    }
}

impl Execution<ConcurrencyGroupExecutionResult> for ConcurrencyGroup {
    fn get_execution_type(&self) -> &ExecutionType {
        &ExecutionType::Chain
    }

    async fn execute(&mut self) -> Result<Vec<ConcurrencyGroupExecutionResult>, Error> {
        // Store the tasks to execute concurrently
        let mut tasks: Vec<tokio::task::JoinHandle<std::result::Result<Vec<ProgramExecutionResult>, Error>>> = Vec::new();
        // Store the final resutls
        let mut final_results: Vec<ConcurrencyGroupExecutionResult> = Vec::new();

        for concurrency_program in self.programs
            .clone() 
        {
            tasks.push(
                tokio::spawn(
                    async move {
                        let program = concurrency_program.lock().await;
                        let result = program.program.lock().await.execute().await;
                        result
                    }
                )
            );
        }
        
        let results = join_all(tasks).await;
        for (index, result) in results.into_iter().enumerate() {
            match result? {
                Ok(result) => {
                    final_results.push(ConcurrencyGroupExecutionResult::new(index, Ok(result)))
                }
                Err(error) => {
                    final_results.push(ConcurrencyGroupExecutionResult::new(index, Err(error)));
                }
            };
        }
        
        Ok(final_results)
    }
}
