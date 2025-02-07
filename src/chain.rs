use anyhow::{Result, Error};

use crate::{program::Program, utility::{execute_argument_function, Execution, ExecutionType}};

pub struct Chain {
    programs: Vec<Program>
}

impl Chain {
    pub fn from_file(path: &str) -> Result<Self, Error> {
        let programs: Vec<Program> = serde_json::from_str(
            &std::fs::read_to_string(&path)
                .expect("Failed to load configurations"),
        )?;

        Ok(
            Self {
                programs
            }
        )
    }
}

impl std::fmt::Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut program_names: String = String::new();
        for program in &self.programs {
            program_names.push_str(
                &(program.to_string() + "\n")
            )
        }
        f.write_str(&format!("{}", program_names))
    }
}

impl Execution for Chain {
    fn get_execution_type(&self) -> &crate::utility::ExecutionType {
        &ExecutionType::Chain
    }

    async fn execute(&mut self) -> Result<(), Error> {
        // Iterate over each configuration and execute the commands
        for mut program in &mut self.programs {
            execute_argument_function(&mut program).await?;
            program.execute().await?;
        }

        Ok(())
    }
}