use anyhow::{anyhow, Error, Ok, Result};
use regex;

/// note
/// three conditions in which the value of a variable is supplied
/// 1. on the chain's startup.
///     - marked by a new syntax.
///         - `:on_program_execution` to ask for value on startup of a program
/// 2. when executing a program.
///     - marked by a new syntax.
///         - unmarked is executed on program startup.
/// 3. get a program's output as a value.
///     - marked by a key called `stdout_stored_to` in the config.

/// For determing the variable lifetime in a chain
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VariableLifetime {
    /// When does this variable gurantee to initialize
    initialization_program_index: usize,
}

impl VariableLifetime {
    pub fn new(initialization_program_index: Option<usize>) -> Self {
        Self {
            // Default to 0 for OnChainStartup variables
            initialization_program_index: initialization_program_index.unwrap_or(0),
        }
    }
}

/// Denotes the different times at which a variable should be initialized.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableInitializationTime {
    /// Initialized on the chain's startup
    OnChainStartup(VariableLifetime),
    /// Initialized when executing a program where the variable
    /// is not pre-assigned on startup.
    /// Triggered by `:on_program_execution` syntax.
    OnProgramExecution(VariableLifetime),
    /// Initialization is deferred until the variable's value
    /// is obtained from a program's output.
    Await(VariableLifetime),
}

