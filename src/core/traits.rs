use std::fmt::Display;

use anyhow::{Error, Result};

pub enum ExecutionType {
    Chain,
    Program,
    Function,
    CommandLine,
    ConcurrencyGroup,
}

impl std::fmt::Display for ExecutionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionType::Chain => f.write_str("Chain"),
            ExecutionType::Program => f.write_str("Program"),
            ExecutionType::Function => f.write_str("Function"),
            ExecutionType::CommandLine => f.write_str("Command Line"),
            ExecutionType::ConcurrencyGroup => f.write_str("Concurrency Group"),
        }
    }
}

/// Anything that can be executed
pub trait Execution<T>
where
    Self: Display,
    T: Clone + Eq + PartialEq + Send + Sync + 'static,
{
    fn get_execution_type(&self) -> &ExecutionType;

    fn execute(&mut self) -> Result<Vec<T>, Error>;
}