impl VariableInitializationTime {
    /// Determine whether the variable is initialized in the current
    /// program
    pub fn is_initialized(&self, program_index: usize) -> bool {
        match self {
            VariableInitializationTime::OnChainStartup(lifetime) => {
                lifetime.initialization_program_index == program_index
            }
            VariableInitializationTime::OnProgramExecution(lifetime) => {
                lifetime.initialization_program_index == program_index
            }
            VariableInitializationTime::Await(lifetime) => {
                lifetime.initialization_program_index == program_index
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Variable {
    /// The name for the variable
    name: String,
    /// The actual value for the variable
    value: Option<String>,
    /// The time when the variable needs to be initialized with a value
    initialization_time: VariableInitializationTime,
    /// The name for users to read on the screen
    human_readable_name: String,
}

impl Variable {
    pub fn new(
        name: String,
        value: Option<String>,
        mut human_readable_name: Option<String>,
        initialization_time: VariableInitializationTime,
    ) -> Self {
        if human_readable_name.is_none() {
            human_readable_name = Some(
                name.split('_')
                    .map(|word| {
                        let mut c = word.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(" "),
            );
        }

        Self {
            name,
            value,
            human_readable_name: human_readable_name.unwrap(),
            initialization_time,
        }
    }

    /// Parses all variables from the input string.
    ///
    /// This is the main method for extracting variables from a string. It searches for substrings
    /// enclosed in `<<` and `>>`, extracts each variable's name and its initialization qualifier (if any),
    /// and then creates a corresponding `Variable` instance.
    ///
    /// The qualifier is determined by checking if the variable name contains a colon (:) separating
    /// the actual name from its qualifier. If the qualifier is `"on_program_execution"`, the variable
    /// is set to initialize during program execution; otherwise, it defaults to chain startup.
    ///
    /// # Arguments
    ///
    /// * `s` - A string slice that may contain variable placeholders.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Variable>, Error>` - A result containing a vector of `Variable` instances
    ///   if successful, or an `Error` if parsing fails.
    pub fn parse_variables_from_str(s: &str, program_index: usize) -> Result<Vec<Variable>, Error> {
        let mut variables: Vec<Variable> = Vec::new();

        // Iterate over each occurrence of a variable placeholder in the string.
        for raw_var in Self::extract_variable_names(s) {
            // For each placeholder, determine the variable name and its initialization time.
            let (name, init_time) = Self::parse_initialization_time(raw_var, program_index);
            // Create a new Variable instance with no assigned value and no human readable name override.
            variables.push(Variable::new(name, None, None, init_time));
        }

        Ok(variables)
    }

    /// Parses the variable name and its initialization time from a raw variable string.
    ///
    /// The function expects the input to be in one of the two formats:
    /// - "variable" (defaults to OnChainStartup)
    /// - "variable:qualifier"
    ///
    /// If a qualifier is provided and it matches "on_program_execution" (case-insensitive),
    /// the variable's initialization time is set to `OnProgramExecution`. Otherwise, it defaults
    /// to `OnChainStartup`.
    ///
    /// # Arguments
    ///
    /// * `s` - A string slice representing the raw variable content.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - A String with the variable name.
    /// - A `VariableInitializationTime` reflecting when the variable should be initialized.
    fn parse_initialization_time(
        s: &str,
        program_index: usize,
    ) -> (String, VariableInitializationTime) {
        // Expects s to be either "variable" or "variable:qualifier"
        if let Some(idx) = s.find(':') {
            let name = s[..idx].to_string();
            let qualifier = s[idx + 1..].to_lowercase();
            let init_time = match qualifier.as_str() {
                "on_program_execution" => VariableInitializationTime::OnProgramExecution(
                    VariableLifetime::new(Some(program_index)),
                ),
                _ => VariableInitializationTime::OnChainStartup(VariableLifetime::new(None)),
            };
            (name, init_time)
        } else {
            (
                s.to_string(),
                VariableInitializationTime::OnChainStartup(VariableLifetime::new(None)),
            )
        }
    }

    /// Parses a variable that is expected to be awaited.
    ///
    /// This function assumes that the input string is formatted as `"<<variable>>"`.
    /// It extracts the variable name and creates a `Variable` with an initialization time
    /// of `Await`, indicating that its value will be obtained later from a program's output.
    ///
    /// # Arguments
    ///
    /// * `s` - A string slice in the format `"<<variable>>"`.
    ///
    /// # Returns
    ///
    /// A `Variable` instance with an initialization time set to `Await`.
    pub fn parse_await_variable(s: &str, program_index: usize) -> Variable {
        let trimmed = s.trim();
        let var_name = trimmed
            .trim_start_matches("<<")
            .trim_end_matches(">>")
            .to_string();
        Variable::new(
            var_name,
            None,
            None,
            VariableInitializationTime::Await(VariableLifetime::new(Some(program_index))),
        )
    }

    /// Extracts variable names from the provided string.
    ///
    /// This function searches the input string for substrings enclosed in `<<` and `>>`
    /// and extracts the variable names contained within.
    ///
    /// # Arguments
    ///
    /// * `s` - A string slice that may contain variable placeholders.
    ///
    /// # Returns
    ///
    /// A vector of string slices representing the extracted variable names.
    pub fn extract_variable_names(s: &str) -> Vec<&str> {
        let re = regex::Regex::new(r"<<([^>]*)>>").unwrap();
        re.find_iter(s)
            .map(|item| {
                let trim_pattern_start: [char; 2] = ['<', '<'];
                let trim_pattern_end: [char; 2] = ['>', '>'];
                item.as_str()
                    .trim_start_matches(trim_pattern_start)
                    .trim_end_matches(trim_pattern_end)
            })
            .collect()
    }

    pub fn register_value<S>(&mut self, value: S)
    where
        S: ToString,
    {
        self.value = Some(value.to_string());
    }

    pub fn get_value(&self) -> Result<String, Error> {
        match &self.value {
            Some(value) => return Ok(value.to_string()),
            None => return Err(anyhow!("Value for {} is empty", self.name)),
        }
    }

    pub fn get_human_readable_name(&self) -> &str {
        &self.human_readable_name
    }

    /// Variable name without additional syntax like `:`
    pub fn get_variable_name(&self) -> &str {
        &self.name
    }

    /// Complete variable name with additional syntax
    pub fn get_raw_variable_name(&self) -> String {
        match self.initialization_time {
            VariableInitializationTime::OnProgramExecution { .. } => {
                "<<".to_string() + &self.name + ":" + "on_program_execution" + ">>"
            }
            _ => "<<".to_string() + &self.name + ">>",
        }
    }

    pub fn get_initialization_time(&self) -> VariableInitializationTime {
        self.initialization_time
    }
}

pub trait VariableGroupControl {
    fn update_value(&mut self, variable_name: &str, value: String);

    fn get_value(&self, variable_name: &str) -> Result<String, Error>;
}
